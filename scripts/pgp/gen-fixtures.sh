#!/usr/bin/env bash
# Regenerate the GnuPG fixtures for mw-crypto's OpenPGP tests (t32-e37) into
# fixtures/crypto/pgp/e37/. Needs GnuPG 2.2+ on PATH (the GnuPG container of the
# interop job, or a host gpg). Every key is made in a throwaway GNUPGHOME; the
# passphrase of every secret key is `interop-pass`. Re-running replaces the files
# with new keys, so the tests read only what is committed.
#
#   rsa-signsub-*   RSA-3072 primary [C] + RSA signing subkey [S] + RSA encryption
#                   subkey [E]; data.sig is made by the signing subkey.
#   revoked-*       Ed25519 + cv25519 key revoked (no reason given: hard) after it signed
#                   data.sig.
#   expired-*       Ed25519 + cv25519 key created 2020-01-01, expiring a year later, that
#                   signed data.sig in 2020 (valid then).
#   sha1-data.sig   a detached signature by the existing Bob key (../gnupg-*.asc)
#                   made with SHA-1, which the verifier refuses.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT="${ROOT}/fixtures/crypto/pgp/e37"
PASS="interop-pass"
mkdir -p "${OUT}"

GNUPGHOME="$(mktemp -d)"
export GNUPGHOME
trap 'gpgconf --kill gpg-agent >/dev/null 2>&1 || true; rm -rf "${GNUPGHOME}"' EXIT
echo "allow-loopback-pinentry" > "${GNUPGHOME}/gpg-agent.conf"

g() { gpg --batch --yes --pinentry-mode loopback --passphrase "${PASS}" "$@"; }
fpr() { gpg --list-keys --with-colons "$1" | awk -F: '/^fpr:/{print $10; exit}'; }

printf 'signed by a signing subkey\n' > "${OUT}/rsa-signsub-data.txt"
printf 'signed before the key was revoked\n' > "${OUT}/revoked-data.txt"
printf 'signed while the key was valid\n' > "${OUT}/expired-data.txt"
printf 'signed with SHA-1\n' > "${OUT}/sha1-data.txt"

# RSA primary (certify only) with separate signing and encryption subkeys.
g --quick-gen-key "Rita RSA <rita@example.com>" rsa3072 cert never
FPR="$(fpr rita@example.com)"
g --quick-add-key "${FPR}" rsa3072 sign never
g --quick-add-key "${FPR}" rsa3072 encr never
gpg --armor --export rita@example.com > "${OUT}/rsa-signsub-public.asc"
g --armor --export-secret-keys rita@example.com > "${OUT}/rsa-signsub-secret.asc"
g --armor --detach-sign --digest-algo SHA256 -u rita@example.com \
  -o "${OUT}/rsa-signsub-data.sig" "${OUT}/rsa-signsub-data.txt"

# Revoked: sign first, then import the revocation certificate gpg wrote at key
# creation. gpg prefixes its armor line with ':' so that it is not imported by
# accident; strip that.
g --quick-gen-key "Rob Revoked <rob@example.com>" future-default default never
g --armor --detach-sign -u rob@example.com -o "${OUT}/revoked-data.sig" "${OUT}/revoked-data.txt"
RFPR="$(fpr rob@example.com)"
sed 's/^:-----BEGIN/-----BEGIN/' "${GNUPGHOME}/openpgp-revocs.d/${RFPR}.rev" > "${GNUPGHOME}/rev.asc"
gpg --batch --import "${GNUPGHOME}/rev.asc"
gpg --armor --export rob@example.com > "${OUT}/revoked-public.asc"

# Expired: made and used in 2020 with a one-year expiry.
g --faked-system-time 20200101T000000 --quick-gen-key "Eve Expired <eve@example.com>" future-default default 1y
g --faked-system-time 20200601T000000 --armor --detach-sign -u eve@example.com \
  -o "${OUT}/expired-data.sig" "${OUT}/expired-data.txt"
gpg --armor --export eve@example.com > "${OUT}/expired-public.asc"

# SHA-1 detached signature by Bob (existing fixture key).
g --import "${ROOT}/fixtures/crypto/pgp/gnupg-secret.asc"
g --armor --detach-sign --digest-algo SHA1 --allow-weak-digest-algos -u bob@example.com \
  -o "${OUT}/sha1-data.sig" "${OUT}/sha1-data.txt"

echo "fixtures written to ${OUT}"
