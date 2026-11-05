//! MAILWOMAN PATCH: SQLCipher crypto provider implemented with RustCrypto.
//!
//! SQLCipher normally delegates AES-256-CBC, HMAC, PBKDF2 and the CSPRNG to
//! OpenSSL (or CommonCrypto / NSS / libtomcrypt). This workspace bans OpenSSL, so
//! the amalgamation is built without a C crypto backend and `build.rs` makes
//! SQLCipher call [`mw_sqlcipher_setup`] to fill its provider table. The on-disk
//! format is unchanged SQLCipher 4 (AES-256-CBC + per-page HMAC-SHA512, PBKDF2-
//! HMAC-SHA512), so files interoperate with stock SQLCipher builds.
//!
//! Call [`ensure_linked`] once before opening a connection: it makes the linker
//! keep this module's object in the final binary on platforms where static
//! archive members are only pulled in when referenced.

use aes::Aes256;
use cbc::cipher::block_padding::NoPadding;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use std::os::raw::{c_char, c_int, c_void};

const SQLITE_OK: c_int = 0;
const SQLITE_ERROR: c_int = 1;

const ALG_SHA1: c_int = 0;
const ALG_SHA256: c_int = 1;
const ALG_SHA512: c_int = 2;

/// Mirror of `sqlcipher_provider` in the SQLCipher amalgamation (field order matters).
#[repr(C)]
pub struct SqlcipherProvider {
    activate: unsafe extern "C" fn(*mut c_void) -> c_int,
    deactivate: unsafe extern "C" fn(*mut c_void) -> c_int,
    get_provider_name: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    add_random: unsafe extern "C" fn(*mut c_void, *mut c_void, c_int) -> c_int,
    random: unsafe extern "C" fn(*mut c_void, *mut c_void, c_int) -> c_int,
    hmac: unsafe extern "C" fn(
        *mut c_void,
        c_int,
        *mut u8,
        c_int,
        *mut u8,
        c_int,
        *mut u8,
        c_int,
        *mut u8,
    ) -> c_int,
    kdf: unsafe extern "C" fn(
        *mut c_void,
        c_int,
        *const u8,
        c_int,
        *mut u8,
        c_int,
        c_int,
        c_int,
        *mut u8,
    ) -> c_int,
    cipher: unsafe extern "C" fn(
        *mut c_void,
        c_int,
        *mut u8,
        c_int,
        *mut u8,
        *mut u8,
        c_int,
        *mut u8,
    ) -> c_int,
    get_cipher: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    get_key_sz: unsafe extern "C" fn(*mut c_void) -> c_int,
    get_iv_sz: unsafe extern "C" fn(*mut c_void) -> c_int,
    get_block_sz: unsafe extern "C" fn(*mut c_void) -> c_int,
    get_hmac_sz: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    ctx_init: unsafe extern "C" fn(*mut *mut c_void) -> c_int,
    ctx_free: unsafe extern "C" fn(*mut *mut c_void) -> c_int,
    fips_status: unsafe extern "C" fn(*mut c_void) -> c_int,
    get_provider_version: unsafe extern "C" fn(*mut c_void) -> *const c_char,
}

unsafe extern "C" fn ok(_: *mut c_void) -> c_int {
    SQLITE_OK
}
unsafe extern "C" fn name(_: *mut c_void) -> *const c_char {
    c"rustcrypto".as_ptr()
}
unsafe extern "C" fn version(_: *mut c_void) -> *const c_char {
    c"aes 0.8 / cbc 0.1 / hmac 0.12 / sha2 0.10 / pbkdf2 0.12".as_ptr()
}
unsafe extern "C" fn cipher_name(_: *mut c_void) -> *const c_char {
    c"AES-256-CBC".as_ptr()
}
unsafe extern "C" fn key_sz(_: *mut c_void) -> c_int {
    32
}
unsafe extern "C" fn iv_sz(_: *mut c_void) -> c_int {
    16
}
unsafe extern "C" fn block_sz(_: *mut c_void) -> c_int {
    16
}
unsafe extern "C" fn fips(_: *mut c_void) -> c_int {
    0
}
unsafe extern "C" fn hmac_sz(_: *mut c_void, alg: c_int) -> c_int {
    match alg {
        ALG_SHA1 => 20,
        ALG_SHA256 => 32,
        ALG_SHA512 => 64,
        _ => 0,
    }
}
unsafe extern "C" fn ctx_init(_: *mut *mut c_void) -> c_int {
    SQLITE_OK
}
unsafe extern "C" fn ctx_free(_: *mut *mut c_void) -> c_int {
    SQLITE_OK
}
/// The OS CSPRNG needs no extra entropy.
unsafe extern "C" fn add_random(_: *mut c_void, _: *mut c_void, _: c_int) -> c_int {
    SQLITE_OK
}

unsafe extern "C" fn random(_: *mut c_void, buf: *mut c_void, len: c_int) -> c_int {
    if len < 0 || buf.is_null() {
        return SQLITE_ERROR;
    }
    // SAFETY: SQLCipher passes a writable buffer of `len` bytes.
    let out = unsafe { std::slice::from_raw_parts_mut(buf.cast::<u8>(), len as usize) };
    match getrandom::getrandom(out) {
        Ok(()) => SQLITE_OK,
        Err(_) => SQLITE_ERROR,
    }
}

fn mac<M: Mac + hmac::digest::KeyInit>(key: &[u8], a: &[u8], b: &[u8], out: &mut [u8]) -> c_int {
    let Ok(mut m) = <M as Mac>::new_from_slice(key) else {
        return SQLITE_ERROR;
    };
    m.update(a);
    if !b.is_empty() {
        m.update(b);
    }
    out.copy_from_slice(&m.finalize().into_bytes());
    SQLITE_OK
}

unsafe extern "C" fn hmac_fn(
    _: *mut c_void,
    alg: c_int,
    key: *mut u8,
    key_len: c_int,
    in1: *mut u8,
    in1_len: c_int,
    in2: *mut u8,
    in2_len: c_int,
    out: *mut u8,
) -> c_int {
    if key.is_null() || in1.is_null() || out.is_null() || key_len < 0 || in1_len < 0 || in2_len < 0
    {
        return SQLITE_ERROR;
    }
    // SAFETY: SQLCipher guarantees the pointer/length pairs; `in2` is optional.
    let (key, a) = unsafe {
        (
            std::slice::from_raw_parts(key, key_len as usize),
            std::slice::from_raw_parts(in1, in1_len as usize),
        )
    };
    let b: &[u8] = if in2.is_null() || in2_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(in2, in2_len as usize) }
    };
    let n = unsafe { hmac_sz(std::ptr::null_mut(), alg) } as usize;
    if n == 0 {
        return SQLITE_ERROR;
    }
    let out = unsafe { std::slice::from_raw_parts_mut(out, n) };
    match alg {
        ALG_SHA1 => mac::<Hmac<sha1::Sha1>>(key, a, b, out),
        ALG_SHA256 => mac::<Hmac<sha2::Sha256>>(key, a, b, out),
        _ => mac::<Hmac<sha2::Sha512>>(key, a, b, out),
    }
}

unsafe extern "C" fn kdf(
    _: *mut c_void,
    alg: c_int,
    pass: *const u8,
    pass_len: c_int,
    salt: *mut u8,
    salt_len: c_int,
    workfactor: c_int,
    key_len: c_int,
    key: *mut u8,
) -> c_int {
    if pass_len < 0 || salt_len < 0 || workfactor < 1 || key_len < 0 || key.is_null() {
        return SQLITE_ERROR;
    }
    let pass: &[u8] = if pass.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(pass, pass_len as usize) }
    };
    let salt: &[u8] = if salt.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(salt, salt_len as usize) }
    };
    let out = unsafe { std::slice::from_raw_parts_mut(key, key_len as usize) };
    let n = workfactor as u32;
    match alg {
        ALG_SHA1 => pbkdf2::pbkdf2_hmac::<sha1::Sha1>(pass, salt, n, out),
        ALG_SHA256 => pbkdf2::pbkdf2_hmac::<sha2::Sha256>(pass, salt, n, out),
        ALG_SHA512 => pbkdf2::pbkdf2_hmac::<sha2::Sha512>(pass, salt, n, out),
        _ => return SQLITE_ERROR,
    }
    SQLITE_OK
}

/// `mode`: 1 = encrypt, 0 = decrypt (SQLCipher's CIPHER_ENCRYPT / CIPHER_DECRYPT).
/// No padding: SQLCipher only passes whole 16-byte blocks.
unsafe extern "C" fn cipher(
    _: *mut c_void,
    mode: c_int,
    key: *mut u8,
    key_len: c_int,
    iv: *mut u8,
    input: *mut u8,
    in_len: c_int,
    out: *mut u8,
) -> c_int {
    if key.is_null() || iv.is_null() || input.is_null() || out.is_null() {
        return SQLITE_ERROR;
    }
    if key_len != 32 || in_len < 0 || in_len % 16 != 0 {
        return SQLITE_ERROR;
    }
    let (key, iv, input) = unsafe {
        (
            std::slice::from_raw_parts(key, 32),
            std::slice::from_raw_parts(iv, 16),
            std::slice::from_raw_parts(input, in_len as usize),
        )
    };
    let out = unsafe { std::slice::from_raw_parts_mut(out, in_len as usize) };
    let done = if mode == 1 {
        cbc::Encryptor::<Aes256>::new_from_slices(key, iv)
            .ok()
            .and_then(|c| c.encrypt_padded_b2b_mut::<NoPadding>(input, out).ok().map(|_| ()))
    } else {
        cbc::Decryptor::<Aes256>::new_from_slices(key, iv)
            .ok()
            .and_then(|c| c.decrypt_padded_b2b_mut::<NoPadding>(input, out).ok().map(|_| ()))
    };
    if done.is_some() { SQLITE_OK } else { SQLITE_ERROR }
}

/// Called from the patched SQLCipher amalgamation when it needs a default provider.
///
/// # Safety
/// `p` must point to a writable `sqlcipher_provider`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mw_sqlcipher_setup(p: *mut SqlcipherProvider) -> c_int {
    if p.is_null() {
        return SQLITE_ERROR;
    }
    // SAFETY: the table was allocated by SQLCipher with the C struct's size.
    unsafe {
        std::ptr::write(
            p,
            SqlcipherProvider {
                activate: ok,
                deactivate: ok,
                get_provider_name: name,
                add_random,
                random,
                hmac: hmac_fn,
                kdf,
                cipher,
                get_cipher: cipher_name,
                get_key_sz: key_sz,
                get_iv_sz: iv_sz,
                get_block_sz: block_sz,
                get_hmac_sz: hmac_sz,
                ctx_init,
                ctx_free,
                fips_status: fips,
                get_provider_version: version,
            },
        );
    }
    SQLITE_OK
}

/// Keeps `mw_sqlcipher_setup` in the final link (see the module note).
pub fn ensure_linked() {
    let f: unsafe extern "C" fn(*mut SqlcipherProvider) -> c_int = mw_sqlcipher_setup;
    std::hint::black_box(f);
}
