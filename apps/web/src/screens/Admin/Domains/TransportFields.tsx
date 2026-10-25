import { For, Show, type JSX } from 'solid-js';
import type { ConnectionSecurity, DomainEndpoint } from '../../../api/admin/domains.ts';
import { t } from '../../../i18n';
import * as css from '../../Settings/styles.css.ts';

export function TransportFields(props: { value: DomainEndpoint; ready: boolean; update: (value: DomainEndpoint) => void }): JSX.Element {
  const security = (): ConnectionSecurity => props.value.security ?? { mode: 'fixed', allowed: [props.value.tls], allowPlaintext: props.value.tls === 'none' };
  const change = (patch: Partial<ConnectionSecurity>) => props.update({ ...props.value, security: { ...security(), ...patch } });
  function text(key: 'serverName' | 'caPem', value: string): void {
    const next = { ...security() };
    if (value) next[key] = value; else delete next[key];
    props.update({ ...props.value, security: next });
  }
  return <fieldset class={css.section} disabled={!props.ready}><legend class={css.heading}>{t('backend-security-title')}</legend>
    <label class={css.field}><input type="checkbox" checked={!!props.value.security} onChange={event => { const next = { ...props.value }; if (event.currentTarget.checked) next.security = security(); else delete next.security; props.update(next); }} title={t('backend-security-customize-help')} /><span>{t('backend-security-customize')}</span></label><p class={css.meta}>{t('backend-security-customize-help')}</p>
    <Show when={!props.ready}><p class={css.meta}>{t('backend-editor-advanced-unavailable')}</p></Show>
    <Show when={props.value.security}>
      <label class={css.field}>{t('backend-security-mode')}<select class={css.select} value={security().mode} onChange={event => change({ mode: event.currentTarget.value as ConnectionSecurity['mode'], ...(event.currentTarget.value === 'fixed' ? { allowed: security().allowed.slice(0, 1) } : {}) })} title={t('backend-security-mode-help')}><option value="fixed">{t('backend-security-fixed')}</option><option value="auto">{t('backend-security-auto')}</option></select><small>{t('backend-security-mode-help')}</small></label>
      <For each={security().mode === 'auto' ? [0, 1, 2] : [0]}>{index => <label class={css.field}>{t('backend-security-order', { index: index + 1 })}<select class={css.select} required={index === 0} value={security().allowed[index] ?? ''} onChange={event => { const next = [...security().allowed]; if (!event.currentTarget.value) next.splice(index, 1); else next[index] = event.currentTarget.value as DomainEndpoint['tls']; change({ allowed: next.filter(Boolean) }); }} title={t('backend-security-order-help')}><Show when={index > 0}><option value="">{t('backend-security-unused')}</option></Show><option value="implicit">TLS</option><option value="starttls">STARTTLS</option><option value="none" disabled={!security().allowPlaintext}>{t('backend-editor-plaintext')}</option></select><small>{t('backend-security-order-help')}</small></label>}</For>
      <label class={css.field}><input type="checkbox" checked={security().allowPlaintext} onChange={event => change({ allowPlaintext: event.currentTarget.checked })} title={t('backend-security-plaintext-help')} /><span>{t('backend-security-plaintext')}</span></label><p class={css.meta}>{t('backend-security-plaintext-help')}</p>
      <label class={css.field}>{t('backend-security-server-name')}<input class={css.input} maxlength="253" value={security().serverName ?? ''} onInput={event => text('serverName', event.currentTarget.value)} title={t('backend-security-server-name-help')} /><small>{t('backend-security-server-name-help')}</small></label>
      <label class={css.field}>{t('backend-security-ca')}<textarea class={css.input} rows="6" maxlength="65536" value={security().caPem ?? ''} onInput={event => text('caPem', event.currentTarget.value)} title={t('backend-security-ca-help')} /><small>{t('backend-security-ca-help')}</small></label>
      <p class={css.meta}>{t('backend-security-verify')}</p>
    </Show>
  </fieldset>;
}
