// Trusted link domains (t39 R7): links to these domains open from Review links
// without the destination step. An administrator can set the list and the
// subdomain switch as defaults and lock either; a locked control is read-only.
import { createEffect, createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { normalizeTrustedLinkDomain } from '../../viewers/linkTrust.ts';
import {normalizeTrustedLinkSender} from '../../viewers/trustedLinkSenders.ts';
import { useSetting } from './useSetting.ts';
import { t } from '../../i18n';
import { SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';
import * as trustCss from './trustedLinkDomains.css.ts';

const isList = (v: unknown): v is string[] => Array.isArray(v) && v.every(item => typeof item === 'string');

export function TrustedLinkDomains(): JSX.Element {
  const domains = useSetting<string[]>('links.trustedDomains', [], isList);
  const subdomains = useSetting<boolean>('links.trustedSubdomains', false);
  const senders = useSetting<string[]>('links.trustedSenders', [], isList);
  const [text, setText] = createSignal('');
  const [invalid, setInvalid] = createSignal('');
  const [senderText,setSenderText]=createSignal(''),[senderInvalid,setSenderInvalid]=createSignal('');
  const saved = createMemo(() => domains.value().join('\n'));
  createEffect(() => setText(saved()));
  createEffect(()=>setSenderText(senders.value().join('\n')));
  const listDisabled = () => !domains.entry() || domains.locked() || domains.saving();
  const subDisabled = () => !subdomains.entry() || subdomains.locked() || subdomains.saving();
  const sendersDisabled=()=>!senders.entry()||senders.locked()||senders.saving();
  async function saveSenders():Promise<void>{
    const entries=senderText().split(/\r?\n/).filter(value=>value.trim()!=='');
    const bad=entries.filter(value=>normalizeTrustedLinkSender(value)===null);
    if(entries.length>100||bad.length){setSenderInvalid(t('privacy-trusted-link-senders-invalid',{entries:bad.slice(0,3).join(', ')}));return;}
    setSenderInvalid('');await senders.set([...new Set(entries.map(value=>normalizeTrustedLinkSender(value)!))]);
  }
  async function save(): Promise<void> {
    const entries = text().split(/[\n,;]+/).map(value => value.trim()).filter(Boolean);
    const bad = entries.filter(value => normalizeTrustedLinkDomain(value) === null);
    if (entries.length > 100 || bad.length > 0) { setInvalid(t('privacy-trusted-links-invalid', { entries: bad.slice(0, 3).join(', ') })); return; }
    setInvalid('');
    await domains.set([...new Set(entries.map(value => normalizeTrustedLinkDomain(value)!))]);
  }
  return <>
    <SettingsCard id="privacy-card-trusted-links" testId="trusted-link-domains" title={t('privacy-trusted-links-title')} description={t('privacy-trusted-links-help')}>
      <SettingRow controlId="trusted-link-domains" stacked label={t('privacy-trusted-links-domains')} help={t('privacy-trusted-links-domains-help')} handle={domains}>{describedBy => <>
        <textarea id="trusted-link-domains" class={css.textarea} value={text()} disabled={listDisabled()} aria-invalid={invalid() ? 'true' : undefined} aria-describedby={describedBy()} onInput={event => setText(event.currentTarget.value)} />
        <Show when={invalid()}><p role="alert">{invalid()}</p></Show>
        <div class={css.actions}><button type="button" class={css.button} disabled={listDisabled()} onClick={() => void save()}>{t('privacy-trusted-links-save')}</button></div>
      </>}</SettingRow>
      <SettingRow controlId="trusted-link-subdomains" label={t('privacy-trusted-links-subdomains')} help={t('privacy-trusted-links-subdomains-help')} handle={subdomains}>{describedBy =>
        <input id="trusted-link-subdomains" type="checkbox" checked={subdomains.value()} disabled={subDisabled()} aria-describedby={describedBy()} onChange={event => void subdomains.set(event.currentTarget.checked)} />
      }</SettingRow>
    </SettingsCard>
    <SettingsCard id="privacy-card-trusted-senders" testId="trusted-link-senders" title={t('privacy-trusted-link-senders-title')} description={t('privacy-trusted-link-senders-help')}>
      <SettingRow controlId="trusted-link-senders" stacked label={t('privacy-trusted-link-senders-list')} help={t('privacy-trusted-link-senders-list-help')} handle={senders}>{describedBy => <>
        <textarea id="trusted-link-senders" class={css.textarea} value={senderText()} disabled={sendersDisabled()} aria-invalid={senderInvalid()?'true':undefined} aria-describedby={describedBy()} onInput={event=>setSenderText(event.currentTarget.value)}/>
        <Show when={senderInvalid()}><p role="alert">{senderInvalid()}</p></Show>
        <div class={css.actions}><button type="button" class={css.button} disabled={sendersDisabled()} onClick={()=>void saveSenders()}>{t('privacy-trusted-link-senders-save')}</button></div>
      </>}</SettingRow>
      <Show when={senders.value().length}><ul class={trustCss.senders}><For each={senders.value()}>{sender=><li class={trustCss.sender}><span dir="ltr" class={trustCss.address}>{sender}</span> <button type="button" class={css.ghost} disabled={sendersDisabled()} onClick={()=>void senders.set(senders.value().filter(value=>value!==sender))}>{t('privacy-trusted-link-senders-remove',{sender})}</button></li>}</For></ul></Show>
    </SettingsCard>
  </>;
}
