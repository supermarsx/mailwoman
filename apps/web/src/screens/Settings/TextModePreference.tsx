import {type JSX} from 'solid-js';
import {rawT} from '../../i18n';
import {ControlRow} from './layout/SettingRow.tsx';
import * as css from './styles.css.ts';

/** Plain control labels keep the way back readable even while display text is transformed. */
export function TextModePreference(props:{enabled:boolean;disabled?:boolean;busy?:boolean;describedBy?:string | undefined;note?:JSX.Element;onChange:(enabled:boolean)=>void}):JSX.Element {
 const id='settings-text-mode';
 return <div data-testid="text-mode-preference"><ControlRow controlId={`${id}-label`} group label={rawT('appearance-text-mode-label')} help={rawT('appearance-text-mode-help')} note={props.note}>{()=>
  <div class={css.options} role="group" aria-labelledby={`${id}-label`} aria-describedby={[`${id}-label-help`,props.describedBy].filter(Boolean).join(' ')} aria-busy={props.busy===true}>
   <button type="button" class={css.option} aria-pressed={!props.enabled} disabled={props.disabled||props.busy} onClick={()=>props.onChange(false)}>{rawT('appearance-text-mode-normal')}</button>
   <button type="button" class={css.option} aria-pressed={props.enabled} disabled={props.disabled||props.busy} onClick={()=>props.onChange(true)}>{rawT('appearance-text-mode-leet')}</button>
  </div>
 }</ControlRow></div>;
}
