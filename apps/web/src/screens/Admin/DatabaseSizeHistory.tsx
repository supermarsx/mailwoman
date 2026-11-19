import {createMemo,createSignal,For,Show,type JSX} from 'solid-js';
import {Icon} from '../../components/Icon.tsx';
import {loadCatalog,t} from '../../i18n';
import {formatDateTime} from '../../i18n/format.ts';
import type {DatabaseReport} from '../../api/admin/database.ts';
import {formatBytes,formatDuration} from './Dashboard/format.ts';
import {ChartCard} from './shell/ChartCard.tsx';
import {LineChart} from './shell/LineChart.tsx';
import {SeriesCard} from './shell/SeriesCard.tsx';
import * as css from './admin.css.ts';
import * as history from './databaseSizeHistory.css.ts';

type Sample=NonNullable<DatabaseReport['growth']>['samples'][number];
export function DatabaseSizeHistory(props:{samples:readonly Sample[]}):JSX.Element{
 void loadCatalog('admin-database');void loadCatalog('admin-dashboard');
 const [range,setRange]=createSignal(3600);
 const samples=createMemo(()=>{
  const ordered=[...props.samples].filter(p=>Number.isFinite(p.at)).sort((a,b)=>a.at-b.at);
  const last=ordered.at(-1)?.at??0;
  return range()===0?ordered:ordered.filter(p=>p.at>=last-range());
 });
 const time=(at:number)=>formatDateTime(new Date(at*1000),{dateStyle:'short',timeStyle:'short'});
 const size=(value:number|undefined)=>typeof value==='number'?formatBytes(value):t('admin-shell-not-available');
 const summary=()=>samples().length?t('admin-db-history-period',{from:time(samples()[0]!.at),to:time(samples().at(-1)!.at)}):t('admin-db-growth-warming');
 return <section class={css.section} aria-label={t('admin-db-growth-history')}>
  <div class={css.headingRow}><h3 class={css.heading}>{t('admin-db-growth-history')}</h3><div class={css.headingRow} role="group" aria-label={t('admin-db-history-range')}><For each={[900,3600,21600,0]}>{seconds=><button class={range()===seconds?`btn ${history.selectedRange}`:'btn btn--ghost'} type="button" aria-pressed={range()===seconds} onClick={()=>setRange(seconds)}><Show when={range()===seconds}><Icon name="check"/></Show>{seconds===0?t('admin-db-history-all'):formatDuration(seconds)}</button>}</For></div></div>
  <p class={css.note}>{t('admin-db-history-help')}</p>
  <div class={history.graphs}>
   <ChartCard title={t('admin-db-file')} testId="database-size-history" summary={summary()}
    legend={[{label:t('admin-db-file'),slot:1},{label:t('admin-db-occupied'),slot:2}]}
    empty={samples().length?undefined:t('admin-db-growth-warming')}
    table={{columns:[t('admin-shell-chart-col-time'),t('admin-db-file'),t('admin-db-occupied')],numeric:[false,true,true],rows:samples().map(p=>[time(p.at),size(p.dbBytes),size(p.occupiedBytes)])}}>
    <LineChart timeScale fitY times={samples().map(p=>p.at)} series={[{label:t('admin-db-file'),slot:1,values:samples().map(p=>p.dbBytes)},{label:t('admin-db-occupied'),slot:2,values:samples().map(p=>p.occupiedBytes??null)}]} format={formatBytes} formatTime={time}/>
   </ChartCard>
   <SeriesCard timeScale title={t('admin-db-wal')} testId="database-wal-history" points={samples().map(p=>({t:p.at,value:p.walBytes}))} format={formatBytes} formatTime={time} about={summary()}/>
  </div>
 </section>;
}
