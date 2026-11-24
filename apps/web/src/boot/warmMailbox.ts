// t52w: on the sign-in screen, the mailbox's files are fetched once the user
// starts on the email field, so signing in does not then wait for them.
//
// Downloads only, with the chunk retry (chunkRetry.ts `prefetchFile`: the
// import waits for them later), from the build manifest in the shell
// (bootManifest.ts): nothing is executed, and the sign-in chunk names no
// mailbox file, so a change to the mailbox does not rename the sign-in chunk.
// Gated like every speculative fetch: the server's `web.prefetch.enabled`, and
// never when the browser asks to save data or is on a 2G connection.

import { ensureWebPerformance } from './performanceBootstrap.ts';
import { BOOT_MANIFEST_ID, type BootManifest } from './bootManifest.ts';
import { prefetchFile } from './chunkRetry.ts';

let done = false;

/** Read the manifest the shell carries; `null` outside a production page. */
export function bootManifest(doc: Document = document): BootManifest | null {
  try {
    const value: unknown = JSON.parse(doc.getElementById(BOOT_MANIFEST_ID)?.textContent ?? 'null');
    return value !== null && typeof value === 'object' && (value as { v?: unknown }).v === 1 ? (value as BootManifest) : null;
  } catch {
    return null;
  }
}

function saveData(): boolean {
  const connection = (navigator as Navigator & { connection?: { saveData?: boolean; effectiveType?: string } }).connection;
  return connection?.saveData === true || connection?.effectiveType === 'slow-2g' || connection?.effectiveType === '2g';
}

/** Download the mailbox's files ahead (once per page). Resolves when they are done. */
export async function warmMailbox(doc: Document = document): Promise<void> {
  if (done) return;
  done = true;
  const policy = await ensureWebPerformance().catch(() => null);
  const manifest = bootManifest(doc);
  if (policy === null || !policy.prefetchEnabled || saveData() || manifest === null) return;
  await Promise.all(manifest.mail.filter((href) => href.startsWith('./assets/')).map((href) => prefetchFile(new URL(href, doc.baseURI).href)));
}

/** Tests only. */
export function resetWarmMailbox(): void { done = false; }
