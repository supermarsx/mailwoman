// The OPFS filesystem backend (t19-e12, tag 26.19).
//
// `offline/opfs.test.ts` already covers `EncryptedCache` against
// `memoryBackend()` — the crypto round trip and the path layout. What had NO
// coverage is `opfsBackend()` itself and the `resolveDir` path walk underneath
// it: the half that only exists in a real browser, and therefore the half most
// likely to rot unnoticed.
//
// jsdom ships no Origin Private File System, and a polyfill would be a new
// devDependency this tag does not take, so this file stands up a small fake
// implementing the slice of the `FileSystemDirectoryHandle` API the backend
// actually calls. That is enough to pin the decisions the backend makes —
// which is where its behaviour lives; the storage itself belongs to the browser.
//
// The contract being pinned is a specific one and easy to break by accident:
// a MISSING file must read back as `null` ("not cached"), not throw, because
// every cached-read path treats a throw as a hard failure.

import { describe, it, expect, afterEach, vi } from 'vitest';
import { memoryBackend, opfsAvailable, opfsBackend } from './opfs.ts';

// ── a minimal in-memory FileSystemDirectoryHandle ───────────────────────────

class FakeFile {
  constructor(public bytes: Uint8Array) {}
  async arrayBuffer(): Promise<ArrayBuffer> {
    // Copy into a standalone ArrayBuffer, as a real File does.
    return this.bytes.slice().buffer as ArrayBuffer;
  }
}

class FakeFileHandle {
  constructor(private readonly dir: FakeDir, private readonly name: string) {}
  async getFile(): Promise<FakeFile> {
    const bytes = this.dir.files.get(this.name);
    if (bytes === undefined) throw notFound(this.name);
    return new FakeFile(bytes);
  }
  async createWritable(): Promise<{
    write(data: BufferSource): Promise<void>;
    close(): Promise<void>;
  }> {
    let staged: Uint8Array | null = null;
    const dir = this.dir;
    const name = this.name;
    return {
      async write(data: BufferSource) {
        staged = new Uint8Array(data as unknown as ArrayBufferLike);
      },
      async close() {
        // A real writable only publishes on close.
        if (staged !== null) dir.files.set(name, staged);
      },
    };
  }
}

function notFound(name: string): DOMException {
  const err = new Error(`NotFoundError: ${name}`);
  err.name = 'NotFoundError';
  return err as unknown as DOMException;
}

class FakeDir {
  readonly dirs = new Map<string, FakeDir>();
  readonly files = new Map<string, Uint8Array>();
  /** Every getDirectoryHandle/getFileHandle call, for asserting the walk. */
  readonly calls: string[] = [];

  async getDirectoryHandle(name: string, opts?: { create?: boolean }): Promise<FakeDir> {
    this.calls.push(`dir:${name}:${opts?.create === true ? 'create' : 'open'}`);
    let d = this.dirs.get(name);
    if (d === undefined) {
      if (opts?.create !== true) throw notFound(name);
      d = new FakeDir();
      this.dirs.set(name, d);
    }
    return d;
  }

  async getFileHandle(name: string, opts?: { create?: boolean }): Promise<FakeFileHandle> {
    this.calls.push(`file:${name}:${opts?.create === true ? 'create' : 'open'}`);
    if (!this.files.has(name)) {
      if (opts?.create !== true) throw notFound(name);
      this.files.set(name, new Uint8Array());
    }
    return new FakeFileHandle(this, name);
  }

  async removeEntry(name: string): Promise<void> {
    this.calls.push(`remove:${name}`);
    if (!this.files.delete(name)) throw notFound(name);
  }

  /** Walk to a nested directory, or `undefined`. */
  descend(...parts: string[]): FakeDir | undefined {
    return parts.reduce<FakeDir | undefined>((d, p) => d?.dirs.get(p), this as FakeDir);
  }
}

/** Install a fake `navigator.storage.getDirectory`, returning the fake root. */
function installOpfs(): FakeDir {
  const root = new FakeDir();
  vi.stubGlobal('navigator', {
    ...globalThis.navigator,
    storage: { getDirectory: async () => root },
  });
  return root;
}

afterEach(() => vi.unstubAllGlobals());

const bytes = (...n: number[]): Uint8Array => new Uint8Array(n);

// ─────────────────────────────────────────────────────────────────────────────

describe('opfsAvailable', () => {
  it('is false under jsdom, which is why memoryBackend is the fallback', () => {
    expect(opfsAvailable()).toBe(false);
  });

  it('becomes true once the API is present', () => {
    installOpfs();
    expect(opfsAvailable()).toBe(true);
  });

  it('stays false when storage exists but getDirectory does not', () => {
    // Some environments expose `navigator.storage` (quota estimation) without
    // OPFS. Probing only for `storage` would wrongly select the OPFS backend.
    vi.stubGlobal('navigator', { ...globalThis.navigator, storage: { estimate: () => ({}) } });
    expect(opfsAvailable()).toBe(false);
  });
});

describe('opfsBackend — writing', () => {
  it('creates the whole directory chain for a nested contract path', async () => {
    const root = installOpfs();
    await opfsBackend().write('acct1/messages/m1.enc', bytes(1, 2, 3));

    // The contract layout is /{accountId}/messages/{stableId}.enc, so a write
    // to a fresh profile has to create two levels before the file exists.
    expect(root.descend('acct1', 'messages')?.files.get('m1.enc')).toEqual(bytes(1, 2, 3));
    expect(root.calls).toContain('dir:acct1:create');
    expect(root.descend('acct1')?.calls).toContain('dir:messages:create');
  });

  it('writes a top-level file with no directory walk at all', async () => {
    const root = installOpfs();
    await opfsBackend().write('searchslice.enc', bytes(9));
    expect(root.files.get('searchslice.enc')).toEqual(bytes(9));
    expect(root.calls).toEqual(['file:searchslice.enc:create']);
  });

  it('overwrites an existing blob rather than appending to it', async () => {
    const root = installOpfs();
    const backend = opfsBackend();
    await backend.write('acct1/headers/inbox.enc', bytes(1, 1, 1, 1));
    await backend.write('acct1/headers/inbox.enc', bytes(2, 2));
    // A stale tail would decrypt-fail on read; AES-GCM has no framing to
    // survive a partial overwrite.
    expect(root.descend('acct1', 'headers')?.files.get('inbox.enc')).toEqual(bytes(2, 2));
  });

  it('rejects a path with no filename at all', async () => {
    installOpfs();
    const backend = opfsBackend();
    for (const path of ['', '/', '///']) {
      await expect(backend.write(path, bytes(1))).rejects.toThrow(/empty OPFS path/);
    }
  });

  it('treats a trailing slash as naming the file, not the directory', async () => {
    // `resolveDir` drops empty segments and pops the LAST one as the filename,
    // so `acct1/messages/` writes a FILE called `messages`. Not reachable in
    // practice — every path comes from `opfsMessagePath`/`opfsHeadersPath`/
    // `opfsSearchSlicePath`, which all end in `.enc` — but worth pinning so the
    // degradation is known rather than assumed.
    const root = installOpfs();
    await opfsBackend().write('acct1/messages/', bytes(1));
    expect(root.descend('acct1')?.files.has('messages')).toBe(true);
    expect(root.descend('acct1', 'messages')).toBeUndefined();
  });
});

describe('opfsBackend — reading', () => {
  it('round-trips the bytes it wrote', async () => {
    installOpfs();
    const backend = opfsBackend();
    await backend.write('acct1/messages/m1.enc', bytes(7, 8, 9));
    expect(await backend.read('acct1/messages/m1.enc')).toEqual(bytes(7, 8, 9));
  });

  it('returns null for a missing FILE rather than throwing', async () => {
    // This is the load-bearing contract: callers treat null as "not cached" and
    // fall through to the network. A throw here would surface as a hard error
    // on a perfectly normal cold cache.
    installOpfs();
    const backend = opfsBackend();
    await backend.write('acct1/messages/m1.enc', bytes(1));
    expect(await backend.read('acct1/messages/absent.enc')).toBeNull();
  });

  it('returns null for a missing DIRECTORY too', async () => {
    // A profile that has never been cached has no `/{accountId}` directory at
    // all, so the walk fails one level earlier than a missing file.
    installOpfs();
    expect(await opfsBackend().read('never-seen/messages/m1.enc')).toBeNull();
  });

  it('never creates anything on a read miss', async () => {
    // A read that created directories would turn a cold-cache probe into a
    // write, and `create: false` is what stops it.
    const root = installOpfs();
    await opfsBackend().read('acct1/messages/m1.enc');
    expect(root.dirs.size).toBe(0);
    expect(root.calls).toEqual(['dir:acct1:open']);
  });
});

describe('opfsBackend — removing', () => {
  it('deletes an existing blob', async () => {
    const root = installOpfs();
    const backend = opfsBackend();
    await backend.write('acct1/messages/m1.enc', bytes(1));
    await backend.remove('acct1/messages/m1.enc');
    expect(root.descend('acct1', 'messages')?.files.has('m1.enc')).toBe(false);
    expect(await backend.read('acct1/messages/m1.enc')).toBeNull();
  });

  it('is idempotent — removing something absent is not an error', async () => {
    // Cache eviction and logout both remove blindly; a throw would abort the
    // rest of the sweep partway through.
    installOpfs();
    const backend = opfsBackend();
    await expect(backend.remove('acct1/messages/absent.enc')).resolves.toBeUndefined();
    await expect(backend.remove('never-seen/messages/m1.enc')).resolves.toBeUndefined();
  });

  it('does not create the directory chain while removing', async () => {
    const root = installOpfs();
    await opfsBackend().remove('never-seen/messages/m1.enc');
    expect(root.dirs.size).toBe(0);
  });
});

describe('opfsBackend — one directory handle for the backend’s life', () => {
  it('calls getDirectory once however many operations run', async () => {
    const root = new FakeDir();
    const getDirectory = vi.fn(async () => root);
    vi.stubGlobal('navigator', { ...globalThis.navigator, storage: { getDirectory } });

    const backend = opfsBackend();
    await backend.write('a/b.enc', bytes(1));
    await backend.read('a/b.enc');
    await backend.remove('a/b.enc');
    // The root promise is captured at construction, so the OPFS root is
    // resolved once rather than per call.
    expect(getDirectory).toHaveBeenCalledTimes(1);
  });
});

describe('memoryBackend and opfsBackend agree', () => {
  // The in-memory backend is both the unit-test fake and the runtime fallback
  // when OPFS is absent, so a behavioural difference between the two would mean
  // the tests pass against something the app never runs.
  it('behave identically for write / read / miss / remove / re-read', async () => {
    installOpfs();
    for (const backend of [memoryBackend(), opfsBackend()]) {
      expect(await backend.read('acct1/messages/m1.enc')).toBeNull();
      await backend.write('acct1/messages/m1.enc', bytes(4, 5, 6));
      expect(await backend.read('acct1/messages/m1.enc')).toEqual(bytes(4, 5, 6));
      await backend.remove('acct1/messages/m1.enc');
      expect(await backend.read('acct1/messages/m1.enc')).toBeNull();
      await expect(backend.remove('acct1/messages/m1.enc')).resolves.toBeUndefined();
    }
  });
});
