import {Show,type JSX} from 'solid-js';
import type { DomainBackend } from '../../../api/admin/domains.ts';
import { t } from '../../../i18n';
import * as css from '../../Settings/styles.css.ts';

export function EwsFields(props: { value: NonNullable<DomainBackend['ews']>; change: (value: NonNullable<DomainBackend['ews']>) => void }): JSX.Element {
  return <div class={css.stack}>
    <label class={css.field}>{t('backend-ews-endpoint')}<input class={css.input} type="url" required maxlength="2048" value={props.value.endpointUrl} onInput={event => props.change({ ...props.value, endpointUrl: event.currentTarget.value })} /><small>{t('backend-ews-endpoint-help')}</small></label>
    <label class={css.field}>{t('backend-ews-auth')}<select class={css.select} value={props.value.authentication} onChange={event=>props.change({...props.value,authentication:event.currentTarget.value as 'basic'|'ntlm'})}><option value="basic">{t('backend-ews-basic')}</option><option value="ntlm">NTLMv2</option></select><small>{t('backend-ews-auth-help')}</small></label>
    <Show when={props.value.authentication==='ntlm'}>
      <label class={css.field}>{t('backend-ews-domain')}<input class={css.input} required maxLength={128} pattern="[A-Za-z0-9._-]{1,128}" value={props.value.ntlmDomain??''} onInput={event=>props.change({...props.value,ntlmDomain:event.currentTarget.value})}/><small>{t('backend-ews-domain-help')}</small></label>
      <label class={css.field}>{t('backend-ews-workstation')}<input class={css.input} maxLength={64} pattern="[A-Za-z0-9._-]{0,64}" value={props.value.workstation??''} onInput={event=>props.change({...props.value,workstation:event.currentTarget.value})}/><small>{t('backend-ews-workstation-help')}</small></label>
    </Show>
    <label class={css.field}>{t('backend-ews-ca')}<textarea class={css.input} rows="5" maxlength="65536" spellcheck={false} value={props.value.caPem ?? ''} onInput={event => { const next = { ...props.value }; if (event.currentTarget.value) next.caPem = event.currentTarget.value; else delete next.caPem; props.change(next); }} /><small>{t('backend-ews-ca-help')}</small></label>
    <p class={css.meta}>{t('backend-ews-routing-help')}</p>
    <p class={css.meta}>{t('backend-ews-diagnostics-help')}</p>
  </div>;
}
