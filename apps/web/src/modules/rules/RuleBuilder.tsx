import { Icon } from '../../components/Icon.tsx';
// Condition/action rule builder (audit #1): edits one rule (name, all/any match,
// conditions, actions) and shows a "where it runs" indicator derived from whether
// the rule is Sieve-expressible. WCAG 2.2 AA: every control labelled.
//
// `thread` conditions are not offered: the server cannot apply them and refuses
// a rule that has one (t32-h2). A stored rule that still shows one renders it
// read-only with a note, and the rule cannot be saved until it is removed.

import { createSignal, For, Show, type JSX } from 'solid-js';
import { createStore } from 'solid-js/store';
import { t, isolate } from '../../i18n';
import type {
  MailRuleActionType,
  MailRuleConditionType,
  MailRuleOp,
} from '../../api/crypto-types.ts';
import { runsAtFor } from './sieve.ts';
import type { RuleDraft } from '../../state/slices/rules.ts';
import { WhereItRuns } from './WhereItRuns.tsx';
import * as css from './styles.css.ts';

const CONDITION_TYPES: ReadonlyArray<{ value: MailRuleConditionType; label: string }> = [
  { value: 'from', label: 'rules-cond-from' },
  { value: 'to', label: 'rules-cond-to' },
  { value: 'subject', label: 'rules-cond-subject' },
];

const OPS: ReadonlyArray<{ value: MailRuleOp; label: string }> = [
  { value: 'contains', label: 'rules-op-contains' },
  { value: 'is', label: 'rules-op-is' },
];

const ACTION_TYPES: ReadonlyArray<{ value: MailRuleActionType; label: string }> = [
  { value: 'move', label: 'rules-act-move' },
  { value: 'tag', label: 'rules-act-tag' },
  { value: 'archive', label: 'rules-act-archive' },
  { value: 'forward', label: 'rules-forward-title' },
  { value: 'forwardCopy', label: 'rules-forward-copy' },
  { value: 'stop', label: 'rules-act-stop' },
];

/** Actions that carry a free-text value (a mailbox/keyword). */
const VALUED_ACTIONS = new Set<MailRuleActionType>(['move', 'tag', 'forward', 'forwardCopy']);

export interface RuleBuilderProps {
  runsAt?: 'engine' | 'server-sieve';
  extensions?: readonly string[];
  initial: RuleDraft;
  /** A rejected promise (a refused save) re-enables the form for another try. */
  onSave: (draft: RuleDraft) => void | Promise<void>;
  onCancel: () => void;
}

export function RuleBuilder(props: RuleBuilderProps): JSX.Element {
  const [draft, setDraft] = createStore<RuleDraft>({ ...props.initial });
  const [saving, setSaving] = createSignal(false);

  const addCondition = (): void =>
    setDraft('conditions', (c) => [...c, { type: 'from', op: 'contains', value: '' }]);
  const removeCondition = (i: number): void =>
    setDraft('conditions', (c) => c.filter((_, idx) => idx !== i));

  const addAction = (): void => setDraft('actions', (a) => [...a, { type: props.extensions && !props.extensions.includes('fileinto') ? 'stop' : 'move', value: '' }]);
  const removeAction = (i: number): void => setDraft('actions', (a) => a.filter((_, idx) => idx !== i));

  const runsAt = (): 'server-sieve' | 'engine' =>
    props.runsAt ?? runsAtFor({ conditions: draft.conditions, actions: draft.actions });

  const hasThread = (): boolean => draft.conditions.some((c) => c.type === 'thread');

  const canSave = (): boolean => draft.name.trim().length > 0 && !hasThread() && !saving();

  const save = (): void => {
    if (!canSave()) return;
    setSaving(true);
    // The slice has already told the user why a save failed; only re-enable here.
    void Promise.resolve()
      .then(() =>
        props.onSave({ ...draft, conditions: [...draft.conditions], actions: [...draft.actions], runsAt: runsAt() }),
      )
      .catch(() => setSaving(false));
  };

  return (
    <form
      class={css.builder}
      aria-label={t('rules-builder-title')}
      onSubmit={(e) => {
        e.preventDefault();
        save();
      }}
    >
      <div class={css.field}>
        <label class={css.label} for="rule-name">
          {t('rules-name-label')}
        </label>
        <input
          id="rule-name"
          class={css.input}
          value={draft.name}
          onInput={(e) => setDraft('name', e.currentTarget.value)}
        />
      </div>

      <fieldset class={css.field}>
        <legend class={css.label}>{t('rules-match-legend')}</legend>
        <div class={css.row} role="radiogroup" aria-label={t('rules-match-legend')}>
          <label class={css.row}>
            <input
              type="radio"
              name="matchAll"
              checked={draft.matchAll}
              onChange={() => setDraft('matchAll', true)}
            />
            {t('rules-match-all')}
          </label>
          <label class={css.row}>
            <input
              type="radio"
              name="matchAll"
              checked={!draft.matchAll}
              onChange={() => setDraft('matchAll', false)}
            />
            {t('rules-match-any')}
          </label>
        </div>
      </fieldset>

      <fieldset class={css.field}>
        <legend class={css.label}>{t('rules-conditions-legend')}</legend>
        <For each={draft.conditions}>
          {(cond, i) => (
            <div class={css.clause}>
              <Show
                when={cond.type !== 'thread'}
                fallback={
                  <p class={css.prose}>
                    {t('rules-cond-thread-unsupported', {
                      op: t(cond.op === 'is' ? 'rules-op-is' : 'rules-op-contains'),
                      value: cond.value,
                    })}
                  </p>
                }
              >
                <label class={css.srOnly} for={`cond-type-${i()}`}>
                  {t('rules-cond-type-label')}
                </label>
                <select
                  id={`cond-type-${i()}`}
                  class={css.select}
                  value={cond.type}
                  onChange={(e) => setDraft('conditions', i(), 'type', e.currentTarget.value as MailRuleConditionType)}
                >
                  <For each={CONDITION_TYPES}>{(o) => <option value={o.value}>{t(o.label)}</option>}</For>
                </select>
                <label class={css.srOnly} for={`cond-op-${i()}`}>
                  {t('rules-cond-op-label')}
                </label>
                <select
                  id={`cond-op-${i()}`}
                  class={css.select}
                  value={cond.op}
                  onChange={(e) => setDraft('conditions', i(), 'op', e.currentTarget.value as MailRuleOp)}
                >
                  <For each={OPS}>{(o) => <option value={o.value}>{t(o.label)}</option>}</For>
                </select>
                <label class={css.srOnly} for={`cond-value-${i()}`}>
                  {t('rules-cond-value-label')}
                </label>
                <input
                  id={`cond-value-${i()}`}
                  class={css.input}
                  value={cond.value}
                  onInput={(e) => setDraft('conditions', i(), 'value', e.currentTarget.value)}
                />
              </Show>
              <button
                type="button"
                class={css.iconBtn}
                aria-label={t('rules-remove-condition')}
                disabled={draft.conditions.length <= 1}
                onClick={() => removeCondition(i())}
              >
                <Icon name="close" />
              </button>
            </div>
          )}
        </For>
        <button type="button" class={css.btn} onClick={addCondition}>
          {t('rules-add-condition')}
        </button>
      </fieldset>

      <fieldset class={css.field}>
        <legend class={css.label}>{t('rules-actions-legend')}</legend>
        <For each={draft.actions}>
          {(act, i) => (
            <div class={css.clause}>
              <label class={css.srOnly} for={`act-type-${i()}`}>
                {t('rules-act-type-label')}
              </label>
              <select
                id={`act-type-${i()}`}
                class={css.select}
                value={act.type}
                onChange={(e) => setDraft('actions', i(), 'type', e.currentTarget.value as MailRuleActionType)}
              >
                <For each={ACTION_TYPES.filter((action) => props.extensions === undefined
                  || (['move','archive'].includes(action.value) ? props.extensions.includes('fileinto')
                    : action.value === 'tag' ? props.extensions.includes('imap4flags')
                    : action.value === 'forwardCopy' ? props.extensions.includes('copy') : true))}>{(o) => <option value={o.value}>{t(o.label)}</option>}</For>
              </select>
              <Show when={VALUED_ACTIONS.has(act.type)}>
                <label class={css.srOnly} for={`act-value-${i()}`}>
                  {t('rules-act-value-label')}
                </label>
                <input
                  id={`act-value-${i()}`}
                  class={css.input}
                  value={act.value ?? ''}
                  onInput={(e) => setDraft('actions', i(), 'value', e.currentTarget.value)}
                />
              </Show>
              <button
                type="button"
                class={css.iconBtn}
                aria-label={t('rules-remove-action')}
                disabled={draft.actions.length <= 1}
                onClick={() => removeAction(i())}
              >
                <Icon name="close" />
              </button>
            </div>
          )}
        </For>
        <button type="button" class={css.btn} onClick={addAction}>
          {t('rules-add-action')}
        </button>
      </fieldset>

      <WhereItRuns runsAt={runsAt()} />

      <div class={css.row}>
        <button type="submit" class={css.btnPrimary} disabled={!canSave()}>
          {t('rules-save')}
        </button>
        <button type="button" class={css.btn} onClick={() => props.onCancel()}>
          {t('common-cancel')}
        </button>
        <Show when={draft.name.trim().length > 0}>
          <span class={css.prose}>{t('rules-editing', { name: isolate(draft.name) })}</span>
        </Show>
      </div>
    </form>
  );
}
