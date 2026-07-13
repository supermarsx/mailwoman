import { createMemo, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import {
  createContactAutocomplete,
  type ContactSuggestion,
} from '../modules/contacts/autocomplete.ts';

// Compose (plan §1.5, §2.1): grown with an identity/signature picker (multiple
// from-addresses, server-pulled allowed-froms) and send-later. The core To /
// Subject / Body fields + the Send button keep their exact labels so the mock +
// engine e2e specs still drive it. e10 adds contacts recipient autocomplete to
// the To field — a surgical addition over e7's `createContactAutocomplete`.

/** The recipient token currently being typed: the text after the last separator. */
function tokenBoundary(value: string): number {
  return Math.max(value.lastIndexOf(','), value.lastIndexOf(';'));
}

export function Compose(props: { onClose: () => void }): JSX.Element {
  const app = useApp();
  const [to, setTo] = createSignal('');
  const [subject, setSubject] = createSignal('');
  const [body, setBody] = createSignal('');
  const [identityId, setIdentityId] = createSignal<string>('');
  const [sendAt, setSendAt] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [acOpen, setAcOpen] = createSignal(false);

  // Recipient autocomplete over the loaded contacts (plan §2.2 / e7 seam). The
  // ranking is client-side over `app.contacts()`; we load contacts on open so a
  // fresh session can still complete. Load failures are non-fatal (empty list).
  const contactAc = createContactAutocomplete(() => app.contacts());

  onMount(() => {
    void app.loadIdentities();
    void app.loadContacts().catch(() => undefined);
  });

  const identity = createMemo(() => app.identities().find((i) => i.id === identityId()) ?? null);

  function onToInput(value: string): void {
    setTo(value);
    const token = value.slice(tokenBoundary(value) + 1).trim();
    contactAc.setQuery(token);
    setAcOpen(token.length > 0);
  }

  /** Replace the in-progress recipient token with the picked contact. */
  function pickSuggestion(s: ContactSuggestion): void {
    const value = to();
    const cut = tokenBoundary(value);
    const head = cut >= 0 ? `${value.slice(0, cut + 1)} ` : '';
    setTo(`${head}${s.display}, `);
    contactAc.reset();
    setAcOpen(false);
  }

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await app.sendMessage({
        to: to(),
        subject: subject(),
        htmlBody: `<p>${escapeHtml(body()).replace(/\n/g, '<br>')}</p>`,
        identity: identity(),
        // datetime-local yields a local wall-clock string; convert to a UTC ISO.
        sendAt: sendAt() !== '' ? new Date(sendAt()).toISOString() : null,
      });
      props.onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Send failed');
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class="compose__backdrop" role="dialog" aria-modal="true" aria-label="Compose message">
      <form class="compose" onSubmit={(e) => void onSubmit(e)}>
        <header class="compose__header">
          <h2>New message</h2>
          <button type="button" class="btn btn--ghost" onClick={() => props.onClose()}>
            ✕
          </button>
        </header>

        <Show when={app.identities().length > 0}>
          <label class="field">
            <span>From</span>
            <select value={identityId()} onChange={(e) => setIdentityId(e.currentTarget.value)}>
              <option value="">Default</option>
              <For each={app.identities()}>
                {(id) => (
                  <option value={id.id}>
                    {id.name} &lt;{id.email}&gt;
                  </option>
                )}
              </For>
            </select>
          </label>
        </Show>

        <label class="field compose__to">
          <span>To</span>
          <input
            type="text"
            required
            placeholder="someone@example.org"
            autocomplete="off"
            value={to()}
            onInput={(e) => onToInput(e.currentTarget.value)}
            onBlur={() => setAcOpen(false)}
          />
          <Show when={acOpen() && contactAc.suggestions().length > 0}>
            <ul class="compose__ac" role="listbox" aria-label="Contact suggestions">
              <For each={contactAc.suggestions()}>
                {(s) => (
                  <li>
                    <button
                      type="button"
                      role="option"
                      aria-selected={false}
                      class="compose__ac-item"
                      data-testid="contact-suggestion"
                      // mousedown (not click) so the pick lands before the input's blur.
                      onMouseDown={(e) => {
                        e.preventDefault();
                        pickSuggestion(s);
                      }}
                    >
                      <span class="compose__ac-name">{s.name.length > 0 ? s.name : s.email}</span>
                      <Show when={s.name.length > 0}>
                        <span class="compose__ac-email">{s.email}</span>
                      </Show>
                    </button>
                  </li>
                )}
              </For>
            </ul>
          </Show>
        </label>
        <label class="field">
          <span>Subject</span>
          <input type="text" value={subject()} onInput={(e) => setSubject(e.currentTarget.value)} />
        </label>
        <label class="field field--grow">
          <span>Body</span>
          <textarea rows="10" value={body()} onInput={(e) => setBody(e.currentTarget.value)} />
        </label>

        <Show when={identity()?.signatureText}>
          {(sig) => <p class="compose__signature">— {sig()}</p>}
        </Show>

        <label class="field">
          <span>Send later</span>
          <input
            type="datetime-local"
            value={sendAt()}
            onInput={(e) => setSendAt(e.currentTarget.value)}
          />
        </label>

        <Show when={error()}>
          <p class="login__error" role="alert">
            {error()}
          </p>
        </Show>
        <footer class="compose__footer">
          <button type="button" class="btn btn--ghost" onClick={() => props.onClose()}>
            Cancel
          </button>
          <button type="submit" class="btn btn--primary" disabled={busy()}>
            {busy() ? 'Sending…' : sendAt() !== '' ? 'Schedule' : 'Send'}
          </button>
        </footer>
      </form>
    </div>
  );
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
