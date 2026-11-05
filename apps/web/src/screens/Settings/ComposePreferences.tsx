import { createEffect, createMemo, createSignal, Show, type JSX } from 'solid-js';
import { normalizeTrustedDomain } from '../../components/compose/recipientTrust.ts';
import { isReplyImagePolicy, type ReplyImagePolicy } from '../../components/compose/replyImagePolicy.ts';
import { isCheckPolicy, type CheckPolicy } from '../../components/compose/sendChecks.ts';
import { isRecipientSeparator, type RecipientSeparator } from '../../components/compose/recipientChips.ts';
import { isReplyFromMode, type ReplyFromMode } from '../../components/compose/replyIdentity.ts';
import { HEADER_COLLAPSE_DEFAULT, HEADER_COLLAPSE_MODES, isHeaderCollapseMode, type HeaderCollapseMode } from '../../components/compose/headerCollapse.ts';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { t } from '../../i18n';
import * as css from './styles.css.ts';
import {ResponseTemplatePreferences} from './ResponseTemplatePreferences.tsx';
import {ComposeDefaults} from './ComposeDefaults.tsx';

const CHECK_ROWS: ReadonlyArray<{ key: string; id: string; label: string }> = [
  { key: 'mail.compose.checkInvalidRecipients', id: 'check-invalid', label: 'settings-check-invalid' },
  { key: 'mail.compose.checkMissingAttachment', id: 'check-attachment', label: 'settings-check-attachment' },
  { key: 'mail.compose.checkEmptySubject', id: 'check-subject', label: 'settings-check-subject' },
];

function CheckPolicyField(props: { settingKey: string; id: string; label: string }): JSX.Element {
  const policy = useSetting<CheckPolicy>(props.settingKey, 'warn', isCheckPolicy);
  const disabled = () => !policy.entry() || policy.locked() || policy.saving();
  return <div class={css.field}><label for={props.id}>{t(props.label)}</label>
    <select id={props.id} class={css.select} value={policy.value()} disabled={disabled()} onChange={event => { const next = event.currentTarget.value; if (isCheckPolicy(next)) void policy.set(next); }} aria-describedby="check-policy-help">
      <option value="allow">{t('settings-check-policy-allow')}</option><option value="warn">{t('settings-check-policy-warn')}</option><option value="warn-remember">{t('settings-check-policy-warn-remember')}</option><option value="deny">{t('settings-check-policy-deny')}</option>
    </select><LockNote locked={policy.locked()}/>
    <Show when={policy.error()}><p role="alert">{policy.error()}</p></Show>
    <div class={css.actions}><button type="button" class={css.ghost} disabled={disabled()} onClick={() => void policy.reset()}>{t('reader-inherit')}</button></div></div>;
}

export function ComposePreferences(): JSX.Element {
  const quote = useSetting<'classic'|'outlook'>('mail.quoteStyle', 'classic', (v): v is 'classic'|'outlook' => v === 'classic' || v === 'outlook');
  const disabled = () => !quote.entry() || quote.locked() || quote.saving();
  const showFrom = useSetting<boolean>('mail.showFrom', false);
  const replyFrom = useSetting<ReplyFromMode>('mail.replyFromMode', 'match', isReplyFromMode);
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
  return <section class={css.section} aria-label={t('settings-compose-title')}><h3 class={css.heading}>{t('settings-compose-title')}</h3>
    <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={showFrom.value()} disabled={!showFrom.entry() || showFrom.locked() || showFrom.saving()} onChange={event => void showFrom.set(event.currentTarget.checked)} aria-describedby="show-from-help"/>{t('settings-show-from')}</label><p id="show-from-help" class={css.meta}>{t('settings-show-from-help')}</p><LockNote locked={showFrom.locked()}/><Show when={showFrom.error()}><p role="alert">{showFrom.error()}</p></Show><div class={css.actions}><button type="button" class={css.ghost} disabled={!showFrom.entry() || showFrom.locked() || showFrom.saving()} onClick={() => void showFrom.reset()}>{t('reader-inherit')}</button></div></div>
    <div class={css.field}><label class={css.label} for="reply-from-mode">{t('settings-reply-from')}</label><select id="reply-from-mode" class={css.select} style={{'max-inline-size':'24rem'}} value={replyFrom.value()} disabled={!replyFrom.entry() || replyFrom.locked() || replyFrom.saving()} onChange={event => { const next = event.currentTarget.value; if (isReplyFromMode(next)) void replyFrom.set(next); }} aria-describedby="reply-from-help"><option value="match">{t('settings-reply-from-match')}</option><option value="default">{t('settings-reply-from-default')}</option><option value="ask">{t('settings-reply-from-ask')}</option><option value="keep">{t('settings-reply-from-keep')}</option></select><p id="reply-from-help" class={css.meta}>{t('settings-reply-from-help')}</p><LockNote locked={replyFrom.locked()}/><Show when={replyFrom.error()}><p role="alert">{replyFrom.error()}</p></Show><div class={css.actions}><button type="button" class={css.ghost} disabled={!replyFrom.entry() || replyFrom.locked() || replyFrom.saving()} onClick={() => void replyFrom.reset()}>{t('reader-inherit')}</button></div></div>
    <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={quick.value()} disabled={!quick.entry() || quick.locked() || quick.saving()} onChange={event => void quick.set(event.currentTarget.checked)} aria-describedby="quick-reply-help"/>{t('mail-quick-reply')}</label><p id="quick-reply-help" class={css.meta}>{t('mail-quick-reply-help')}</p><LockNote locked={quick.locked()}/><Show when={quick.error()}><p role="alert">{quick.error()}</p></Show><div class={css.actions}><button type="button" class={css.ghost} disabled={!quick.entry() || quick.locked() || quick.saving()} onClick={() => void quick.reset()}>{t('reader-inherit')}</button></div></div>
    <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={autoChip.value()} disabled={!autoChip.entry() || autoChip.locked() || autoChip.saving()} onChange={event => void autoChip.set(event.currentTarget.checked)} aria-describedby="auto-chip-help"/>{t('settings-auto-chip')}</label><p id="auto-chip-help" class={css.meta}>{t('settings-auto-chip-help')}</p><LockNote locked={autoChip.locked()}/><Show when={autoChip.error()}><p role="alert">{autoChip.error()}</p></Show><div class={css.actions}><button type="button" class={css.ghost} disabled={!autoChip.entry() || autoChip.locked() || autoChip.saving()} onClick={() => void autoChip.reset()}>{t('reader-inherit')}</button></div></div>
    <div class={css.field}><label for="recipient-separator">{t('settings-recipient-separator')}</label><select id="recipient-separator" class={css.select} style={{'max-inline-size':'24rem'}} value={separator.value()} disabled={!separator.entry() || separator.locked() || separator.saving()} onChange={event => { const next = event.currentTarget.value; if (isRecipientSeparator(next)) void separator.set(next); }}><option value="comma">{t('settings-recipient-separator-comma')}</option><option value="semicolon">{t('settings-recipient-separator-semicolon')}</option></select><LockNote locked={separator.locked()}/><Show when={separator.error()}><p role="alert">{separator.error()}</p></Show><div class={css.actions}><button type="button" class={css.ghost} disabled={!separator.entry() || separator.locked() || separator.saving()} onClick={() => void separator.reset()}>{t('reader-inherit')}</button></div></div>
    <div class={css.field}><label for="header-collapse">{t('settings-header-collapse')}</label><select id="header-collapse" class={css.select} style={{'max-inline-size':'24rem'}} value={headerCollapse.value()} disabled={!headerCollapse.entry() || headerCollapse.locked() || headerCollapse.saving()} aria-describedby="header-collapse-help" onChange={event => { const next = event.currentTarget.value; if (isHeaderCollapseMode(next)) void headerCollapse.set(next); }}>{HEADER_COLLAPSE_MODES.map(mode => <option value={mode}>{t(`settings-header-collapse-${mode}`)}</option>)}</select><p id="header-collapse-help" class={css.meta}>{t('settings-header-collapse-help')}</p><LockNote locked={headerCollapse.locked()}/>
      <Show when={headerCollapse.error()}><p role="alert">{headerCollapse.error()}</p></Show>
      <div class={css.actions}><button type="button" class={css.ghost} disabled={!headerCollapse.entry() || headerCollapse.locked() || headerCollapse.saving()} onClick={() => void headerCollapse.reset()}>{t('reader-inherit')}</button></div></div>
    <p id="check-policy-help" class={css.meta}>{t('settings-check-help')}</p>
    {CHECK_ROWS.map(row => <CheckPolicyField settingKey={row.key} id={row.id} label={row.label}/>)}
    <div class={css.field}><label for="quote-style">{t('settings-quote-style')}</label><select id="quote-style" class={css.select} value={quote.value()} disabled={disabled()} onChange={event => void quote.set(event.currentTarget.value as 'classic'|'outlook')} aria-describedby="quote-style-help">
      <option value="classic">{t('settings-quote-classic')}</option><option value="outlook">{t('settings-quote-outlook')}</option>
    </select><p id="quote-style-help" class={css.meta}>{t('settings-quote-help')}</p><LockNote locked={quote.locked()}/>
    <Show when={quote.error()}><p role="alert">{quote.error()}</p></Show>
    <div class={css.actions}><button type="button" class={css.ghost} disabled={disabled()} onClick={() => void quote.reset()}>{t('reader-inherit')}</button></div></div>
    <div class={css.field}><label for="external-warning">{t('settings-external-warning')}</label>
      <select id="external-warning" class={css.select} value={warning.value()} disabled={!warning.entry() || warning.locked() || warning.saving()} onChange={event => void warning.set(event.currentTarget.value as 'off'|'warn'|'confirm'|'warn-remember'|'deny')} aria-describedby="external-warning-help">
        <option value="off">{t('settings-external-off')}</option><option value="warn">{t('settings-external-warn')}</option><option value="confirm">{t('settings-external-confirm')}</option><option value="warn-remember">{t('settings-external-warn-remember')}</option><option value="deny">{t('settings-external-deny')}</option>
      </select><p id="external-warning-help" class={css.meta}>{t('settings-external-help')}</p><LockNote locked={warning.locked()}/>
      <Show when={warning.error()}><p role="alert">{warning.error()}</p></Show>
      <div class={css.actions}><button type="button" class={css.ghost} disabled={!warning.entry() || warning.locked() || warning.saving()} onClick={() => void warning.reset()}>{t('reader-inherit')}</button></div>
    </div>
    <div class={css.field}><label for="trusted-domains">{t('settings-trusted-domains')}</label>
      <textarea id="trusted-domains" class={css.textarea} value={domainText()} disabled={domainsDisabled()} onInput={event => setDomainText(event.currentTarget.value)} aria-describedby="trusted-domains-help"/>
      <p id="trusted-domains-help" class={css.meta}>{t('settings-trusted-domains-help')}</p><LockNote locked={domains.locked()}/>
      <Show when={domainError() || domains.error()}><p role="alert">{domainError() || domains.error()}</p></Show>
      <div class={css.actions}><button type="button" class={css.button} disabled={domainsDisabled()} onClick={() => void saveDomains()}>{t('settings-trusted-domains-save')}</button>
      <button type="button" class={css.ghost} disabled={domainsDisabled()} onClick={() => void domains.reset()}>{t('reader-inherit')}</button></div>
    </div>
    <div class={css.field}><label for="reply-images">{t('settings-reply-images')}</label>
      <select id="reply-images" class={css.select} value={imagePolicy.value()} disabled={!imagePolicy.entry() || imagePolicy.locked() || imagePolicy.saving()} onChange={event => void imagePolicy.set(event.currentTarget.value as ReplyImagePolicy)} aria-describedby="reply-images-help">
        <option value="ask">{t('settings-reply-images-ask')}</option><option value="allow">{t('settings-reply-images-allow')}</option><option value="trusted">{t('settings-reply-images-trusted')}</option><option value="deny">{t('settings-reply-images-deny')}</option>
      </select><p id="reply-images-help" class={css.meta}>{t('settings-reply-images-help')}</p><LockNote locked={imagePolicy.locked()}/>
      <Show when={imagePolicy.error()}><p role="alert">{imagePolicy.error()}</p></Show>
      <div class={css.actions}><button type="button" class={css.ghost} disabled={!imagePolicy.entry() || imagePolicy.locked() || imagePolicy.saving()} onClick={() => void imagePolicy.reset()}>{t('reader-inherit')}</button></div>
    </div>
    <div class={css.field}><label for="reply-image-domains">{t('settings-reply-image-domains')}</label>
      <textarea id="reply-image-domains" class={css.textarea} value={imageDomainText()} disabled={imageDomainsDisabled()} onInput={event => setImageDomainText(event.currentTarget.value)} aria-describedby="reply-image-domains-help"/>
      <p id="reply-image-domains-help" class={css.meta}>{t('settings-reply-image-domains-help')}</p><LockNote locked={imageDomains.locked()}/>
      <Show when={imageDomainError() || imageDomains.error()}><p role="alert">{imageDomainError() || imageDomains.error()}</p></Show>
      <div class={css.actions}><button type="button" class={css.button} disabled={imageDomainsDisabled()} onClick={() => void saveImageDomains()}>{t('settings-reply-image-domains-save')}</button>
      <button type="button" class={css.ghost} disabled={imageDomainsDisabled()} onClick={() => void imageDomains.reset()}>{t('reader-inherit')}</button></div>
    </div>
    <ComposeDefaults/>
    <ResponseTemplatePreferences/>
  </section>;
}
