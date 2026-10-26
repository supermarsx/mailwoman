import {createEffect,createSignal,onCleanup,Show,type JSX} from 'solid-js';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';

/** Mount on first visit, retain ordinary panels, and discard every private tree
 * when the authenticated generation changes, including same-ID realm changes. */
export function RetainedSettingsSection(props:{name:string;active:boolean;retain?:boolean;accountBound?:boolean;children:()=>JSX.Element}):JSX.Element {
 const [generation,setGeneration]=createSignal({account:preferenceAccount()});
 onCleanup(observePreferenceAccount(account=>setGeneration(previous=>previous.account===account?previous:{account})));
 return <Show when={props.accountBound===false||generation().account?generation():undefined} keyed>{_generation=> <SectionLifetime {...props}/>}</Show>;
}
function SectionLifetime(props:{name:string;active:boolean;retain?:boolean;children:()=>JSX.Element}):JSX.Element {
 const [visited,setVisited]=createSignal(props.active);
 createEffect(()=>{if(props.active)setVisited(true);});
 return <Show when={props.retain===false?props.active:visited()}><div data-settings-section={props.name} hidden={!props.active} inert={!props.active}>{props.children()}</div></Show>;
}
