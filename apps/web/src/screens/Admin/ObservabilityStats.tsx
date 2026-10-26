import {createResource,For,Show,type JSX} from 'solid-js';
import {t} from '../../i18n';
import {useShell} from './shell/context.ts';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {RequestHistory} from './RequestHistory.tsx';
import {BarChart} from './shell/BarChart.tsx';
import {StatTile} from './shell/StatTile.tsx';
import {formatMs,formatBytes} from './Dashboard/format.ts';
import * as css from './Dashboard/dashboard.css.ts';
import * as admin from './admin.css.ts';
export function ObservabilityStats():JSX.Element {
 const shell=useShell();
 const [result,{refetch}]=createResource(()=>shell.can.can('dashboard.read'),()=>shell.services.loadDashboard());
 const data=()=>{if(result.state==='pending'||result.state==='unresolved'||result.error)return undefined;const value=result.latest;return value?.kind==='ok'?value.data:undefined;};
 const unavailable=()=>t('admin-shell-not-available');
 return <section class={css.page} aria-label={t('admin-obs-statistics')}><div class={css.head}><h2>{t('admin-obs-statistics')}</h2><Show when={shell.can.can('dashboard.read')}><button class="btn" disabled={result.loading} onClick={()=>void refetch()}>{t('admin-dash-refresh')}</button></Show></div>
 <section class={admin.card}><RequestHistory refreshToken={data()}/></section>
 <Show when={shell.can.can('dashboard.read')}><Show when={!result.loading||data()} fallback={<AsyncPending variant="page"/>}><Show when={data()} fallback={<p role="status">{unavailable()}</p>}>{value=><>
 <section class={admin.card}><h3>{t('admin-obs-accounts')}</h3><div class={css.charts}>
 <For each={['provisionedUsers','accountsSignedIn24h','accountsSignedIn7d','domains','activeSessions','activeAdminSessions'] as const}>{key=><StatTile label={t(`admin-obs-count-${key}`)} value={value().counts[key]}/>}</For>
 </div></section>
 <section class={admin.card}><h3>{t('admin-dash-security')}</h3><Show when={value().security} fallback={<p>{unavailable()}</p>}>{security=><div class={css.charts}><StatTile label={t('admin-dash-failed-webmail')} value={security().failedLogins24h.webmail}/><StatTile label={t('admin-dash-failed-admin')} value={security().failedLogins24h.admin}/><StatTile label={t('admin-dash-active-blocks')} value={security().activeBlocks}/></div>}</Show><Show when={value().blockedAccounts}>{blocked=><BarChart title={t('admin-dash-blocked-chart')} bars={blocked().byDomain.map(row=>({label:row.domain,value:row.accounts}))}/>}</Show></section>
 <section class={admin.card}><h3>{t('admin-dash-storage')}</h3><div class={css.charts}><StatTile label={t('admin-dash-db-size')} value={value().storage?.dbBytes??null} format={formatBytes}/><StatTile label={t('admin-dash-wal-size')} value={value().storage?.walBytes??null} format={formatBytes}/><StatTile label={t('admin-dash-index-size')} value={value().storage?.searchIndexBytes??null} format={formatBytes}/><StatTile label={t('admin-obs-db-latency')} value={value().health?.dbPingMs??null} format={formatMs}/><StatTile label={t('admin-obs-outbox-pending')} value={value().health?.outbox?.pending??null}/><StatTile label={t('admin-obs-outbox-failed')} value={value().health?.outbox?.failed24h??null}/></div></section>
 </>}</Show></Show></Show></section>;
}
