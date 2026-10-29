// An `.ics` / `text/calendar` attachment opened in the attachment preview or
// its tab (t53): the same event card the reader shows, for that one part.
// The server reads the part from the message by its blob id (`<emailId>.<n>`);
// the bytes the preview already fetched are not parsed here. When the server
// cannot read the part as a calendar (or the account has no calendar surface),
// the raw text is shown in the sandboxed text viewer instead.

import { lazy, Show, Suspense, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { EventCard } from '../components/event-card/EventCard.tsx';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';

const TextViewer = lazy(() => import('./TextViewer.tsx'));

/** The message id and part of an engine blob id (`<emailId>.<n>`), if it is one. */
export function messagePart(blobId: string | null | undefined): { emailId: string; blobId: string } | null {
  const m = /^([^.\s]+)\.(\d+)$/.exec(blobId ?? '');
  return m && !m[1]!.startsWith('U') ? { emailId: m[1]!, blobId: blobId! } : null;
}

export function CalendarAttachmentViewer(props: ViewerProps): JSX.Element {
  const app = useApp();
  const part = (): { emailId: string; blobId: string } | null => messagePart(props.part.blobId);
  const raw = (): JSX.Element => (
    <Suspense fallback={<p class="mw-viewer__loading">{t('common-viewer-loading')}</p>}>
      <TextViewer {...props} />
    </Suspense>
  );
  return (
    <Show when={part() && app.accountId()} fallback={raw()}>
      <EventCard accountId={app.accountId()!} emailId={part()!.emailId} blobId={part()!.blobId} fallback={raw()} />
    </Show>
  );
}

export default CalendarAttachmentViewer;
