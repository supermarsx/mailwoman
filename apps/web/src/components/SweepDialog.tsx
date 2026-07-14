import { createMemo, createSignal, For, Show, onMount, onCleanup, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, isolate } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import type { SweepStrategy } from '../state/slices/mail.ts';

// Outlook-style "sweep" (plan §1.5): bulk-clean a sender's mail with a preview
// of exactly what will move to Trash before you commit, and (like every list
// mutation) an undo. The dialog is opened for a specific sender address.

const STRATEGIES: { value: SweepStrategy; labelId: string }[] = [
  { value: 'all', labelId: 'mail-sweep-all' },
  { value: 'keep-latest', labelId: 'mail-sweep-keep-latest' },
  { value: 'older-than', labelId: 'mail-sweep-older-than' },
  { value: 'block', labelId: 'mail-sweep-block' },
];

export function SweepDialog(props: { fromEmail: string; onClose: () => void }): JSX.Element {
  const app = useApp();
  const [strategy, setStrategy] = createSignal<SweepStrategy>('all');
  const [days, setDays] = createSignal(30);
  const [busy, setBusy] = createSignal(false);

  const preview = createMemo(() => app.sweepPreview(props.fromEmail, strategy(), days()));

  // Dialog focus management (self-contained per t8-e1): move focus in on open,
  // restore it on close, Escape closes.
  let dialogEl: HTMLDivElement | undefined;
  let previouslyFocused: HTMLElement | null = null;
  onMount(() => {
    previouslyFocused = document.activeElement as HTMLElement | null;
    dialogEl?.querySelector<HTMLElement>('input, button')?.focus();
  });
  onCleanup(() => previouslyFocused?.focus());

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
    <div
      class="sweep__backdrop"
      role="dialog"
      aria-modal="true"
      aria-label={t('mail-sweep-label')}
      ref={dialogEl}
      onKeyDown={(e) => {
        if (e.key === 'Escape') {
          e.preventDefault();
          props.onClose();
        }
      }}
    >
      <div class="sweep">
        <header class="sweep__header">
          <h2>{t('mail-sweep-title', { sender: isolate(props.fromEmail) })}</h2>
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-label={t('mail-compose-close')} onClick={() => props.onClose()}>
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
                {t(s.labelId)}
              </label>
            )}
          </For>
          <Show when={strategy() === 'older-than'}>
            <label class="sweep__days field">
              <span>{t('mail-sweep-days')}</span>
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
          {t('mail-sweep-count', { count: preview().length })}
        </p>
        <ul class="sweep__preview">
          <For each={preview().slice(0, 20)}>
            {(m) => (
              <li class="sweep__row">
                <span class="sweep__subject">{m.subject ?? t('mail-no-subject')}</span>
                <span class="sweep__date">{m.receivedAt.slice(0, 10)}</span>
              </li>
            )}
          </For>
        </ul>

        <footer class="sweep__footer">
          <button type="button" class={`btn btn--ghost ${a11y.focusable}`} onClick={() => props.onClose()}>
            {t('mail-sweep-cancel')}
          </button>
          <button
            type="button"
            class={`btn btn--primary ${a11y.focusable}`}
            disabled={busy() || preview().length === 0}
            onClick={() => void run()}
          >
            {busy() ? t('mail-sweeping') : t('mail-sweep-run', { count: preview().length })}
          </button>
        </footer>
      </div>
    </div>
  );
}
