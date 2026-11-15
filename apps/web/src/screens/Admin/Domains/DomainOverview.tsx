import {For,type JSX} from 'solid-js';
import {domainBackends,type DomainDetail} from '../../../api/admin/domains.ts';
import {t,formatNumber} from '../../../i18n';
import {Icon,type IconName} from '../../../components/Icon.tsx';
import * as css from '../admin.css.ts';
import * as tiles from '../protocolGuard.css.ts';
export function DomainOverview(props:{domain:DomainDetail}):JSX.Element{
 const backends=()=>domainBackends(props.domain.upstream);
 const metrics=()=>[
  {label:'domain-overview-accounts',value:props.domain.overview?.connectedAccounts??null,icon:'person'},
  {label:'backend-list-title',value:backends().length,icon:'mail'},
  {label:'backend-editor-enabled',value:backends().filter(row=>row.enabled!==false).length,icon:'check'},
  {label:'domain-editor-aliases',value:props.domain.aliases.length,icon:'link'},
  {label:'domain-editor-hosts',value:props.domain.hosts.length,icon:'globe'},
  {label:'domain-editor-allowlist',value:props.domain.allowlist.length,icon:'shield'},
  {label:'domain-editor-blocklist',value:props.domain.blocklist.length,icon:'block'},
 ];
 return <section class={css.card}><h3 class={css.heading}>{t('domain-page-overview')}</h3><p class={css.note}>{t('domain-editor-overview-help')}</p><div class={tiles.tiles}><For each={metrics()}>{item=><article class={tiles.tile}><span class={tiles.tileLabel}><Icon name={item.icon as IconName}/>{t(item.label)}</span><strong class={tiles.tileValue}>{item.value===null?'—':formatNumber(item.value)}</strong></article>}</For></div><dl class={css.note}><dt>{t('domain-editor-enabled')}</dt><dd>{t(props.domain.enabled?'domain-editor-enabled':'domain-editor-disabled')}</dd><dt>{t('backend-editor-protocol')}</dt><dd>{[...new Set(backends().map(row=>row.protocol.toUpperCase()))].join(', ')||t('backend-list-none')}</dd></dl></section>;
}
