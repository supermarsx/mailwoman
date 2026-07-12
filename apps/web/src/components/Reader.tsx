import { Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import type { EmailAddress } from '../api/jmap-types.ts';

function addressList(addrs: EmailAddress[] | null): string {
  if (addrs === null || addrs.length === 0) return '';
  return addrs.map((a) => (a.name && a.name.length > 0 ? `${a.name} <${a.email}>` : a.email)).join(', ');
}

export function Reader(): JSX.Element {
  const app = useApp();

  return (
    <section class="reader" aria-label="Message">
      <Show
        when={app.openEmail()}
        fallback={<p class="reader__empty">Select a message to read</p>}
      >
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
