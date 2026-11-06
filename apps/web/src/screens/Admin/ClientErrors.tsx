// Admin > Observability > Client errors.
//
// Errors the web app caught in people's browsers (error boundaries, window
// errors, rejected promises, failed chunks, workers, CSP violations), grouped by
// fingerprint. The list shows what is failing and how often; a group opens to the
// stack trace with the build's source maps applied (application frames
// highlighted, vendor frames folded), a breakdown by browser, OS, build and route,
// and the latest occurrences. Reports carry no message content, address, token or
// URL query (docs/admin/observability.md).

import {createMemo,createResource,createSignal,For,onMount,Show,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {formatDateTime} from '../../i18n/format.ts';
import {useShell} from './shell/context.ts';
import {sectionPath} from './shell/router.ts';
import {resourceValue} from '../../components/resourceValue.ts';
import {ChartCard} from './shell/ChartCard.tsx';
import {BarRows} from './shell/BarChart.tsx';
import {LineChart} from './shell/LineChart.tsx';
import {ConfirmDialog} from './shell/ConfirmDialog.tsx';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {formatCount} from './Dashboard/format.ts';
import {deleteClientError,listClientErrors,loadClientError,setClientErrorStatus,type ClientErrorDetail,type ClientErrorGroup,type ClientErrorStatus} from '../../api/admin/clientErrors.ts';
import * as admin from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
import * as css from './clientErrors.css.ts';

const RANGES:readonly [string,number][]=[['1h',1],['24h',24],['7d',168],['30d',720],['90d',2160]];
const STATUSES:readonly ClientErrorStatus[]=['open','resolved','ignored'];
const when=(seconds:number)=>formatDateTime(seconds*1000);

/** Split a stack into message lines and frames; a frame is an application frame when it resolves into `src/`. */
export function splitStack(text:string):{head:string[];frames:{text:string;app:boolean}[]} {
 const head:string[]=[];const frames:{text:string;app:boolean}[]=[];
 for(const line of text.split('\n')){
  const isFrame=/^\s+at\s/.test(line)||/@\S+:\d+:\d+\s*$/.test(line);
  if(!isFrame){if(frames.length===0)head.push(line);continue;}
  frames.push({text:line.trim(),app:/(^|[\s(@])src\//.test(line)&&!line.includes('node_modules')});
 }
 return {head,frames};
}

function download(name:string,type:string,body:string):void {
 const url=URL.createObjectURL(new Blob([body],{type}));
 const a=document.createElement('a');a.href=url;a.download=name;a.click();
 setTimeout(()=>URL.revokeObjectURL(url),1000);
}
/** A CSV cell; a leading formula character is defused so a spreadsheet never runs it. */
const csvCell=(v:string|number)=>{const s=String(v);return /^[=+\-@\t\r]/.test(s)?`"'${s.replaceAll('"','""')}"`:/[",\n]/.test(s)?`"${s.replaceAll('"','""')}"`:s;};
export function groupsToCsv(groups:readonly ClientErrorGroup[]):string {
 const head=['id','status','kind','name','boundary','route','count','sessions','firstSeen','lastSeen','lastBuild','message'];
 return [head.join(','),...groups.map(g=>[g.id,g.status,g.kind,g.name,g.boundary,g.routeTemplate,g.count,g.sessions,g.firstSeen,g.lastSeen,g.lastBuild,g.message].map(csvCell).join(','))].join('\r\n');
}

export function issueMarkdown(d:ClientErrorDetail,stack:string):string {
 const g=d.group;
 return [`### ${g.name}: ${g.message}`,'',`- Where: ${g.boundary||'(no boundary)'} on \`${g.routeTemplate||'/'}\``,`- Kind: ${g.kind}`,`- Occurrences: ${g.count} in about ${g.sessions} sessions`,`- First seen: ${new Date(g.firstSeen*1000).toISOString()}`,`- Last seen: ${new Date(g.lastSeen*1000).toISOString()} (build ${g.lastBuild||'unknown'})`,'','```',stack,'```'].join('\n');
}

function Spark(props:{points:[number,number][]|undefined;from:number;to:number}):JSX.Element {
 const path=createMemo(()=>{
  const p=props.points??[];if(p.length===0)return '';
  const buckets=24;const span=Math.max(1,props.to-props.from);const sums=new Array<number>(buckets).fill(0);
  for(const [t0,n] of p){const i=Math.min(buckets-1,Math.max(0,Math.floor((t0-props.from)/span*buckets)));sums[i]=(sums[i]??0)+n;}
  const max=Math.max(1,...sums);
  return sums.map((v,i)=>`${i===0?'M':'L'}${(i/(buckets-1)*96).toFixed(1)} ${(22-v/max*20).toFixed(1)}`).join(' ');
 });
 return <svg class={`${css.spark} ${path()===''?css.sparkFlat:''}`} viewBox="0 0 96 24" role="img" aria-label={t('ce-trend')} preserveAspectRatio="none"><path d={path()||'M0 22 L96 22'} fill="none" stroke="currentColor" stroke-width="2" vector-effect="non-scaling-stroke"/></svg>;
}

export function ClientErrors(): JSX.Element {
 onMount(()=>{void loadCatalog('client-errors');});
 const shell=useShell();
 const groupId=()=>shell.router.route().rest[1];
 const canWrite=()=>shell.can.can('settings.write');
 const go=(path:string)=>{void shell.router.navigate(path);};
 return <Show when={groupId()} fallback={<List open={id=>go(sectionPath('observability',['client-errors',id]))}/>}>
  {id=><Detail id={id()} canWrite={canWrite()} back={()=>go(sectionPath('observability',['client-errors']))}/>}
 </Show>;
}

function List(props:{open:(id:string)=>void}):JSX.Element {
 const [status,setStatus]=createSignal<ClientErrorStatus|''>('open');
 const [build,setBuild]=createSignal('');
 const [boundary,setBoundary]=createSignal('');
 const [range,setRange]=createSignal(168);
 const [result,{refetch}]=createResource(()=>({status:status(),build:build(),boundary:boundary(),rangeHours:range()}),f=>listClientErrors(f).then(data=>({data}),()=>({failed:true as const})));
 const value=()=>{const v=resourceValue(result);return v&&'data' in v?v.data:undefined;};
 const failed=()=>{const v=resourceValue(result);return !!v&&'failed' in v;};
 const groups=()=>value()?.groups??[];
 const builds=createMemo(()=>[...new Set(groups().map(g=>g.lastBuild).filter(b=>b!==''))]);
 const boundaries=createMemo(()=>[...new Set(groups().map(g=>g.boundary).filter(b=>b!==''))]);
 const to=()=>Math.floor(Date.now()/1000);
 return <section aria-label={t('ce-title')} data-testid="client-errors">
  <h2 class={ux.pageTitle}>{t('ce-title')}</h2>
  <p class={ux.pageDescription}>{t('ce-intro')}</p>
  <Show when={value()?.policy}>{p=><>
   <Show when={!p().enabled}><p role="status" class={admin.note} data-testid="ce-disabled">{t('ce-disabled')}</p></Show>
   <Show when={p().enabled&&!p().sourceMaps}><p role="status" class={admin.note}>{t('ce-maps-off')}</p></Show>
  </>}</Show>
  <div class={css.filters}>
   <label class={css.filterField}>{t('ce-filter-status')}<select class={ux.control} value={status()} onChange={e=>setStatus(e.currentTarget.value as ClientErrorStatus|'')}>
    <option value="">{t('ce-status-all')}</option><For each={STATUSES}>{s=><option value={s}>{t(`ce-state-${s}`)}</option>}</For></select></label>
   <label class={css.filterField}>{t('ce-filter-build')}<select class={ux.control} value={build()} onChange={e=>setBuild(e.currentTarget.value)}>
    <option value="">{t('ce-any')}</option><For each={builds()}>{b=><option value={b}>{b}</option>}</For></select></label>
   <label class={css.filterField}>{t('ce-filter-boundary')}<select class={ux.control} value={boundary()} onChange={e=>setBoundary(e.currentTarget.value)}>
    <option value="">{t('ce-any')}</option><For each={boundaries()}>{b=><option value={b}>{b}</option>}</For></select></label>
   <label class={css.filterField}>{t('ce-filter-range')}<select class={ux.control} value={String(range())} onChange={e=>setRange(Number(e.currentTarget.value))}>
    <For each={RANGES}>{([label,hours])=><option value={String(hours)}>{label}</option>}</For></select></label>
   <div class={css.toolbar}>
    <button type="button" class="btn" onClick={()=>void refetch()}>{t('ce-refresh')}</button>
    <button type="button" class="btn" disabled={groups().length===0} onClick={()=>download('client-errors.csv','text/csv',groupsToCsv(groups()))}>{t('ce-export-csv')}</button>
    <button type="button" class="btn" disabled={groups().length===0} onClick={()=>download('client-errors.json','application/json',JSON.stringify({groups:groups()},null,2))}>{t('ce-export-json')}</button>
   </div>
  </div>
  <Show when={failed()}><p role="alert" class={admin.note} data-testid="ce-failed">{t('ce-load-failed')}</p></Show>
  <Show when={!failed()&&value()&&groups().length===0}><p role="status" class={admin.note} data-testid="ce-empty">{t('ce-empty')}</p></Show>
  <Show when={groups().length>0}>
   <div class={admin.tableWrap}><table class={admin.table}>
    <caption class={ux.pageDescription}>{t('ce-table-caption')}</caption>
    <thead><tr><th scope="col">{t('ce-col-error')}</th><th scope="col" class={css.num}>{t('ce-col-count')}</th><th scope="col" class={css.num}>{t('ce-col-sessions')}</th><th scope="col">{t('ce-col-trend')}</th><th scope="col">{t('ce-col-first')}</th><th scope="col">{t('ce-col-last')}</th><th scope="col">{t('ce-col-build')}</th><th scope="col">{t('ce-col-boundary')}</th><th scope="col">{t('ce-col-status')}</th></tr></thead>
    <tbody><For each={groups()}>{g=><tr data-testid="ce-row">
     <td><div class={css.name}><a href={`#/observability/client-errors/${g.id}`} onClick={e=>{if(e.button!==0||e.ctrlKey||e.metaKey||e.shiftKey)return;e.preventDefault();props.open(g.id);}}>{g.name}</a><span class={css.message}>{g.message}</span></div></td>
     <td class={css.num}>{formatCount(g.count)}</td><td class={css.num}>{formatCount(g.sessions)}</td>
     <td><Spark points={g.series} from={value()!.since} to={to()}/></td>
     <td>{when(g.firstSeen)}</td><td>{when(g.lastSeen)}</td><td>{g.lastBuild||t('ce-unknown')}</td><td>{g.boundary||t('ce-unknown')}</td>
     <td><span class={`${admin.badge} ${g.status==='open'?css.statusOpen:''}`}>{t(`ce-state-${g.status}`)}</span><Show when={g.reopenedCount>0}> <span class={admin.badge}>{t('ce-regressed')}</span></Show></td>
    </tr>}</For></tbody>
   </table></div>
  </Show>
 </section>;
}

function Detail(props:{id:string;canWrite:boolean;back:()=>void}):JSX.Element {
 const [result,{refetch}]=createResource(()=>props.id,id=>loadClientError(id).then(data=>({data}),(error:unknown)=>({failed:error})));
 const detail=()=>{const v=resourceValue(result);return v&&'data' in v?v.data:undefined;};
 const failed=()=>{const v=resourceValue(result);return !!v&&'failed' in v;};
 const [raw,setRaw]=createSignal(false);
 const [vendor,setVendor]=createSignal(false);
 const [selected,setSelected]=createSignal(0);
 const [feedback,setFeedback]=createSignal<{kind:'success'|'error';message:string}|null>(null);
 const [confirmDelete,setConfirmDelete]=createSignal(false);
 const event=()=>detail()?.events[selected()];
 const symbolicated=()=>event()?.stackSymbolicated??null;
 const stackText=()=>raw()||symbolicated()===null?event()?.stackRaw??'':symbolicated()!;
 const parsed=createMemo(()=>splitStack(stackText()));
 const act=async(status:ClientErrorStatus)=>{
  try{await setClientErrorStatus(props.id,status);setFeedback({kind:'success',message:t(`ce-marked-${status}`)});await refetch();}
  catch(error){setFeedback({kind:'error',message:describeAdminError(error)});}
 };
 const copy=(text:string)=>{try{void navigator.clipboard.writeText(text).then(()=>setFeedback({kind:'success',message:t('ce-copied')}),()=>undefined);}catch{/* clipboard unavailable */}};
 return <section aria-label={t('ce-detail')} data-testid="client-error-detail">
  <button type="button" class={ux.backLink} onClick={props.back}>{t('ce-back')}</button>
  <Show when={failed()}><p role="alert" class={admin.note}>{t('ce-load-failed')}</p></Show>
  <Show when={detail()}>{d=>{
   const g=()=>d().group;
   return <>
    <div class={ux.detailHead}>
     <h2 class={ux.detailTitle}>{g().name}</h2>
     <span class={`${admin.badge} ${g().status==='open'?css.statusOpen:''}`} data-testid="ce-status">{t(`ce-state-${g().status}`)}</span>
    </div>
    <p class={css.message}>{g().message}</p>
    <p class={ux.pageDescription}>{t('ce-where',{boundary:g().boundary||t('ce-unknown'),route:g().routeTemplate||'/'})} {t('ce-counts',{count:g().count,sessions:g().sessions})} {t('ce-seen',{first:when(g().firstSeen),last:when(g().lastSeen)})}</p>
    <Show when={g().resolvedBuild!==null&&g().status==='resolved'}><p class={admin.note}>{t('ce-resolved-in',{build:g().resolvedBuild??''})}</p></Show>
    <Feedback kind={feedback()?.kind??'info'} message={feedback()?.message} testId="ce-feedback"/>
    <div class={css.toolbar}>
     <Show when={props.canWrite}>
      <Show when={g().status!=='resolved'}><button type="button" class="btn" onClick={()=>void act('resolved')}>{t('ce-resolve')}</button></Show>
      <Show when={g().status!=='ignored'}><button type="button" class="btn" onClick={()=>void act('ignored')}>{t('ce-ignore')}</button></Show>
      <Show when={g().status!=='open'}><button type="button" class="btn" onClick={()=>void act('open')}>{t('ce-reopen')}</button></Show>
      <button type="button" class="btn btn--ghost" onClick={()=>setConfirmDelete(true)}>{t('ce-delete')}</button>
     </Show>
     <button type="button" class="btn" onClick={()=>copy(`${g().name}: ${g().message}\n${stackText()}`)}>{t('ce-copy-text')}</button>
     <button type="button" class="btn" onClick={()=>copy(issueMarkdown(d(),stackText()))}>{t('ce-copy-markdown')}</button>
    </div>
    <h3>{t('ce-stack')}</h3>
    <div class={css.toolbar}>
     <label><input type="checkbox" checked={raw()} disabled={symbolicated()===null} onChange={e=>setRaw(e.currentTarget.checked)}/> {t('ce-show-raw')}</label>
     <Show when={symbolicated()===null}><span class={admin.note} data-testid="ce-not-symbolicated">{t('ce-not-symbolicated')}</span></Show>
    </div>
    <Show when={raw()||symbolicated()===null} fallback={<>
     <pre class={css.rawStack}>{parsed().head.join('\n')}</pre>
     <ul class={css.frames} data-testid="ce-frames"><For each={parsed().frames}>{f=><Show when={f.app||vendor()}><li class={`${css.frame} ${f.app?css.frameApp:''}`} data-frame={f.app?'app':'vendor'}>{f.text}</li></Show>}</For></ul>
     <label><input type="checkbox" checked={vendor()} onChange={e=>setVendor(e.currentTarget.checked)}/> {t('ce-show-vendor',{count:parsed().frames.filter(f=>!f.app).length})}</label>
    </>}><pre class={css.rawStack} data-testid="ce-raw">{stackText()}</pre></Show>
    <div class={css.detailGrid}>
     <Show when={d().series.length>0}><ChartCard testId="ce-chart-over-time" title={t('ce-chart-time')} unit={t('ce-unit-per-hour')} summary={t('ce-chart-time-summary',{count:g().count})}
      table={{columns:[t('ce-col-time'),t('ce-col-count')],numeric:[false,true],rows:d().series.map(([s,n])=>[when(s),formatCount(n)])}}>
      <LineChart times={d().series.map(p=>p[0])} series={[{label:t('ce-col-count'),slot:1,values:d().series.map(p=>p[1])}]} format={n=>formatCount(n)} formatTime={s=>formatDateTime(s*1000,{dateStyle:'short',timeStyle:'short'})}/>
     </ChartCard></Show>
     <For each={(['browser','os','build','route'] as const)}>{dim=>{
      const rows=()=>d().breakdown[dim].filter(([k])=>k!=='');
      return <ChartCard testId={`ce-chart-${dim}`} title={t(`ce-chart-${dim}`)} unit={t('ce-unit-occurrences')} summary={t('ce-bars-about',{count:rows().length})}
       table={{columns:[t(`ce-chart-${dim}`),t('ce-col-count')],numeric:[false,true],rows:rows().map(([k,n])=>[k,formatCount(n)])}} empty={rows().length===0?t('ce-no-data'):undefined}>
       <BarRows bars={rows().map(([label,value])=>({label,value}))} format={formatCount}/>
      </ChartCard>;
     }}</For>
    </div>
    <h3>{t('ce-events')}</h3>
    <div class={admin.tableWrap}><table class={admin.table} data-testid="ce-events">
     <thead><tr><th scope="col">{t('ce-col-ref')}</th><th scope="col">{t('ce-col-time')}</th><th scope="col">{t('ce-col-build')}</th><th scope="col">{t('ce-col-browser')}</th><th scope="col">{t('ce-col-os')}</th><th scope="col">{t('ce-col-viewport')}</th><th scope="col">{t('ce-col-locale')}</th><th scope="col" class={css.num}>{t('ce-col-count')}</th></tr></thead>
     <tbody><For each={d().events}>{(e,i)=><tr aria-selected={selected()===i()}>
      <td><button type="button" class="btn btn--ghost" onClick={()=>setSelected(i())}><code>{e.id.replace(/-/g,'').slice(0,8)}</code></button></td>
      <td>{when(e.time)}</td><td>{e.build||t('ce-unknown')}</td><td>{e.uaFamily} {e.extra?.uaVersion??''}</td><td>{e.osFamily}</td><td>{e.viewport}</td><td>{e.locale}</td><td class={css.num}>{e.occurrences}</td>
     </tr>}</For></tbody>
    </table></div>
    <ConfirmDialog open={confirmDelete()} title={t('ce-delete-title')} body={t('ce-delete-body')} confirmLabel={t('ce-delete')}
     onCancel={()=>setConfirmDelete(false)} onConfirm={async()=>{await deleteClientError(props.id);setConfirmDelete(false);props.back();}}/>
   </>;
  }}</Show>
 </section>;
}
