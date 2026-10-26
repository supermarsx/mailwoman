import { createEffect, createMemo, createSignal, Show, type JSX } from 'solid-js';
import { normalizeTrustedDomain } from '../../components/compose/recipientTrust.ts';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { t } from '../../i18n';
import * as css from './styles.css.ts';

export function ComposePreferences(): JSX.Element {
  const quote = useSetting<'classic'|'outlook'>('mail.quoteStyle', 'classic', (v): v is 'classic'|'outlook' => v === 'classic' || v === 'outlook');
  const disabled = () => !quote.entry() || quote.locked() || quote.saving();
  const showFrom = useSetting<boolean>('mail.showFrom', false);
  const quick = useSetting<boolean>('mail.quickReply', false);
  const warning = useSetting<'off'|'warn'|'confirm'>('mail.externalRecipientWarning', 'warn', (v): v is 'off'|'warn'|'confirm' => v === 'off' || v === 'warn' || v === 'confirm');
  const domains = useSetting<string[]>('mail.trustedRecipientDomains', [], (v): v is string[] => Array.isArray(v) && v.every(item => typeof item === 'string'));
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
    <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={showFrom.value()} disabled={!showFrom.entry() || showFrom.locked() || showFrom.saving()} onChange={event => void showFrom.set(event.currentTarget.checked)} aria-describedby="show-from-help"/>{t('settings-show-from')}</label><p id="show-from-help" class={css.meta}>{t('settings-show-from-help')}</p><LockNote locked={showFrom.locked()}/><Show when={showFrom.error()}><p role="alert">{showFrom.error()}</p></Show><div class={css.actions}><button type="button" class={css.button} disabled={!showFrom.entry() || showFrom.locked() || showFrom.saving()} onClick={() => void showFrom.reset()}>{t('reader-inherit')}</button></div></div>
    <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={quick.value()} disabled={!quick.entry() || quick.locked() || quick.saving()} onChange={event => void quick.set(event.currentTarget.checked)} aria-describedby="quick-reply-help"/>{t('mail-quick-reply')}</label><p id="quick-reply-help" class={css.meta}>{t('mail-quick-reply-help')}</p><LockNote locked={quick.locked()}/><Show when={quick.error()}><p role="alert">{quick.error()}</p></Show><div class={css.actions}><button type="button" class={css.button} disabled={!quick.entry() || quick.locked() || quick.saving()} onClick={() => void quick.reset()}>{t('reader-inherit')}</button></div></div>
    <div class={css.field}><label for="quote-style">{t('settings-quote-style')}</label><select id="quote-style" class={css.select} value={quote.value()} disabled={disabled()} onChange={event => void quote.set(event.currentTarget.value as 'classic'|'outlook')} aria-describedby="quote-style-help">
      <option value="classic">{t('settings-quote-classic')}</option><option value="outlook">{t('settings-quote-outlook')}</option>
    </select><p id="quote-style-help" class={css.meta}>{t('settings-quote-help')}</p><LockNote locked={quote.locked()}/>
    <Show when={quote.error()}><p role="alert">{quote.error()}</p></Show>
    <div class={css.actions}><button type="button" class={css.button} disabled={disabled()} onClick={() => void quote.reset()}>{t('reader-inherit')}</button></div></div>
    <div class={css.field}><label for="external-warning">{t('settings-external-warning')}</label>
      <select id="external-warning" class={css.select} value={warning.value()} disabled={!warning.entry() || warning.locked() || warning.saving()} onChange={event => void warning.set(event.currentTarget.value as 'off'|'warn'|'confirm')} aria-describedby="external-warning-help">
        <option value="off">{t('settings-external-off')}</option><option value="warn">{t('settings-external-warn')}</option><option value="confirm">{t('settings-external-confirm')}</option>
      </select><p id="external-warning-help" class={css.meta}>{t('settings-external-help')}</p><LockNote locked={warning.locked()}/>
      <Show when={warning.error()}><p role="alert">{warning.error()}</p></Show>
      <div class={css.actions}><button type="button" class={css.button} disabled={!warning.entry() || warning.locked() || warning.saving()} onClick={() => void warning.reset()}>{t('reader-inherit')}</button></div>
    </div>
    <div class={css.field}><label for="trusted-domains">{t('settings-trusted-domains')}</label>
      <textarea id="trusted-domains" class={css.textarea} value={domainText()} disabled={domainsDisabled()} onInput={event => setDomainText(event.currentTarget.value)} aria-describedby="trusted-domains-help"/>
      <p id="trusted-domains-help" class={css.meta}>{t('settings-trusted-domains-help')}</p><LockNote locked={domains.locked()}/>
      <Show when={domainError() || domains.error()}><p role="alert">{domainError() || domains.error()}</p></Show>
      <div class={css.actions}><button type="button" class={css.button} disabled={domainsDisabled()} onClick={() => void saveDomains()}>{t('settings-trusted-domains-save')}</button>
      <button type="button" class={css.button} disabled={domainsDisabled()} onClick={() => void domains.reset()}>{t('reader-inherit')}</button></div>
    </div>
  </section>;
}
