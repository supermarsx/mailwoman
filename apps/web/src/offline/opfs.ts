// OPFS encrypted message/PIM cache (contract §2.5 layout:
//   /{accountId}/messages/{stableId}.enc
//   /{accountId}/headers/{mailboxId}.enc
//   /{accountId}/searchslice.enc
// ). Every blob is AES-256-GCM (12-byte IV prefix) under the profile key.
//
// The filesystem is behind a `BlobBackend` interface so unit tests exercise the
// real crypto round-trip against an in-memory store, and so the OPFS specifics
// (which jsdom lacks) stay isolated.

import { opfsHeadersPath, opfsMessagePath, opfsSearchSlicePath } from '../contracts/offline.ts';
import type { Email, Id } from '../api/jmap-types.ts';
import { decryptJson, encryptJson } from './crypto.ts';

export interface BlobBackend {
  read(path: string): Promise<Uint8Array | null>;
  write(path: string, data: Uint8Array): Promise<void>;
  remove(path: string): Promise<void>;
}

/** The encrypted cache surface consumed by the offline slice / cached reads. */
export class EncryptedCache {
  constructor(
    private readonly backend: BlobBackend,
    private readonly key: CryptoKey,
  ) {}

  private async readJson<T>(path: string): Promise<T | null> {
    const blob = await this.backend.read(path);
    if (blob === null) return null;
    return decryptJson<T>(this.key, blob);
  }

  private async writeJson(path: string, value: unknown): Promise<void> {
    await this.backend.write(path, await encryptJson(this.key, value));
  }

  putMessage(accountId: Id, stableId: Id, email: Email): Promise<void> {
    return this.writeJson(opfsMessagePath(accountId, stableId), email);
  }
  getMessage(accountId: Id, stableId: Id): Promise<Email | null> {
    return this.readJson<Email>(opfsMessagePath(accountId, stableId));
  }

  putHeaders(accountId: Id, mailboxId: Id, headers: Email[]): Promise<void> {
    return this.writeJson(opfsHeadersPath(accountId, mailboxId), headers);
  }
  getHeaders(accountId: Id, mailboxId: Id): Promise<Email[] | null> {
    return this.readJson<Email[]>(opfsHeadersPath(accountId, mailboxId));
  }

  putSearchSlice(accountId: Id, headers: Email[]): Promise<void> {
    return this.writeJson(opfsSearchSlicePath(accountId), headers);
  }
  getSearchSlice(accountId: Id): Promise<Email[] | null> {
    return this.readJson<Email[]>(opfsSearchSlicePath(accountId));
  }
}

/** Is the Origin Private File System available? (Absent under jsdom.) */
export function opfsAvailable(): boolean {
  return (
    typeof navigator !== 'undefined' &&
    typeof navigator.storage !== 'undefined' &&
    typeof navigator.storage.getDirectory === 'function'
  );
}

async function resolveDir(
  root: FileSystemDirectoryHandle,
  path: string,
  create: boolean,
): Promise<{ dir: FileSystemDirectoryHandle; file: string }> {
  const parts = path.split('/').filter((p) => p.length > 0);
  const file = parts.pop();
  if (file === undefined) throw new Error(`offline: empty OPFS path "${path}"`);
  let dir = root;
  for (const part of parts) {
    dir = await dir.getDirectoryHandle(part, { create });
  }
  return { dir, file };
}

/** The real OPFS-backed store (browser only). */
export function opfsBackend(): BlobBackend {
  const rootP = navigator.storage.getDirectory();
  return {
    async read(path) {
      try {
        const root = await rootP;
        const { dir, file } = await resolveDir(root, path, false);
        const handle = await dir.getFileHandle(file, { create: false });
        const bytes = await (await handle.getFile()).arrayBuffer();
        return new Uint8Array(bytes);
      } catch {
        // NotFoundError (missing dir/file) reads back as "not cached".
        return null;
      }
    },
    async write(path, data) {
      const root = await rootP;
      const { dir, file } = await resolveDir(root, path, true);
      const handle = await dir.getFileHandle(file, { create: true });
      const writable = await handle.createWritable();
      // `data` is byte-backed at runtime; cast satisfies FileSystemWriteChunkType
      // (Uint8Array now defaults its buffer type to ArrayBufferLike).
      await writable.write(data as BufferSource);
      await writable.close();
    },
    async remove(path) {
      try {
        const root = await rootP;
        const { dir, file } = await resolveDir(root, path, false);
        await dir.removeEntry(file);
      } catch {
        // Already absent — nothing to remove.
      }
    },
  };
}

/** In-memory backend: the unit-test fake and the fallback when OPFS is absent. */
export function memoryBackend(): BlobBackend {
  const files = new Map<string, Uint8Array>();
  return {
    async read(path) {
      return files.get(path) ?? null;
    },
    async write(path, data) {
      files.set(path, data);
    },
    async remove(path) {
      files.delete(path);
    },
  };
}
