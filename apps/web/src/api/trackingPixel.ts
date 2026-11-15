import {parseRecipients,type DraftInput} from './jmapCompose.ts';
import type {TrackingToken} from './tracking.ts';
export class TrackingUnavailable extends Error {
  constructor(readonly reason: string) {
    super(`open tracking unavailable: ${reason}`);
    this.name = 'TrackingUnavailable';
  }
}

export function trackingPixel(url: string): string {
  const safe = url.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  return `<img src="${safe}" width="1" height="1" alt="">`;
}

/** The draft with the image appended and the token to bind once it is sent.
 *  The token is minted for the draft's subject and every recipient address
 *  (To, Cc and Bcc: the sender's own record of who was sent the message). */
export async function withTrackingPixel(
  draft: DraftInput,
  mint: (subject: string, to: string[]) => Promise<TrackingToken>,
): Promise<{ draft: DraftInput; token: string }> {
  const to = [draft.to, draft.cc ?? '', draft.bcc ?? ''].flatMap((f) => parseRecipients(f).map((r) => r.email));
  const minted = await mint(draft.subject, to);
  const pixel = trackingPixel(minted.url);
  // Complete composer documents keep the pixel inside the body. Legacy
  // fragments remain fragments; avoid reparsing or altering authored markup.
  const closingBody = /<\/body>\s*<\/html>\s*$/i;
  const htmlBody = closingBody.test(draft.htmlBody)
    ? draft.htmlBody.replace(closingBody, closing => pixel + closing)
    : draft.htmlBody + pixel;
  return { draft: { ...draft, htmlBody }, token: minted.token };
}
