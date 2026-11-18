import {For,Show,type JSX} from 'solid-js';
import { t } from '../i18n';
import {settings} from '../api/settings.ts';
import { InlineLoading } from './InlineLoading.tsx';
import {useDelayedBusy} from './useDelayedBusy.ts';
import {useLoadingNotice} from './useLoadingNotice.ts';
import * as a11y from './mailA11y.css.ts';
import * as css from './MessageLoading.css.ts';

function Skeleton(props:{delay:number;message:string}):JSX.Element {
 const visible=useDelayedBusy(()=>true,props.delay);
 useLoadingNotice(()=>true);
 return <div role="status" aria-live="polite" aria-busy="true" data-testid="viewer-loading" data-visible={visible()}>
  <span class={a11y.srOnly}>{props.message}</span>
  <Show when={visible()}><div class={css.skeleton} data-skeleton="viewer" aria-hidden="true"><For each={[0,1,2]}>{()=> <div class={css.paragraph}><For each={[0,1,2,3]}>{()=> <span class={css.line}/>}</For></div>}</For></div></Show>
 </div>;
}
/** Local, delayed viewer skeleton; existing user/admin appearance choices apply. */
export function MessageLoading(props: { variant?: string; delayMs?: number; message?: string }): JSX.Element {
  const variant=()=>props.variant??String(settings()['reader.loading']?.value??'skeleton');
  const delay=()=>{const value=props.delayMs??Number(settings()['reader.loadingDelayMs']?.value??120);return Number.isFinite(value)?Math.max(0,Math.min(1000,value)):120;};
  return <Show when={variant()!=='indicator'&&variant()!=='none'} fallback={<InlineLoading class={css.loading} message={props.message??t('common-loading')} delayMs={variant()==='none'?3_600_000:delay()}/>}><Skeleton delay={delay()} message={props.message??t('common-loading')}/></Show>;
}
