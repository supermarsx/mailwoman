import { createSignal, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';

export function Compose(props: { onClose: () => void }): JSX.Element {
  const app = useApp();
  const [to, setTo] = createSignal('');
  const [subject, setSubject] = createSignal('');
  const [body, setBody] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await app.sendMessage({
        to: to(),
        subject: subject(),
        htmlBody: `<p>${escapeHtml(body()).replace(/\n/g, '<br>')}</p>`,
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
        <label class="field">
          <span>To</span>
          <input
            type="text"
            required
            placeholder="someone@example.org"
            value={to()}
            onInput={(e) => setTo(e.currentTarget.value)}
          />
        </label>
        <label class="field">
          <span>Subject</span>
          <input type="text" value={subject()} onInput={(e) => setSubject(e.currentTarget.value)} />
        </label>
        <label class="field field--grow">
          <span>Body</span>
          <textarea rows="10" value={body()} onInput={(e) => setBody(e.currentTarget.value)} />
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
            {busy() ? 'Sending…' : 'Send'}
          </button>
        </footer>
      </form>
    </div>
  );
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
