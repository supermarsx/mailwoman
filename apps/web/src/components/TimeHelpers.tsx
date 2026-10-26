import { For, Show, type JSX } from 'solid-js';
import { settings } from '../api/settings.ts';
import { t } from '../i18n';
import { formatDateTime } from '../i18n/format.ts';
import { displayTimeZone, validTimeZone, wallTimeCandidates } from '../i18n/timezones.ts';
import { Icon } from './Icon.tsx';
import './time-helpers.css';

export const messageTimeZone=():string=>displayTimeZone(settings()['time.displayZone']?.value);
function comparisonZone():string {
  const value=settings()['time.comparisonZone']?.value;
  return typeof value==='string'&&validTimeZone(value)&&value!==messageTimeZone()?value:'';
}
function dateIn(value:string|Date,zone:string):string {return formatDateTime(value,{year:'numeric',month:'short',day:'numeric',hour:'2-digit',minute:'2-digit',second:'2-digit',timeZone:zone,timeZoneName:'short'});}
export function MessageTimes(props:{sent?:string|null|undefined;received:string}):JSX.Element {
  const when=()=>props.sent??props.received;
  return <Show when={settings()['time.showHelpers']?.value!==false}><details class="time-helper">
    <summary title={dateIn(when(),messageTimeZone())}><Icon name="clock"/><time dateTime={when()}>{formatDateTime(when(),{month:'short',day:'numeric',hour:'2-digit',minute:'2-digit',timeZone:messageTimeZone()})}</time></summary>
    <dl><dt>{t('mail-time-sent')}</dt><dd>{props.sent?dateIn(props.sent,messageTimeZone()):t('mail-time-unknown')}</dd>
      <dt>{t('mail-time-received')}</dt><dd>{dateIn(props.received,messageTimeZone())}</dd>
      <Show when={comparisonZone()}>{zone=><><dt>{t('mail-time-comparison',{zone:zone()})}</dt><dd>{dateIn(when(),zone())}</dd></>}</Show>
    </dl><p>{t('mail-time-privacy')}</p>
  </details></Show>;
}
export function EventTimeComparison(props:{wallTime:string;zone:string}):JSX.Element {
  const candidates=()=>wallTimeCandidates(props.wallTime,props.zone);
  return <Show when={settings()['time.showHelpers']?.value!==false&&candidates().length>0}>
    <section class="time-helper" aria-label={t('mail-time-event-comparison')}>
      <Show when={candidates().length>1}><p role="status">{t('mail-time-ambiguous')}</p></Show>
      <For each={candidates()}>{instant=><dl><dt>{messageTimeZone()}</dt><dd>{dateIn(instant,messageTimeZone())}</dd>
        <Show when={comparisonZone()}>{zone=><><dt>{zone()}</dt><dd>{dateIn(instant,zone())}</dd></>}</Show>
      </dl>}</For>
    </section>
  </Show>;
}
