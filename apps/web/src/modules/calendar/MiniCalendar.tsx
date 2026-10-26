import {createEffect,createSignal,For,type JSX} from 'solid-js';
import {Icon} from '../../components/Icon.tsx';
import {t} from '../../i18n';
import {formatFull,formatMonthYear,monthGrid,weekdayNames} from './datetime.ts';
import * as css from './miniCalendar.css.ts';

export function MiniCalendar(props:{date:Date;select:(date:Date)=>void}):JSX.Element {
 const [month,setMonth]=createSignal(new Date(props.date.getFullYear(),props.date.getMonth(),1));
 createEffect(()=>setMonth(new Date(props.date.getFullYear(),props.date.getMonth(),1)));
 const move=(offset:number)=>setMonth(value=>new Date(value.getFullYear(),value.getMonth()+offset,1));
 const selected=(date:Date)=>date.toDateString()===props.date.toDateString();
 return <section class={css.root} aria-label={t('calendar-mini')}><header class={css.header}>
  <button type="button" aria-label={t('calendar-mini-prev')} onClick={()=>move(-1)}><Icon name="chevronLeft"/></button><strong>{formatMonthYear(month())}</strong><button type="button" aria-label={t('calendar-mini-next')} onClick={()=>move(1)}><Icon name="chevron"/></button>
 </header><table class={css.grid} aria-label={formatMonthYear(month())}><thead><tr><For each={weekdayNames()}>{day=><th scope="col">{day}</th>}</For></tr></thead><tbody><For each={monthGrid(month().getFullYear(),month().getMonth())}>{week=><tr><For each={week}>{date=><td><button type="button" aria-label={formatFull(date)} aria-pressed={selected(date)} aria-current={date.toDateString()===new Date().toDateString()?'date':undefined} data-outside={date.getMonth()!==month().getMonth()?'true':undefined} onClick={()=>props.select(date)}>{date.getDate()}</button></td>}</For></tr>}</For></tbody></table><button type="button" class={css.today} onClick={()=>props.select(new Date())}>{t('calendar-today')}</button></section>;
}
