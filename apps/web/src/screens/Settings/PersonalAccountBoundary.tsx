import {createSignal,onCleanup,Show,type JSX} from 'solid-js';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
/** Dispose private editor/resource state when the verified account generation changes. */
export function PersonalAccountBoundary(props:{children:()=>JSX.Element}):JSX.Element {
 const [owner,setOwner]=createSignal({account:preferenceAccount()});
 onCleanup(observePreferenceAccount(account=>setOwner(previous=>previous.account===account?previous:{account})));
 return <Show when={owner()} keyed>{_owner=>props.children()}</Show>;
}
