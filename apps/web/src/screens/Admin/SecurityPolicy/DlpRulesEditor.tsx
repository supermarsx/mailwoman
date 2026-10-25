// The stored DLP rules of the security policy, edited as rules instead of a JSON
// text (t32-admin §3.1 e16, absorbing t29-e11).
//
// Each rule is a group of plain controls: name, on/off, what happens on a match,
// priority, the sentence the sender sees, the built-in detectors, a custom
// pattern, recipient domains, attachment types and size. Members the editor has
// no control for (dictionaries, classification) are kept as stored. Rules from
// `MW_DLP_RULES` are listed above the editor, read-only: the server applies them
// beside the stored ones and an environment rule wins over a stored rule with the
// same id.
//
// `validateDlpRules` is the client check run before any request (UX5); the server
// checks the rules again and answers 422 with the reason.

import { For, Index, Show, createUniqueId, type JSX } from 'solid-js';
import type { DlpAction, DlpDetector, DlpRule } from '../../../api/crypto-types.ts';
import { isolate, t } from '../../../i18n';
import { ListEditor } from '../shell/ListEditor.tsx';
import { StatusBadge } from '../shell/StatusBadge.tsx';
import * as shell from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from '../shell/extras.css.ts';
import * as own from './securityPolicy.css.ts';

export const DLP_DETECTORS: readonly DlpDetector[] = ['pan', 'iban', 'national-id', 'ssn', 'pt-nif', 'pt-niss', 'pt-citizen-card', 'credentials', 'custom-regex'];
export const DLP_ACTIONS: readonly DlpAction[] = ['warn', 'block', 'require-encryption', 'notify-admin'];

/** A new rule with an id no listed rule uses. */
export function newDlpRule(taken: readonly string[]): DlpRule {
  let n = taken.length + 1;
  while (taken.includes(`rule-${n}`)) n += 1;
  return {
    id: `rule-${n}`,
    name: '',
    enabled: true,
    priority: 0,
    action: 'warn',
    message: '',
    conditions: {
      detectors: [],
      customRegex: null,
      dictionaries: [],
      attachmentTypes: [],
      maxAttachmentSize: null,
      recipientDomains: [],
      recipientDomainMode: null,
      classification: null,
    },
  };
}

/** The rules from a stored `dlpRulesJson`, or null when it is not a list of rules. */
export function parseDlpRules(json: string): DlpRule[] | null {
  let raw: unknown;
  try {
    raw = JSON.parse(json);
  } catch {
    return null;
  }
  if (!Array.isArray(raw)) return null;
  const ok = raw.every(
    (r) =>
      typeof r === 'object' &&
      r !== null &&
      typeof (r as DlpRule).id === 'string' &&
      typeof (r as DlpRule).name === 'string' &&
      typeof (r as DlpRule).conditions === 'object' &&
      (r as DlpRule).conditions !== null,
  );
  return ok ? (raw as DlpRule[]) : null;
}

const DOMAIN = /^(?=.{1,253}$)(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,63}$/;

/** The first problem with the rules, as one sentence, or null. */
export function validateDlpRules(rules: readonly DlpRule[]): string | null {
  const ids = new Set<string>();
  for (const [i, r] of rules.entries()) {
    const n = i + 1;
    if (r.name.trim() === '') return t('admin-policy-dlp-err-name', { n });
    if (ids.has(r.id)) return t('admin-policy-dlp-err-id', { n, id: isolate(r.id) });
    ids.add(r.id);
    if (!Number.isInteger(r.priority)) return t('admin-policy-dlp-err-priority', { n });
    const c = r.conditions;
    if (c.detectors.includes('custom-regex')) {
      if ((c.customRegex ?? '').trim() === '') return t('admin-policy-dlp-err-pattern-missing', { n });
      try {
        new RegExp(c.customRegex ?? '', 'u');
      } catch {
        return t('admin-policy-dlp-err-pattern', { n });
      }
    }
    if (c.maxAttachmentSize !== null && (!Number.isInteger(c.maxAttachmentSize) || c.maxAttachmentSize < 0)) {
      return t('admin-policy-dlp-err-size', { n });
    }
    const hasCondition =
      c.detectors.length > 0 ||
      c.dictionaries.length > 0 ||
      c.attachmentTypes.length > 0 ||
      c.maxAttachmentSize !== null ||
      c.recipientDomains.length > 0 ||
      c.classification !== null;
    if (!hasCondition) return t('admin-policy-dlp-err-condition', { n });
  }
  return null;
}

export interface DlpRulesEditorProps {
  id: string;
  /** The field's name, for the group. */
  label: string;
  describedBy: string | undefined;
  disabled: boolean;
  rules: readonly DlpRule[];
  envRules: readonly DlpRule[];
  onChange: (rules: DlpRule[]) => void;
}

export function DlpRulesEditor(props: DlpRulesEditorProps): JSX.Element {
  const envIds = (): string[] => props.envRules.map((r) => r.id);
  function update(i: number, patch: Partial<DlpRule>): void {
    props.onChange(props.rules.map((r, j) => (j === i ? { ...r, ...patch } : r)));
  }
  function updateConditions(i: number, patch: Partial<DlpRule['conditions']>): void {
    const r = props.rules[i];
    if (r) update(i, { conditions: { ...r.conditions, ...patch } });
  }

  return (
    <div class={own.dlp} id={props.id} role="group" aria-label={props.label} aria-describedby={props.describedBy} data-testid="dlp-editor">
      <Show when={props.envRules.length > 0}>
        <div class={css.panel} data-testid="dlp-env-rules">
          <h3 class={css.panelTitle}>{t('admin-policy-dlp-env-heading')}</h3>
          <p class={css.panelText}>{t('admin-policy-dlp-env-note')}</p>
          <ul class={css.listItems}>
            <For each={props.envRules}>
              {(r) => (
                <li class={css.listItem}>
                  <span class={css.listText} dir="auto">
                    {r.name} <span class={css.mono}>({r.id})</span>
                  </span>
                  <span class={css.badgeRow}>
                    <StatusBadge tone={r.enabled ? 'ok' : 'off'}>{r.enabled ? t('admin-policy-dlp-on') : t('admin-policy-dlp-off')}</StatusBadge>
                    <StatusBadge tone="neutral">{t(`admin-policy-dlp-action-${r.action}`)}</StatusBadge>
                  </span>
                </li>
              )}
            </For>
          </ul>
        </div>
      </Show>
      <Show when={props.rules.length === 0}>
        <p class={css.listEmpty} id={`${props.id}-empty`}>
          {t('admin-policy-dlp-none')}
        </p>
      </Show>
      <Index each={props.rules}>
        {(rule, i) => (
          <RuleEditor
            rule={rule()}
            index={i}
            disabled={props.disabled}
            shadowed={envIds().includes(rule().id)}
            onChange={(patch) => update(i, patch)}
            onConditions={(patch) => updateConditions(i, patch)}
            onRemove={() => props.onChange(props.rules.filter((_, j) => j !== i))}
          />
        )}
      </Index>
      <div class={css.actions}>
        <button
          type="button"
          class="btn"
          disabled={props.disabled}
          onClick={() => props.onChange([...props.rules, newDlpRule([...props.rules.map((r) => r.id), ...envIds()])])}
        >
          {t('admin-policy-dlp-add')}
        </button>
      </div>
    </div>
  );
}

interface RuleEditorProps {
  rule: DlpRule;
  index: number;
  disabled: boolean;
  shadowed: boolean;
  onChange: (patch: Partial<DlpRule>) => void;
  onConditions: (patch: Partial<DlpRule['conditions']>) => void;
  onRemove: () => void;
}

function RuleEditor(props: RuleEditorProps): JSX.Element {
  const id = createUniqueId();
  const r = (): DlpRule => props.rule;
  const c = (): DlpRule['conditions'] => props.rule.conditions;
  const title = (): string => (r().name.trim() === '' ? t('admin-policy-dlp-rule-untitled', { n: props.index + 1 }) : r().name);
  const toggleDetector = (d: DlpDetector, on: boolean): void => {
    const rest = c().detectors.filter((x) => x !== d);
    props.onConditions({ detectors: on ? [...rest, d] : rest, ...(d === 'custom-regex' && !on ? { customRegex: null } : {}) });
  };

  return (
    <fieldset class={own.rule} data-testid="dlp-rule" data-rule-id={r().id}>
      <legend class={own.ruleLegend} dir="auto">
        {title()}
      </legend>
      <Show when={props.shadowed}>
        <p class={shell.help}>{t('admin-policy-dlp-shadowed', { id: isolate(r().id) })}</p>
      </Show>
      <div class={own.ruleGrid}>
        <div class={shell.field}>
          <label class={shell.fieldLabel} for={`${id}-name`}>
            {t('admin-policy-dlp-name')}
          </label>
          <input id={`${id}-name`} class={ux.control} type="text" value={r().name} disabled={props.disabled} onInput={(e) => props.onChange({ name: e.currentTarget.value })} />
        </div>
        <div class={shell.field}>
          <label class={shell.fieldLabel} for={`${id}-action`}>
            {t('admin-policy-dlp-action')}
          </label>
          <select id={`${id}-action`} class={ux.control} value={r().action} disabled={props.disabled} onChange={(e) => props.onChange({ action: e.currentTarget.value as DlpAction })}>
            <For each={DLP_ACTIONS}>
              {(a) => (
                <option value={a} selected={a === r().action}>
                  {t(`admin-policy-dlp-action-${a}`)}
                </option>
              )}
            </For>
          </select>
        </div>
        <div class={shell.field}>
          <label class={shell.fieldLabel} for={`${id}-priority`}>
            {t('admin-policy-dlp-priority')}
          </label>
          <input
            id={`${id}-priority`}
            class={ux.control}
            type="number"
            step="1"
            inputMode="numeric"
            value={String(r().priority)}
            disabled={props.disabled}
            aria-describedby={`${id}-priority-help`}
            onInput={(e) => props.onChange({ priority: e.currentTarget.value === '' ? 0 : Number(e.currentTarget.value) })}
          />
          <p id={`${id}-priority-help`} class={shell.help}>
            {t('admin-policy-dlp-priority-help')}
          </p>
        </div>
      </div>
      <div class={css.checkRow}>
        <input id={`${id}-enabled`} type="checkbox" checked={r().enabled} disabled={props.disabled} onChange={(e) => props.onChange({ enabled: e.currentTarget.checked })} />
        <label for={`${id}-enabled`}>{t('admin-policy-dlp-enabled')}</label>
      </div>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${id}-message`}>
          {t('admin-policy-dlp-message')}
        </label>
        <input
          id={`${id}-message`}
          class={ux.control}
          type="text"
          value={r().message}
          disabled={props.disabled}
          aria-describedby={`${id}-message-help`}
          onInput={(e) => props.onChange({ message: e.currentTarget.value })}
        />
        <p id={`${id}-message-help`} class={shell.help}>
          {t('admin-policy-dlp-message-help')}
        </p>
      </div>
      <fieldset class={css.checks}>
        <legend class={shell.fieldLabel}>{t('admin-policy-dlp-detectors')}</legend>
        <For each={DLP_DETECTORS}>
          {(d) => (
            <div class={css.checkRow}>
              <input
                id={`${id}-det-${d}`}
                type="checkbox"
                checked={c().detectors.includes(d)}
                disabled={props.disabled}
                onChange={(e) => toggleDetector(d, e.currentTarget.checked)}
              />
              <label for={`${id}-det-${d}`}>{t(`admin-policy-dlp-detector-${d}`)}</label>
            </div>
          )}
        </For>
      </fieldset>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${id}-keywords`}>{t('admin-policy-dlp-keywords')}</label>
        <ListEditor
          id={`${id}-keywords`}
          label={t('admin-policy-dlp-keywords')}
          items={c().dictionaries}
          split="none"
          disabled={props.disabled}
          placeholder={t('admin-policy-dlp-keywords-placeholder')}
          emptyText={t('admin-policy-dlp-keywords-none')}
          onChange={(list) => props.onConditions({ dictionaries: list })}
        />
        <p class={shell.help}>{t('admin-policy-dlp-keywords-help')}</p>
      </div>
      <Show when={c().detectors.includes('custom-regex')}>
        <div class={shell.field}>
          <label class={shell.fieldLabel} for={`${id}-regex`}>
            {t('admin-policy-dlp-pattern')}
          </label>
          <input
            id={`${id}-regex`}
            class={`${ux.control} ${css.mono}`}
            type="text"
            spellcheck={false}
            value={c().customRegex ?? ''}
            disabled={props.disabled}
            aria-describedby={`${id}-regex-help`}
            onInput={(e) => props.onConditions({ customRegex: e.currentTarget.value })}
          />
          <p id={`${id}-regex-help`} class={shell.help}>
            {t('admin-policy-dlp-pattern-help')}
          </p>
        </div>
      </Show>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${id}-domains`}>
          {t('admin-policy-dlp-domains')}
        </label>
        <ListEditor
          id={`${id}-domains`}
          label={t('admin-policy-dlp-domains')}
          items={c().recipientDomains}
          lowerCase
          mono
          disabled={props.disabled}
          placeholder={t('admin-policy-dlp-domains-placeholder')}
          emptyText={t('admin-policy-dlp-domains-none')}
          validate={(d) => (DOMAIN.test(d) ? null : t('admin-policy-dlp-err-domain', { domain: isolate(d) }))}
          onChange={(list) => props.onConditions({ recipientDomains: list, recipientDomainMode: list.length === 0 ? null : (c().recipientDomainMode ?? 'in') })}
        />
      </div>
      <Show when={c().recipientDomains.length > 0}>
        <div class={shell.field}>
          <label class={shell.fieldLabel} for={`${id}-mode`}>
            {t('admin-policy-dlp-domain-mode')}
          </label>
          <select
            id={`${id}-mode`}
            class={ux.control}
            value={c().recipientDomainMode ?? 'in'}
            disabled={props.disabled}
            onChange={(e) => props.onConditions({ recipientDomainMode: e.currentTarget.value as 'in' | 'notIn' })}
          >
            <option value="in">{t('admin-policy-dlp-domain-mode-in')}</option>
            <option value="notIn">{t('admin-policy-dlp-domain-mode-not-in')}</option>
          </select>
        </div>
      </Show>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${id}-types`}>
          {t('admin-policy-dlp-types')}
        </label>
        <ListEditor
          id={`${id}-types`}
          label={t('admin-policy-dlp-types')}
          items={c().attachmentTypes}
          lowerCase
          mono
          disabled={props.disabled}
          placeholder={t('admin-policy-dlp-types-placeholder')}
          emptyText={t('admin-policy-dlp-types-none')}
          onChange={(list) => props.onConditions({ attachmentTypes: list })}
        />
      </div>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${id}-size`}>
          {t('admin-policy-dlp-size')}
        </label>
        <input
          id={`${id}-size`}
          class={ux.control}
          type="number"
          min="0"
          step="1"
          inputMode="numeric"
          value={c().maxAttachmentSize === null ? '' : String(c().maxAttachmentSize)}
          disabled={props.disabled}
          aria-describedby={`${id}-size-help`}
          onInput={(e) => props.onConditions({ maxAttachmentSize: e.currentTarget.value === '' ? null : Number(e.currentTarget.value) })}
        />
        <p id={`${id}-size-help`} class={shell.help}>
          {t('admin-policy-dlp-size-help')}
        </p>
      </div>
      <Show when={c().dictionaries.length > 0 || c().classification !== null}>
        <p class={shell.help}>{t('admin-policy-dlp-kept')}</p>
      </Show>
      <div class={css.actions}>
        <button type="button" class="btn btn--ghost" disabled={props.disabled} onClick={() => props.onRemove()}>
          {t('admin-policy-dlp-remove', { name: isolate(title()) })}
        </button>
      </div>
    </fieldset>
  );
}
