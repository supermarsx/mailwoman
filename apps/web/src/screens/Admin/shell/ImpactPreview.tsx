// Impact preview before a risky write (t32-cfg UX6).
//
// The screen sends the write with `"dryRun": true` (`loadImpact`); the server
// answers `{impact: {...}}` without writing. This dialog lists each consequence
// with its count and the first 20 names ("Users refused: 3 — a@x, b@x, c@x"),
// then asks for confirmation; a destructive write also needs the target's name
// typed (`typedConfirmation`). Nothing is written until Confirm.
//
// States: checking (Confirm disabled), the list, "nothing changes", a failed check
// with the reason and Try again, and "Not available" when the server returned no
// preview (it is then said plainly, and the admin may still confirm).
//
// A modal `alertdialog` like ConfirmDialog: focus moves in, Tab stays inside,
// Escape cancels, focus returns to the opener.

import { For, Match, Show, Switch, createResource, createSignal, createUniqueId, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { formatNumber, isolate, t } from '../../../i18n';
import { createFocusTrap } from '../../../components/a11y';
import type { ImpactItem } from './adminApi.ts';
import { Feedback, describeAdminError } from './Feedback.tsx';
import * as css from './shell.css.ts';
import * as ux from './ux.css.ts';

/** How many names are listed per consequence. */
export const IMPACT_NAMES_SHOWN = 20;

export interface ImpactPreviewProps {
  open: boolean;
  /** Names the object: "Switch example.org to listed mode?" */
  title: string;
  /** What will happen and what will not. */
  body?: string;
  /** The dry run. Resolves null when the server returned no preview. */
  loadImpact: () => Promise<Record<string, ImpactItem> | null>;
  /** The label of each consequence key, given its count ("Users refused"). */
  labels: Record<string, (count: number) => string>;
  confirmLabel: string;
  /** Red confirm button (default true). */
  danger?: boolean;
  /** When set, Confirm works only after this exact text is typed. */
  typedConfirmation?: string;
  /** May return a promise; a rejection keeps the dialog open with the error. */
  onConfirm: () => void | Promise<void>;
  onCancel: () => void;
}

export function ImpactPreview(props: ImpactPreviewProps): JSX.Element {
  return (
    <Show when={props.open}>
      <ImpactPreviewOpen {...props} />
    </Show>
  );
}

function ImpactPreviewOpen(props: ImpactPreviewProps): JSX.Element {
  const id = createUniqueId();
  const [el, setEl] = createSignal<HTMLElement>();
  const [typed, setTyped] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [impact, { refetch }] = createResource(() => props.loadImpact());
  let typedInput: HTMLInputElement | undefined;
  let cancelButton: HTMLButtonElement | undefined;

  // Read the value only once the dry run has answered: an errored resource throws when read.
  const value = (): Record<string, ImpactItem> | null | undefined => (impact.state === 'ready' ? impact() : undefined);
  const entries = (): [string, ImpactItem][] => Object.entries(value() ?? {});
  const checked = (): boolean => impact.state === 'ready';
  const matches = (): boolean => props.typedConfirmation === undefined || typed() === props.typedConfirmation;

  function cancel(): void {
    if (!busy()) props.onCancel();
  }

  createFocusTrap(el, { onEscape: cancel, initialFocus: () => typedInput ?? cancelButton });

  async function confirm(e: Event): Promise<void> {
    e.preventDefault();
    if (!checked() || !matches() || busy()) return;
    setBusy(true);
    setError(null);
    try {
      await props.onConfirm();
    } catch (err) {
      setError(describeAdminError(err));
    } finally {
      setBusy(false);
    }
  }

  const label = (key: string, count: number): string => props.labels[key]?.(count) ?? key;

  return (
    <Portal>
      <div class={css.dialogScrim} onClick={cancel} />
      <div
        ref={setEl}
        class={css.dialog}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        aria-describedby={`${id}-impact`}
        tabindex="-1"
        data-testid="impact-preview"
      >
        <form class={ux.dialogForm} onSubmit={(e) => void confirm(e)}>
          <h2 id={`${id}-title`} class={css.dialogTitle}>
            {props.title}
          </h2>
          <Show when={props.body}>{(body) => <p class={css.help}>{body()}</p>}</Show>
          <div id={`${id}-impact`} class={ux.impact} aria-live="polite" aria-busy={impact.loading}>
            <Switch>
              <Match when={impact.loading || impact.state === 'unresolved'}>
                <p class={css.help} role="status">
                  {t('admin-ux-impact-checking')}
                </p>
              </Match>
              <Match when={impact.state === 'errored'}>
                <div class={ux.stateRow}>
                  <Feedback kind="error" message={describeAdminError(impact.error)} />
                  <button type="button" class="btn" onClick={() => void refetch()}>
                    {t('admin-shell-retry')}
                  </button>
                </div>
              </Match>
              <Match when={value() === null}>
                <p class={css.help}>{t('admin-ux-impact-unavailable')}</p>
              </Match>
              <Match when={entries().length === 0 || entries().every(([, v]) => v.count === 0)}>
                <p class={css.help}>{t('admin-ux-impact-none')}</p>
              </Match>
              <Match when={entries().length > 0}>
                <h3 class={ux.impactHeading}>{t('admin-ux-impact-heading')}</h3>
                <ul class={ux.impactList}>
                  <For each={entries()}>
                    {([key, item]) => (
                      <li class={ux.impactItem} data-impact={key}>
                        <span class={ux.impactCount}>
                          {t('admin-ux-impact-count', { label: label(key, item.count), count: formatNumber(item.count) })}
                        </span>
                        <Show when={(item.names ?? []).length > 0}>
                          <span class={ux.impactNames}>
                            {(item.names ?? [])
                              .slice(0, IMPACT_NAMES_SHOWN)
                              .map((n) => isolate(n))
                              .join(', ')}
                            <Show when={item.count > Math.min(IMPACT_NAMES_SHOWN, (item.names ?? []).length)}>
                              {' '}
                              {t('admin-ux-impact-more', {
                                count: item.count - Math.min(IMPACT_NAMES_SHOWN, (item.names ?? []).length),
                              })}
                            </Show>
                          </span>
                        </Show>
                      </li>
                    )}
                  </For>
                </ul>
              </Match>
            </Switch>
          </div>
          <Show when={props.typedConfirmation}>
            {(target) => (
              <label class="field">
                <span>{t('admin-shell-confirm-type', { name: isolate(target()) })}</span>
                <input
                  ref={typedInput}
                  type="text"
                  autocomplete="off"
                  spellcheck={false}
                  value={typed()}
                  onInput={(e) => setTyped(e.currentTarget.value)}
                />
              </label>
            )}
          </Show>
          <Feedback kind="error" message={error()} />
          <div class={css.dialogActions}>
            <button ref={cancelButton} type="button" class="btn btn--ghost" onClick={cancel} disabled={busy()}>
              {t('admin-shell-cancel')}
            </button>
            <button
              type="submit"
              class={props.danger === false ? 'btn btn--primary' : 'btn btn--danger'}
              disabled={!checked() || !matches() || busy()}
            >
              {props.confirmLabel}
            </button>
          </div>
        </form>
      </div>
    </Portal>
  );
}
