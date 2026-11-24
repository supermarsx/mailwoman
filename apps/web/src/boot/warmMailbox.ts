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
import { pageManifest } from './bootManifest.ts';
import { prefetchFile } from './chunkRetry.ts';

let done = false;

function saveData(): boolean {
  const connection = (navigator as Navigator & { connection?: { saveData?: boolean; effectiveType?: string } }).connection;
  return connection?.saveData === true || connection?.effectiveType === 'slow-2g' || connection?.effectiveType === '2g';
}

/** Download the mailbox's files ahead (once per page). Resolves when they are done. */
export async function warmMailbox(doc: Document = document): Promise<void> {
  if (done) return;
  done = true;
  const policy = await ensureWebPerformance().catch(() => null);
  const manifest = pageManifest(doc);
  if (policy === null || !policy.prefetchEnabled || saveData() || manifest === null) return;
  await Promise.all(manifest.mail.filter((href) => href.startsWith('./assets/')).map((href) => prefetchFile(new URL(href, doc.baseURI).href)));
}

/** Tests only. */
export function resetWarmMailbox(): void { done = false; }
