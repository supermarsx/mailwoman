// Step-by-step setup frame (t32-ext.md §K): new domain, lockout policy, settings
// policy.
//
// Each step validates before Next; Back keeps what was entered. The last step is
// always a review that lists every value in plain language, and nothing is saved
// until Save on that step. Focus moves to the step's heading on every change so
// screen-reader and keyboard users land at the top of the new step. While the
// wizard holds unsaved input it registers with the dirty-state guard.

import { For, Show, createSignal, createUniqueId, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { Feedback, describeAdminError } from './Feedback.tsx';
import { useDirtyGuard } from './useDirtyGuard.ts';
import * as css from './shell.css.ts';

export interface WizardStep {
  id: string;
  title: string;
  content: () => JSX.Element;
  /** Returns a sentence saying what to fix, or null when the step is complete. */
  validate?: () => string | null;
}

export interface ReviewRow {
  label: string;
  value: string;
}

export interface WizardProps {
  title: string;
  steps: readonly WizardStep[];
  /** The review step's rows: every value, in plain language. */
  review: () => readonly ReviewRow[];
  /** Performs the save. A rejection keeps the wizard on the review step with the error. */
  onSave: () => Promise<void>;
  onCancel?: () => void;
  saveLabel?: string;
  /** True while the admin has entered something not yet saved. */
  dirty?: () => boolean;
}

export function Wizard(props: WizardProps): JSX.Element {
  const id = createUniqueId();
  const [index, setIndex] = createSignal(0);
  // A step's check stays on the step; a failed save is a toast (t32 f39).
  const [problem, setProblem] = createSignal<string | null>(null);
  const [saveError, setSaveError] = createSignal<string | null>(null);
  const [saving, setSaving] = createSignal(false);
  const [saved, setSaved] = createSignal(false);
  let heading: HTMLHeadingElement | undefined;

  useDirtyGuard(() => !saved() && (props.dirty?.() ?? false));

  /** Steps plus the review step at the end. */
  const total = (): number => props.steps.length + 1;
  const onReview = (): boolean => index() === props.steps.length;
  const titleAt = (i: number): string =>
    i < props.steps.length ? (props.steps[i]?.title ?? '') : t('admin-shell-wizard-review');

  function go(i: number): void {
    setProblem(null);
    setIndex(i);
    queueMicrotask(() => heading?.focus());
  }

  function next(): void {
    const step = props.steps[index()];
    const msg = step?.validate?.() ?? null;
    if (msg !== null) {
      setProblem(msg);
      return;
    }
    go(index() + 1);
  }

  async function save(): Promise<void> {
    setSaving(true);
    setProblem(null);
    setSaveError(null);
    try {
      await props.onSave();
      setSaved(true);
    } catch (err) {
      setSaveError(describeAdminError(err));
    } finally {
      setSaving(false);
    }
  }

  return (
    <section class={css.wizard} aria-labelledby={`${id}-title`}>
      <h2 id={`${id}-title`} class={css.dialogTitle}>
        {props.title}
      </h2>
      <ol class={css.steps} aria-label={t('admin-shell-wizard-steps')}>
        <For each={Array.from({ length: total() }, (_, i) => i)}>
          {(i) => (
            <li
              class={[css.stepItem, i < index() ? css.stepDone : ''].join(' ')}
              aria-current={i === index() ? 'step' : undefined}
            >
              {t('admin-shell-wizard-step-n', { n: i + 1, title: titleAt(i) })}
            </li>
          )}
        </For>
      </ol>
      <h3 ref={heading} tabindex="-1" class={css.chartTitle}>
        {t('admin-shell-wizard-step-of', { n: index() + 1, total: total(), title: titleAt(index()) })}
      </h3>
      <Show
        when={onReview()}
        fallback={<div data-step={props.steps[index()]?.id}>{props.steps[index()]?.content()}</div>}
      >
        <dl class={css.reviewList} data-step="review">
          <For each={props.review()}>
            {(row) => (
              <>
                <dt>{row.label}</dt>
                <dd>{row.value}</dd>
              </>
            )}
          </For>
        </dl>
      </Show>
      <Feedback kind="error" message={problem()} />
      <Feedback kind="error" as="toast" message={saveError()} />
      <Show when={saved()}>
        <Feedback kind="success" message={t('admin-shell-wizard-saved')} />
      </Show>
      <div class={css.wizardActions}>
        <div class={css.rowActions}>
          <Show when={props.onCancel}>
            <button type="button" class="btn btn--ghost" onClick={() => props.onCancel?.()} disabled={saving()}>
              {t('admin-shell-cancel')}
            </button>
          </Show>
        </div>
        <div class={css.rowActions}>
          <button type="button" class="btn" disabled={index() === 0 || saving() || saved()} onClick={() => go(index() - 1)}>
            {t('admin-shell-wizard-back')}
          </button>
          <Show
            when={onReview()}
            fallback={
              <button type="button" class="btn btn--primary" onClick={next}>
                {t('admin-shell-wizard-next')}
              </button>
            }
          >
            <button type="button" class="btn btn--primary" disabled={saving() || saved()} onClick={() => void save()}>
              {props.saveLabel ?? t('admin-shell-save')}
            </button>
          </Show>
        </div>
      </div>
    </section>
  );
}
