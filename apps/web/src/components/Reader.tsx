import { createSignal, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { SweepDialog } from './SweepDialog.tsx';
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
