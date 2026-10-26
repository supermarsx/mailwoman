import {createSignal,type JSX} from 'solid-js';
import {Icon} from '../../../components/Icon.tsx';
import {t} from '../../../i18n';
import * as css from './dashboard.css.ts';
import type {DashboardWidgetId} from './preferences.ts';
export function reorderDashboard(order:readonly DashboardWidgetId[],source:DashboardWidgetId,target:DashboardWidgetId):DashboardWidgetId[]{
 const next=[...order],from=next.indexOf(source),to=next.indexOf(target);if(from<0||to<0||from===to)return next;next.splice(from,1);next.splice(to,0,source);return next;
}
export function DashboardReorder(props:{id:DashboardWidgetId;order:readonly DashboardWidgetId[];selector:string;disabled?:boolean;move:(source:DashboardWidgetId,target:DashboardWidgetId)=>void}):JSX.Element {
 const [dragging,setDragging]=createSignal(false);let pointer:number|undefined;let startX=0,startY=0;
 return <button type="button" class={css.dragHandle} disabled={props.disabled} aria-label={t('admin-dash-reorder',{name:t(`admin-personal-dashboard-${props.id}`)})} title={t('admin-dash-reorder-help')} aria-pressed={dragging()}
 onPointerDown={event=>{if(event.button!==0)return;pointer=event.pointerId;startX=event.clientX;startY=event.clientY;event.currentTarget.setPointerCapture?.(pointer);setDragging(true);}}
 onPointerCancel={()=>{pointer=undefined;setDragging(false);}}
 onPointerUp={event=>{if(pointer!==event.pointerId)return;pointer=undefined;setDragging(false);if(Math.hypot(event.clientX-startX,event.clientY-startY)<5)return;const target=document.elementFromPoint(event.clientX,event.clientY)?.closest<HTMLElement>(props.selector)?.dataset['reorderId'];if(target&&props.order.includes(target as DashboardWidgetId))props.move(props.id,target as DashboardWidgetId);}}
 onKeyDown={event=>{if(event.key==='Escape'){pointer=undefined;setDragging(false);return;}const delta=event.key==='ArrowUp'?-1:event.key==='ArrowDown'?1:0;if(!delta)return;event.preventDefault();const target=props.order[props.order.indexOf(props.id)+delta];if(target)props.move(props.id,target);}}><Icon name="menu"/></button>;
}
