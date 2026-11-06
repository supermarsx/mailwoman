// Settings > Compose > Sending options: the Reply-To row, where the copy of a sent
// message is filed, the default message font and what to do with expired messages.
// Each value is a settings-model key; an administrator can lock any of them.
import { createEffect, createSignal, onMount, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { SelectRow, ToggleRow, type CardSpec } from './layout/rows.tsx';
import { SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';

export const COMPOSE_DELIVERY_CARD: CardSpec = {
  id: 'compose-card-delivery', section: 'compose', title: 'cd-title', help: 'cd-help', rows: [
    { id: 'cd-reply-to-show', label: 'cd-reply-to-show', help: 'cd-reply-to-show-help' },
    { id: 'cd-sent-mode', label: 'cd-sent-mode', help: 'cd-sent-mode-help' },
    { id: 'cd-sent-fixed', label: 'cd-sent-fixed', help: 'cd-sent-fixed-help' },
    { id: 'cd-sent-identity', label: 'cd-sent-identity', help: 'cd-sent-identity-help' },
    { id: 'cd-sent-rules', label: 'cd-sent-rules', help: 'cd-sent-rules-help' },
    { id: 'cd-sent-skip', label: 'cd-sent-skip', help: 'cd-sent-skip-help' },
    { id: 'cd-expiry', label: 'cd-expiry', help: 'cd-expiry-help' },
  ],
};

const SHOW = ['auto', 'always', 'never'] as const;
const MODES = ['default', 'fixed', 'byRecipient', 'byIdentity', 'bySubjectRule'] as const;
const EXPIRY = ['off', 'dim', 'archive-suggestion', 'hide'] as const;
const isOf = <T extends string>(list: readonly T[]) => (v: unknown): v is T => typeof v === 'string' && (list as readonly string[]).includes(v);
const isStrings = (v: unknown): v is string[] => Array.isArray(v) && v.every((x) => typeof x === 'string');

/** A setting that holds a list of lines, edited as text and saved on request. */
function LinesRow(props: { id: string; label: string; help: string; settingKey: string; save: string }): JSX.Element {
  const handle = useSetting<string[]>(props.settingKey, [], isStrings);
  const [text, setText] = createSignal('');
  createEffect(() => setText(handle.value().join('\n')));
  const disabled = (): boolean => !handle.entry() || handle.locked() || handle.saving();
  const save = (): void => void handle.set(text().split('\n').map((l) => l.trim()).filter(Boolean).slice(0, 50));
  return <SettingRow stacked controlId={props.id} label={props.label} help={props.help} handle={handle} onReset={() => void handle.reset()} resetDisabled={disabled()}>{(describedBy) => <>
    <textarea id={props.id} class={css.textarea} value={text()} disabled={disabled()} onInput={(e) => setText(e.currentTarget.value)} aria-describedby={describedBy()} />
    <button type="button" class={css.button} disabled={disabled()} onClick={save}>{props.save}</button>
  </>}</SettingRow>;
}

export function ComposeDelivery(): JSX.Element {
  onMount(() => { void loadCatalog('compose-layout'); });
  const show = useSetting<(typeof SHOW)[number]>('compose.replyTo.show', 'auto', isOf(SHOW));
  const mode = useSetting<(typeof MODES)[number]>('compose.sentFolder.mode', 'default', isOf(MODES));
  const fixed = useSetting<string>('compose.sentFolder.fixedId', '', (v): v is string => typeof v === 'string');
  const skip = useSetting<boolean>('compose.sentFolder.skipCopy', false);
  const expiry = useSetting<(typeof EXPIRY)[number]>('mail.expiry.hideExpired', 'off', isOf(EXPIRY));
  const [fixedText, setFixedText] = createSignal('');
  createEffect(() => setFixedText(fixed.value()));
  const fixedDisabled = (): boolean => !fixed.entry() || fixed.locked() || fixed.saving();
  return <SettingsCard id="compose-card-delivery" title={t('cd-title')} description={t('cd-help')}>
    <SelectRow id="cd-reply-to-show" label={t('cd-reply-to-show')} help={t('cd-reply-to-show-help')} handle={show} accept={isOf(SHOW)} options={SHOW.map((v) => ({ value: v, label: t(`cd-reply-to-show-${v}`) }))} />
    <SelectRow id="cd-sent-mode" label={t('cd-sent-mode')} help={t('cd-sent-mode-help')} handle={mode} accept={isOf(MODES)} options={MODES.map((v) => ({ value: v, label: t(`cd-sent-mode-${v}`) }))} />
    <SettingRow controlId="cd-sent-fixed" label={t('cd-sent-fixed')} help={t('cd-sent-fixed-help')} handle={fixed} onReset={() => void fixed.reset()} resetDisabled={fixedDisabled()}>{(describedBy) => <>
      <input id="cd-sent-fixed" type="text" maxlength="200" value={fixedText()} disabled={fixedDisabled()} aria-describedby={describedBy()} onInput={(e) => setFixedText(e.currentTarget.value)} onChange={(e) => void fixed.set(e.currentTarget.value.trim())} />
    </>}</SettingRow>
    <LinesRow id="cd-sent-identity" label={t('cd-sent-identity')} help={t('cd-sent-identity-help')} settingKey="compose.sentFolder.perIdentity" save={t('cd-save')} />
    <LinesRow id="cd-sent-rules" label={t('cd-sent-rules')} help={t('cd-sent-rules-help')} settingKey="compose.sentFolder.rules" save={t('cd-save')} />
    <ToggleRow id="cd-sent-skip" label={t('cd-sent-skip')} help={t('cd-sent-skip-help')} handle={skip} />
    <SelectRow id="cd-expiry" label={t('cd-expiry')} help={t('cd-expiry-help')} handle={expiry} accept={isOf(EXPIRY)} options={EXPIRY.map((v) => ({ value: v, label: t(`cd-expiry-${v}`) }))} />
    <Show when={mode.value() === 'fixed' && fixed.value() === ''}><p role="status">{t('cd-sent-fixed-missing')}</p></Show>
  </SettingsCard>;
}
