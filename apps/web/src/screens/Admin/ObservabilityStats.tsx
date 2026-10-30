// Admin › Observability › Statistics (t106 layout).
//
// One period control at the top applies to every figure, chart and table that
// covers a period. Below it three sections: key figures (stat tiles), charts
// (one card each) and tables. Figures that are a reading of "now" (accounts,
// sessions, storage, outbox) say so in their labels and reload with the page's
// refresh. Loading keeps the page in place: skeletons the size of the final
// cards on the first load, the last render dimmed on later ones (t75).

import {createResource,For,Show,onMount,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useShell} from './shell/context.ts';
import {resourceValue} from '../../components/resourceValue.ts';
import {createRequestHistory,HistoryNotes,PeriodPicker,TrafficCharts,TrafficTables,TrafficTiles} from './RequestHistory.tsx';
import {StatTile} from './shell/StatTile.tsx';
import {BarRows} from './shell/BarChart.tsx';
import {ChartCard,StatsSection,TileSkeleton} from './shell/ChartCard.tsx';
import {formatBytes,formatCount,formatMs} from './Dashboard/format.ts';
import * as css from './Dashboard/dashboard.css.ts';
import * as stats from './shell/stats.css.ts';
import {num,obj,type DashboardHealth} from '../../api/admin/health.ts';

/** Messages not yet sent: due, retrying or being sent (t103 outbox probe). */
function outboxWaiting(h: DashboardHealth | null | undefined): number | null {
  const m = h?.components?.outbox?.status === 'disabled' ? undefined : h?.components?.outbox?.metrics;
  if (!m) return null;
  const parts = [num(m, 'queued'), num(m, 'deferred'), num(m, 'sending')];
  return parts.some((v) => v === null) ? null : parts.reduce<number>((a, v) => a + (v ?? 0), 0);
}

/** Sends that failed or ended with delivery unknown in the last hour. */
function outboxFailedHour(h: DashboardHealth | null | undefined): number | null {
  const m = h?.components?.outbox?.status === 'disabled' ? undefined : h?.components?.outbox?.metrics;
  if (!m) return null;
  const s = obj(m, 'sendsLastHour');
  const f = num(s, 'failed');
  const u = num(s, 'uncertain');
  return f === null || u === null ? null : f + u;
}

const ACCOUNT_KEYS=['provisionedUsers','accountsSignedIn24h','accountsSignedIn7d','domains','activeSessions','activeAdminSessions'] as const;

function Group(props:{title:string;children:JSX.Element}):JSX.Element {
 return <div class={stats.group} role="group" aria-label={props.title}><h4 class={stats.groupTitle}>{props.title}</h4><div class={stats.tiles}>{props.children}</div></div>;
}

export function ObservabilityStats():JSX.Element {
 const shell=useShell();
 onMount(()=>void loadCatalog('admin-stats'));
 const canDash=()=>shell.can.can('dashboard.read');
 const [result,{refetch}]=createResource(()=>canDash()||undefined,()=>shell.services.loadDashboard());
 const history=createRequestHistory();
 const dash=()=>{const value=resourceValue(result);return value?.kind==='ok'?value.data:undefined;};
 const dashFirstLoad=()=>canDash()&&result.loading&&dash()===undefined;
 const dashMissing=()=>canDash()&&!result.loading&&dash()===undefined;
 const busy=()=>history.loading()||result.loading;
 const stale=()=>busy()&&(history.data()!==undefined||dash()!==undefined);
 const unavailable=()=>t('admin-shell-not-available');
 const skeletons=(n:number)=><For each={Array.from({length:n})}>{()=><TileSkeleton/>}</For>;
 return <section class={css.page} aria-label={t('admin-obs-statistics')} aria-busy={busy()} data-testid="statistics">
  <div class={css.head}><h2>{t('admin-obs-statistics')}</h2></div>
  <PeriodPicker history={history} refreshLabel={t('admin-stats-refresh')} onRefresh={()=>{if(canDash())void refetch();}}/>
  <HistoryNotes history={history}/>
  <div class={`${stats.view} ${stale()?stats.stale:''}`}>
   <StatsSection kind="tiles" title={t('admin-stats-key-figures')} testId="stats-tiles">
    <Group title={t('admin-stats-group-traffic')}><TrafficTiles history={history}/></Group>
    <Show when={canDash()}>
     <Show when={!dashMissing()} fallback={<p role="status">{unavailable()}</p>}>
      <Group title={t('admin-obs-accounts')}>
       <Show when={dash()} fallback={skeletons(ACCOUNT_KEYS.length)}>{value=><For each={ACCOUNT_KEYS}>{key=><StatTile label={t(`admin-obs-count-${key}`)} value={value().counts[key]} format={formatCount}/>}</For>}</Show>
      </Group>
      <Group title={t('admin-dash-security')}>
       <Show when={dash()} fallback={skeletons(3)}>{value=><Show when={value().security} fallback={<p class={stats.cardUnit}>{unavailable()}</p>}>{security=><>
        <StatTile label={t('admin-dash-failed-webmail')} value={security().failedLogins24h.webmail} format={formatCount}/>
        <StatTile label={t('admin-dash-failed-admin')} value={security().failedLogins24h.admin} format={formatCount}/>
        <StatTile label={t('admin-dash-active-blocks')} value={security().activeBlocks} format={formatCount}/>
       </>}</Show>}</Show>
      </Group>
      <Group title={t('admin-stats-group-storage')}>
       <Show when={dash()} fallback={skeletons(6)}>{value=><>
        <StatTile label={t('admin-dash-db-size')} value={value().storage?.dbBytes??null} format={formatBytes}/>
        <StatTile label={t('admin-dash-wal-size')} value={value().storage?.walBytes??null} format={formatBytes}/>
        <StatTile label={t('admin-dash-index-size')} value={value().storage?.searchIndexBytes??null} format={formatBytes}/>
        <StatTile label={t('admin-obs-db-latency')} value={value().health?.dbPingMs??null} format={formatMs}/>
        <StatTile label={t('admin-obs-outbox-pending')} value={outboxWaiting(value().health)} format={formatCount}/>
        <StatTile label={t('admin-obs-outbox-failed-hour')} value={outboxFailedHour(value().health)} format={formatCount}/>
       </>}</Show>
      </Group>
     </Show>
    </Show>
   </StatsSection>
   <StatsSection kind="charts" title={t('admin-stats-charts')} testId="stats-charts">
    <div class={stats.charts}>
     <TrafficCharts history={history}/>
     <Show when={dash()?.blockedAccounts}>{blocked=>{
      const bars=()=>blocked().byDomain.map(row=>({label:row.domain,value:row.accounts}));
      return <ChartCard testId="chart-blocked" title={t('admin-dash-blocked-chart')} unit={t('admin-stats-unit-accounts')}
       summary={t('admin-stats-bars-about',{count:bars().length})}
       table={{columns:[t('admin-shell-chart-col-item'),t('admin-shell-chart-col-value')],numeric:[false,true],rows:bars().map(b=>[b.label,formatCount(b.value)])}}
       empty={bars().length===0?t('admin-stats-empty-blocked'):undefined}>
       <BarRows bars={bars()} format={formatCount}/>
      </ChartCard>;
     }}</Show>
    </div>
   </StatsSection>
   <StatsSection kind="tables" title={t('admin-stats-tables')} testId="stats-tables">
    <div class={stats.tables}><TrafficTables history={history}/></div>
   </StatsSection>
  </div>
  <Show when={dashFirstLoad()}><span class={stats.cardUnit} role="status">{t('admin-stats-loading')}</span></Show>
 </section>;
}
