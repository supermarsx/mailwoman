import {type JSX} from 'solid-js';
import {t,loadCatalog} from '../i18n';
import {Icon} from './Icon.tsx';
import {SelectionBar,type SelectionAction} from './SelectionBar.tsx';
import type {RangeSelection} from './rangeSelection.ts';
import './rangeSelection.css';
export function RangeSelectionToggle(props:{selection:RangeSelection}):JSX.Element{
 void loadCatalog('selection-bar');return <button type="button" class="btn btn--ghost range-selection-toggle" title={t('selection-bar-label')} aria-label={t('selection-bar-label')} aria-pressed={props.selection.mode()} disabled={props.selection.busy()} onClick={props.selection.toggleMode} onKeyDown={event=>props.selection.keydown('',event)}><Icon name="check"/></button>;
}
export function RangeSelectionBar(props:{selection:RangeSelection;total:number;actions:SelectionAction[];returnFocus?:()=>void}):JSX.Element{
 const s=props.selection;return <SelectionBar count={s.selected().size} countLabel={t('selection-bar-count',{count:s.selected().size})} announcement={s.selected().size?t('selection-bar-count',{count:s.selected().size}):''} active={s.mode()} total={props.total} selectAll={s.all} clear={s.clear} dismiss={s.dismiss} actions={props.actions.map(action=>({...action,...(action.run?{run:async()=>{try{await action.run?.();}catch(error){s.reportError(error);}}}:{})}))} returnFocus={props.returnFocus??(()=>undefined)} scope={s.scope()} busy={s.busy()}/>;
}
