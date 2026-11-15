import { For, Show, onMount, type JSX } from 'solid-js';
import { domainBackends, type DomainDetail } from '../../../api/admin/domains.ts';
import { loadCatalog, t } from '../../../i18n';
import * as css from '../../Settings/styles.css.ts';
import * as table from '../../Settings/managementTable.css.ts';
import * as header from '../../Settings/linkedAccounts.css.ts';
import {Icon} from '../../../components/Icon.tsx';

export function BackendList(props: { domain: DomainDetail; editable: boolean; open: (id: string) => void; setFailover?: (on: boolean) => void }): JSX.Element {
  onMount(() => void loadCatalog('admin-domain-backends'));
  const rows = () => domainBackends(props.domain.upstream);
  const selected = (role: 'mail' | 'submission' | 'rules') => rows().filter(row => row.role === role && row.enabled !== false).sort((a, b) => Number(!!b.preferred) - Number(!!a.preferred) || (a.priority ?? 100) - (b.priority ?? 100) || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0))[0]?.id;
  return <section class={css.section} aria-label={t('backend-list-title')}>
    <div class={header.header}><div class={header.headerText}><h4 class={css.heading}>{t('backend-list-title')}</h4><p class={css.prose}>{t('backend-list-help')}</p></div><Show when={props.editable}><button type="button" class={`${css.button} ${header.addButton}`} onClick={()=>props.open('new')}><Icon name="plus"/>{t('backend-list-add')}</button></Show></div>
    <Show when={props.domain.upstream.version!==2}><p class={css.meta}>{t('backend-list-upgrade')}</p></Show>
    <Show when={props.domain.upstream.version===2}><label class={css.check}><input type="checkbox" checked={props.domain.upstream.failover===true} disabled={!props.editable||!props.setFailover} onChange={event=>props.setFailover?.(event.currentTarget.checked)}/><span>{t('backend-list-failover')}<p class={css.meta}>{t('backend-list-failover-help')}</p></span></label></Show>
    <div class={table.scroll}><table class={table.table}><thead><tr><th class={table.heading}>{t('backend-editor-id')}</th><th class={table.heading}>{t('backend-editor-protocol')}</th><th class={table.heading}>{t('backend-editor-connection')}</th><th class={table.heading}>{t('backend-editor-priority')}</th><th class={table.heading}>{t('backend-editor-enabled')}</th><th class={table.heading}>{t('backend-list-edit')}</th></tr></thead><tbody><For each={rows()}>{row=><tr class={table.row} data-current={selected(row.role)===row.id}>
     <td class={table.cell}><strong>{row.id}</strong><span class={table.text}>{t(`backend-list-role-${row.role}`)}</span></td>
     <td class={table.cell}><span class={table.name}><Icon name={row.protocol==='smtp'?'mail':row.protocol==='sieve'?'filter':'mail'}/>{row.protocol.toUpperCase()}</span><span class={table.text}>{row.ews?.authentication.toUpperCase()??row.endpoint?.tls.toUpperCase()??'HTTPS'}</span></td>
     <td class={table.cell}>{row.endpoint?`${row.endpoint.host}:${row.endpoint.port}`:row.ews?.endpointUrl??row.sessionUrl}</td><td class={table.cell}>{row.priority??100}</td>
     <td class={table.cell}><span class={table.name}><Icon name={row.enabled===false?'eyeOff':selected(row.role)===row.id?'check':'mail'}/>{row.enabled===false?t('domain-editor-disabled'):selected(row.role)===row.id?t('backend-list-selected'):t('domain-editor-enabled')}</span></td>
     <td class={table.cell}><Show when={props.editable}><button type="button" class="btn btn--ghost" aria-label={`${t('backend-list-edit')} ${row.id}`} title={t('backend-list-edit')} onClick={()=>props.open(row.id)}><Icon name="edit"/></button></Show></td>
    </tr>}</For></tbody></table></div><Show when={rows().length===0}><p class={css.meta}>{t('backend-list-none')}</p></Show>
  </section>;
}
