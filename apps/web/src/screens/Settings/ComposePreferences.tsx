import { createEffect, createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { normalizeTrustedDomain } from '../../components/compose/recipientTrust.ts';
import { isReplyImagePolicy, type ReplyImagePolicy } from '../../components/compose/replyImagePolicy.ts';
import { isCheckPolicy, type CheckPolicy } from '../../components/compose/sendChecks.ts';
import { isRecipientSeparator, type RecipientSeparator } from '../../components/compose/recipientChips.ts';
import { DEFAULT_HEADER_PRIORITY, HEADER_LABELS, HEADER_SOURCES, isReplyFromMode, parseHeaderPriority, type ReplyFromMode } from '../../components/compose/replyIdentity.ts';
import { HEADER_COLLAPSE_DEFAULT, HEADER_COLLAPSE_MODES, isHeaderCollapseMode, type HeaderCollapseMode } from '../../components/compose/headerCollapse.ts';
import { useSetting } from './useSetting.ts';
import { SelectRow, type CardSpec, ToggleRow } from './layout/rows.tsx';
import { SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import { t } from '../../i18n';
import * as css from './styles.css.ts';
import {ResponseTemplatePreferences} from './ResponseTemplatePreferences.tsx';
import {ComposeDefaults} from './ComposeDefaults.tsx';
import {ComposerFeatures} from './ComposerFeatures.tsx';
import {ComposeDelivery} from './ComposeDelivery.tsx';

const CHECK_ROWS: ReadonlyArray<{ key: string; id: string; label: string }> = [
  { key: 'mail.compose.checkInvalidRecipients', id: 'check-invalid', label: 'settings-check-invalid' },
  { key: 'mail.compose.checkMissingAttachment', id: 'check-attachment', label: 'settings-check-attachment' },
  { key: 'mail.compose.checkEmptySubject', id: 'check-subject', label: 'settings-check-subject' },
];

const POLICIES = ['allow', 'warn', 'warn-remember', 'deny'] as const;

function CheckPolicyField(props: { settingKey: string; id: string; label: string }): JSX.Element {
  const policy = useSetting<CheckPolicy>(props.settingKey, 'warn', isCheckPolicy);
  return <SelectRow id={props.id} label={t(props.label)} handle={policy} options={POLICIES.map(value => ({ value, label: t(`settings-check-policy-${value}`) }))} accept={isCheckPolicy} />;
}

/** Static description of this section for the cross-section search index. */
export const COMPOSE_CARDS: readonly CardSpec[] = [
  { id: 'compose-card-addressing', section: 'compose', title: 'settings-compose-card-addressing', help: 'settings-compose-card-addressing-help', rows: [
    { id: 'show-from', label: 'settings-show-from', help: 'settings-show-from-help' },
    { id: 'reply-from-mode', label: 'settings-reply-from', help: 'settings-reply-from-help' },
    { id: 'quick-reply', label: 'mail-quick-reply', help: 'mail-quick-reply-help' },
    { id: 'auto-chip', label: 'settings-auto-chip', help: 'settings-auto-chip-help' },
    { id: 'recipient-separator', label: 'settings-recipient-separator' },
    { id: 'header-collapse', label: 'settings-header-collapse', help: 'settings-header-collapse-help' },
    { id: 'quote-style', label: 'settings-quote-style', help: 'settings-quote-help' },
  ] },
  { id: 'compose-card-checks', section: 'compose', title: 'settings-compose-card-checks', help: 'settings-check-help', rows: [
    { id: 'check-invalid', label: 'settings-check-invalid' }, { id: 'check-attachment', label: 'settings-check-attachment' }, { id: 'check-subject', label: 'settings-check-subject' },
    { id: 'external-warning', label: 'settings-external-warning', help: 'settings-external-help' },
    { id: 'trusted-domains', label: 'settings-trusted-domains', help: 'settings-trusted-domains-help' },
  ] },
  { id: 'compose-card-images', section: 'compose', title: 'settings-compose-card-images', help: 'settings-compose-card-images-help', rows: [
    { id: 'reply-images', label: 'settings-reply-images', help: 'settings-reply-images-help' },
    { id: 'reply-image-domains', label: 'settings-reply-image-domains', help: 'settings-reply-image-domains-help' },
  ] },
];

export function ComposePreferences(): JSX.Element {
  const quote = useSetting<'classic'|'outlook'>('mail.quoteStyle', 'classic', (v): v is 'classic'|'outlook' => v === 'classic' || v === 'outlook');
  const showFrom = useSetting<boolean>('mail.showFrom', false);
  const replyFrom = useSetting<ReplyFromMode>('mail.replyFromMode', 'match', isReplyFromMode);
  const headerOrder = useSetting<string[]>('mail.replyFrom.headerPriority', DEFAULT_HEADER_PRIORITY.map(source => HEADER_LABELS[source]), (v): v is string[] => Array.isArray(v) && v.every(item => typeof item === 'string'));
  const [headerText, setHeaderText] = createSignal('');
  const [headerError, setHeaderError] = createSignal('');
  createEffect(() => setHeaderText(parseHeaderPriority(headerOrder.value()).map(source => HEADER_LABELS[source]).join('\n')));
  const headerDisabled = () => !headerOrder.entry() || headerOrder.locked() || headerOrder.saving();
  async function saveHeaders(): Promise<void> {
    const entries = headerText().split(/[\n,;]+/).map(value => value.trim()).filter(Boolean);
    const known = entries.map(value => HEADER_SOURCES.find(source => HEADER_LABELS[source].toLowerCase() === value.toLowerCase()));
    if (entries.length === 0 || entries.length > HEADER_SOURCES.length || known.some(value => value === undefined)) { setHeaderError(t('settings-reply-from-headers-invalid')); return; }
    setHeaderError('');
    await headerOrder.set([...new Set(known as typeof HEADER_SOURCES[number][])].map(source => HEADER_LABELS[source]));
  }
  const quick = useSetting<boolean>('mail.quickReply', false);
  const autoChip = useSetting<boolean>('mail.compose.autoChipRecipients', true);
  const separator = useSetting<RecipientSeparator>('mail.compose.recipientSeparator', 'comma', isRecipientSeparator);
  const headerCollapse = useSetting<HeaderCollapseMode>('mail.compose.headerCollapse', HEADER_COLLAPSE_DEFAULT, isHeaderCollapseMode);
  const warning = useSetting<'off'|'warn'|'confirm'|'warn-remember'|'deny'>('mail.externalRecipientWarning', 'warn', (v): v is 'off'|'warn'|'confirm'|'warn-remember'|'deny' => v === 'off' || v === 'warn' || v === 'confirm' || v === 'warn-remember' || v === 'deny');
  const domains = useSetting<string[]>('mail.trustedRecipientDomains', [], (v): v is string[] => Array.isArray(v) && v.every(item => typeof item === 'string'));
  const imagePolicy = useSetting<ReplyImagePolicy>('mail.replyImages', 'ask', isReplyImagePolicy);
  const imageDomains = useSetting<string[]>('mail.replyImageTrustedDomains', [], (v): v is string[] => Array.isArray(v) && v.every(item => typeof item === 'string'));
  const [imageDomainText, setImageDomainText] = createSignal('');
  const [imageDomainError, setImageDomainError] = createSignal('');
  createEffect(() => setImageDomainText(imageDomains.value().join('\n')));
  const imageDomainsDisabled = () => !imageDomains.entry() || imageDomains.locked() || imageDomains.saving();
  async function saveImageDomains(): Promise<void> {
    const entries = imageDomainText().split(/[\n,;]+/).map(value => value.trim()).filter(Boolean);
    const normalized = entries.map(normalizeTrustedDomain);
    if (entries.length > 200 || normalized.some(value => value === null)) { setImageDomainError(t('settings-trusted-domains-invalid')); return; }
    setImageDomainError('');
    await imageDomains.set([...new Set(normalized as string[])]);
  }
  const [domainText, setDomainText] = createSignal('');
  const [domainError, setDomainError] = createSignal('');
  const savedDomains = createMemo(() => domains.value().join('\n'));
  createEffect(() => setDomainText(savedDomains()));
  const domainsDisabled = () => !domains.entry() || domains.locked() || domains.saving();
  async function saveDomains(): Promise<void> {
    const entries = domainText().split(/[\n,;]+/).map(value => value.trim()).filter(Boolean);
    const normalized = entries.map(normalizeTrustedDomain);
    if (entries.length > 100 || normalized.some(value => value === null)) {
      setDomainError(t('settings-trusted-domains-invalid')); return;
    }
    setDomainError('');
    await domains.set([...new Set(normalized as string[])]);
  }
  return <>
    <header><h3 class={css.heading}>{t('settings-compose-title')}</h3></header>
    <SettingsCard id="compose-card-addressing" title={t('settings-compose-card-addressing')} description={t('settings-compose-card-addressing-help')}>
      <ToggleRow id="show-from" label={t('settings-show-from')} help={t('settings-show-from-help')} handle={showFrom} />
      <SelectRow id="reply-from-mode" label={t('settings-reply-from')} help={t('settings-reply-from-help')} handle={replyFrom} accept={isReplyFromMode} options={(['match', 'default', 'ask', 'keep'] as const).map(v => ({ value: v, label: t(`settings-reply-from-${v}`) }))} />
      <SettingRow stacked controlId="reply-from-headers" label={t('settings-reply-from-headers')} help={t('settings-reply-from-headers-help')} handle={headerOrder} onReset={() => void headerOrder.reset()} resetDisabled={headerDisabled()}>{describedBy => <>
        <textarea id="reply-from-headers" class={css.textarea} value={headerText()} disabled={headerDisabled()} onInput={event => setHeaderText(event.currentTarget.value)} aria-describedby={describedBy()} />
        <Show when={headerError() || headerOrder.error()}><p role="alert">{headerError() || headerOrder.error()}</p></Show>
        <button type="button" class={css.button} disabled={headerDisabled()} onClick={() => void saveHeaders()}>{t('settings-reply-from-headers-save')}</button>
      </>}</SettingRow>
      <ToggleRow id="quick-reply" label={t('mail-quick-reply')} help={t('mail-quick-reply-help')} handle={quick} />
      <ToggleRow id="auto-chip" label={t('settings-auto-chip')} help={t('settings-auto-chip-help')} handle={autoChip} />
      <SelectRow id="recipient-separator" label={t('settings-recipient-separator')} handle={separator} accept={isRecipientSeparator} options={(['comma', 'semicolon'] as const).map(v => ({ value: v, label: t(`settings-recipient-separator-${v}`) }))} />
      <SelectRow id="header-collapse" label={t('settings-header-collapse')} help={t('settings-header-collapse-help')} handle={headerCollapse} accept={isHeaderCollapseMode} options={HEADER_COLLAPSE_MODES.map(v => ({ value: v, label: t(`settings-header-collapse-${v}`) }))} />
      <SelectRow id="quote-style" label={t('settings-quote-style')} help={t('settings-quote-help')} handle={quote} accept={(v): v is 'classic' | 'outlook' => v === 'classic' || v === 'outlook'} options={[{ value: 'classic', label: t('settings-quote-classic') }, { value: 'outlook', label: t('settings-quote-outlook') }]} />
    </SettingsCard>
    <SettingsCard id="compose-card-checks" title={t('settings-compose-card-checks')} description={t('settings-check-help')}>
      <For each={CHECK_ROWS}>{row => <CheckPolicyField settingKey={row.key} id={row.id} label={row.label} />}</For>
      <SelectRow id="external-warning" label={t('settings-external-warning')} help={t('settings-external-help')} handle={warning}
        accept={(v): v is 'off' | 'warn' | 'confirm' | 'warn-remember' | 'deny' => ['off', 'warn', 'confirm', 'warn-remember', 'deny'].includes(v)}
        options={(['off', 'warn', 'confirm', 'warn-remember', 'deny'] as const).map(v => ({ value: v, label: t(`settings-external-${v}`) }))} />
      <SettingRow stacked controlId="trusted-domains" label={t('settings-trusted-domains')} help={t('settings-trusted-domains-help')} handle={domains} onReset={() => void domains.reset()} resetDisabled={domainsDisabled()}>{describedBy => <>
        <textarea id="trusted-domains" class={css.textarea} value={domainText()} disabled={domainsDisabled()} onInput={event => setDomainText(event.currentTarget.value)} aria-describedby={describedBy()} />
        <Show when={domainError() || domains.error()}><p role="alert">{domainError() || domains.error()}</p></Show>
        <button type="button" class={css.button} disabled={domainsDisabled()} onClick={() => void saveDomains()}>{t('settings-trusted-domains-save')}</button>
      </>}</SettingRow>
    </SettingsCard>
    <SettingsCard id="compose-card-images" title={t('settings-compose-card-images')} description={t('settings-compose-card-images-help')}>
      <SelectRow id="reply-images" label={t('settings-reply-images')} help={t('settings-reply-images-help')} handle={imagePolicy} accept={isReplyImagePolicy} options={(['ask', 'allow', 'trusted', 'deny'] as const).map(v => ({ value: v, label: t(`settings-reply-images-${v}`) }))} />
      <SettingRow stacked controlId="reply-image-domains" label={t('settings-reply-image-domains')} help={t('settings-reply-image-domains-help')} handle={imageDomains} onReset={() => void imageDomains.reset()} resetDisabled={imageDomainsDisabled()}>{describedBy => <>
        <textarea id="reply-image-domains" class={css.textarea} value={imageDomainText()} disabled={imageDomainsDisabled()} onInput={event => setImageDomainText(event.currentTarget.value)} aria-describedby={describedBy()} />
        <Show when={imageDomainError() || imageDomains.error()}><p role="alert">{imageDomainError() || imageDomains.error()}</p></Show>
        <button type="button" class={css.button} disabled={imageDomainsDisabled()} onClick={() => void saveImageDomains()}>{t('settings-reply-image-domains-save')}</button>
      </>}</SettingRow>
    </SettingsCard>
    <ComposeDefaults />
    <ComposeDelivery />
    <ComposerFeatures />
    <ResponseTemplatePreferences />
  </>;
}
