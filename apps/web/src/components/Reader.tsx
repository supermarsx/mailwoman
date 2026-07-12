import { createMemo, createResource, createSignal, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { SweepDialog } from './SweepDialog.tsx';
import { ThumbnailStrip, type StripItem } from '../viewers/ThumbnailStrip.tsx';
import { AttachmentViewer } from '../viewers/AttachmentViewer.tsx';
import { buildDownloadUrl, fetchObjectUrl, type AttachmentPart } from '../viewers/attachments.ts';
import type { Email, EmailAddress } from '../api/jmap-types.ts';

function addressList(addrs: EmailAddress[] | null): string {
  if (addrs === null || addrs.length === 0) return '';
  return addrs.map((a) => (a.name && a.name.length > 0 ? `${a.name} <${a.email}>` : a.email)).join(', ');
}

function ReaderToolbar(props: { email: Email }): JSX.Element {
  const app = useApp();
  const [sweeping, setSweeping] = createSignal(false);
  const id = () => props.email.id;
  const sender = () => props.email.from?.[0]?.email ?? '';
  const pinned = () => props.email.pinned === true;

  return (
    <div class="reader__toolbar" role="toolbar" aria-label="Message actions">
      <button
        type="button"
        class="btn btn--ghost"
        aria-pressed={pinned()}
        onClick={() => void app.pinMessage(id(), !pinned())}
      >
        {pinned() ? 'Unpin' : 'Pin'}
      </button>
      <button type="button" class="btn btn--ghost" onClick={() => void app.archiveMessage(id())}>
        Archive
      </button>
      <button type="button" class="btn btn--ghost" onClick={() => void app.trashMessage(id())}>
        Delete
      </button>
      <button type="button" class="btn btn--ghost" onClick={() => void app.markSpam(id())}>
        Spam
      </button>
      <button
        type="button"
        class="btn btn--ghost"
        data-testid="reader-export"
        onClick={() => void app.exportMessage()}
      >
        Export
      </button>
      <Show when={sender() !== ''}>
        <button type="button" class="btn btn--ghost" onClick={() => setSweeping(true)}>
          Sweep sender
        </button>
      </Show>
      <Show when={sweeping()}>
        <SweepDialog fromEmail={sender()} onClose={() => setSweeping(false)} />
      </Show>
    </div>
  );
}

/** Attachment thumbnails + the on-click viewer, rendered AROUND (never inside)
 *  the sandboxed message iframe. Each viewer keeps its own sandbox (§2.4). */
function AttachmentsPane(props: { email: Email }): JSX.Element {
  const app = useApp();
  const [openItem, setOpenItem] = createSignal<StripItem | null>(null);

  const items = createMemo<StripItem[]>(() => {
    const parts = (props.email as { attachments?: AttachmentPart[] }).attachments ?? [];
    return parts
      .filter((a) => a.blobId !== null && a.blobId !== undefined && a.blobId !== '')
      .map((a) => ({
        blobId: a.blobId as string,
        name: a.name !== null && a.name !== undefined && a.name.length > 0 ? a.name : '(unnamed)',
        mime: a.type.length > 0 ? a.type : 'application/octet-stream',
        size: a.size,
      }));
  });

  function blobUrlFor(item: StripItem): Promise<string> {
    const url = app.downloadUrl();
    const acct = app.accountId();
    if (url === null || acct === null) return Promise.resolve('');
    return fetchObjectUrl(
      buildDownloadUrl(url, { accountId: acct, blobId: item.blobId, name: item.name, mime: item.mime }),
    );
  }

  const [blobUrl] = createResource(openItem, (item) => blobUrlFor(item));

  return (
    <Show when={items().length > 0}>
      <section class="reader__attachments" aria-label="Attachments" data-testid="reader-attachments">
        <ThumbnailStrip
          items={items()}
          selectedBlobId={openItem()?.blobId ?? ''}
          onSelect={(it) => setOpenItem(it)}
          resolveThumb={blobUrlFor}
        />
        <Show when={openItem()}>
          {(item) => (
            <div
              class="attachment-modal"
              role="dialog"
              aria-modal="true"
              aria-label={`Attachment ${item().name}`}
              data-testid="attachment-viewer"
            >
              <div class="attachment-modal__bar">
                <span class="attachment-modal__name">{item().name}</span>
                <button
                  type="button"
                  class="btn btn--ghost"
                  aria-label="Close attachment"
                  onClick={() => setOpenItem(null)}
                >
                  ✕
                </button>
              </div>
              <Show when={blobUrl()} fallback={<p class="attachment-modal__loading">Loading attachment…</p>}>
                {(url) => (
                  <AttachmentViewer
                    part={{ partId: null, blobId: item().blobId, size: item().size, type: item().mime }}
                    blobUrl={url()}
                    mime={item().mime}
                    name={item().name}
                  />
                )}
              </Show>
            </div>
          )}
        </Show>
      </section>
    </Show>
  );
}

export function Reader(): JSX.Element {
  const app = useApp();

  return (
    <section class="reader" aria-label="Message">
      <Show when={app.openEmail()} fallback={<p class="reader__empty">Select a message to read</p>}>
        {(email) => (
          <>
            <header class="reader__header">
              <button type="button" class="btn btn--ghost reader__close" onClick={() => app.closeMessage()}>
                ← Back
              </button>
              <h2 class="reader__subject">{email().subject ?? '(no subject)'}</h2>
              <div class="reader__meta">
                <span>From: {addressList(email().from)}</span>
                <span>To: {addressList(email().to)}</span>
              </div>
              <ReaderToolbar email={email()} />
            </header>
            <AttachmentsPane email={email()} />
            <Show
              when={app.sanitizedHtml() !== null}
              fallback={<p class="reader__empty">{app.readLoading() ? 'Sanitizing…' : 'No content'}</p>}
            >
              {/*
                Security-critical (SPEC §7.2): sanitized HTML is rendered in a
                sandboxed iframe. The sandbox attribute intentionally omits
                allow-scripts AND allow-same-origin, so even if the sanitizer
                somehow missed something, scripts cannot run and the frame has
                an opaque origin with no access to the parent.
              */}
              <iframe
                class="reader__frame"
                title="Message body"
                sandbox=""
                srcdoc={app.sanitizedHtml() ?? ''}
              />
            </Show>
          </>
        )}
      </Show>
    </section>
  );
}
