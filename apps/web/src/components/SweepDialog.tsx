import { createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import type { SweepStrategy } from '../state/slices/mail.ts';

// Outlook-style "sweep" (plan §1.5): bulk-clean a sender's mail with a preview
// of exactly what will move to Trash before you commit, and (like every list
// mutation) an undo. The dialog is opened for a specific sender address.

const STRATEGIES: { value: SweepStrategy; label: string }[] = [
  { value: 'all', label: 'Delete all from this sender' },
  { value: 'keep-latest', label: 'Keep the latest, delete the rest' },
  { value: 'older-than', label: 'Delete older than N days' },
  { value: 'block', label: 'Delete all and block this sender' },
];

export function SweepDialog(props: { fromEmail: string; onClose: () => void }): JSX.Element {
  const app = useApp();
  const [strategy, setStrategy] = createSignal<SweepStrategy>('all');
  const [days, setDays] = createSignal(30);
  const [busy, setBusy] = createSignal(false);

  const preview = createMemo(() => app.sweepPreview(props.fromEmail, strategy(), days()));

  async function run(): Promise<void> {
    setBusy(true);
    try {
      await app.executeSweep(props.fromEmail, strategy(), days());
      props.onClose();
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class="sweep__backdrop" role="dialog" aria-modal="true" aria-label="Sweep messages">
      <div class="sweep">
        <header class="sweep__header">
          <h2>Sweep {props.fromEmail}</h2>
          <button type="button" class="btn btn--ghost" aria-label="Close" onClick={() => props.onClose()}>
            ✕
          </button>
        </header>

        <fieldset class="sweep__options">
          <For each={STRATEGIES}>
            {(s) => (
              <label class="sweep__option">
                <input
                  type="radio"
                  name="sweep-strategy"
                  value={s.value}
                  checked={strategy() === s.value}
                  onChange={() => setStrategy(s.value)}
                />
                {s.label}
              </label>
            )}
          </For>
          <Show when={strategy() === 'older-than'}>
            <label class="sweep__days field">
              <span>Days</span>
              <input
                type="number"
                min="1"
                value={days()}
                onInput={(e) => setDays(Math.max(1, Number(e.currentTarget.value) || 1))}
              />
            </label>
          </Show>
        </fieldset>

        <p class="sweep__count" aria-live="polite">
          {preview().length} message{preview().length === 1 ? '' : 's'} will move to Trash
        </p>
        <ul class="sweep__preview">
          <For each={preview().slice(0, 20)}>
            {(m) => (
              <li class="sweep__row">
                <span class="sweep__subject">{m.subject ?? '(no subject)'}</span>
                <span class="sweep__date">{m.receivedAt.slice(0, 10)}</span>
              </li>
            )}
          </For>
        </ul>

        <footer class="sweep__footer">
          <button type="button" class="btn btn--ghost" onClick={() => props.onClose()}>
            Cancel
          </button>
          <button
            type="button"
            class="btn btn--primary"
            disabled={busy() || preview().length === 0}
            onClick={() => void run()}
          >
            {busy() ? 'Sweeping…' : `Sweep ${preview().length}`}
          </button>
        </footer>
      </div>
    </div>
  );
}
