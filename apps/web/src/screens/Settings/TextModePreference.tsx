import {createUniqueId, type JSX} from 'solid-js';
import {rawT} from '../../i18n';
import * as css from './styles.css.ts';

/** Plain control labels keep the way back readable even while display text is transformed. */
export function TextModePreference(props:{enabled:boolean;disabled?:boolean;busy?:boolean;describedBy?:string | undefined;onChange:(enabled:boolean)=>void}):JSX.Element {
 const id=createUniqueId();
 return <div class={css.stackedRow} data-testid="text-mode-preference">
  <span class={css.label} id={`${id}-label`}>{rawT('appearance-text-mode-label')}</span>
  <div class={css.options} role="group" aria-labelledby={`${id}-label`} aria-describedby={[`${id}-hint`,props.describedBy].filter(Boolean).join(' ')} aria-busy={props.busy===true}>
   <button type="button" class={css.option} aria-pressed={!props.enabled} disabled={props.disabled||props.busy} onClick={()=>props.onChange(false)}>{rawT('appearance-text-mode-normal')}</button>
   <button type="button" class={css.option} aria-pressed={props.enabled} disabled={props.disabled||props.busy} onClick={()=>props.onChange(true)}>{rawT('appearance-text-mode-leet')}</button>
  </div>
  <p class={css.stackedHint} id={`${id}-hint`}>{rawT('appearance-text-mode-help')}</p>
 </div>;
}
