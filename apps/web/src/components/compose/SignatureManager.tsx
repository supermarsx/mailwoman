import {createSignal,lazy,Suspense,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {createFocusTrap} from '../a11y';
import {InlineLoading} from '../InlineLoading.tsx';
import {t} from '../../i18n';
import {createSettingsNavigation,SettingsNavigationContext} from '../../screens/Settings/navigation.tsx';
import * as css from './signatureGroup.css.ts';
const Signatures=lazy(()=>import('../../screens/Settings/Signatures.tsx'));
/** Keep the draft mounted while its signature settings are edited. */
export function SignatureManager(props:{document:Document;onClose:()=>void}):JSX.Element {
 const [element,setElement]=createSignal<HTMLDivElement>();
 const navigation=createSettingsNavigation({section:'signatures'});
 const close=async()=>{if(await navigation.canLeave())props.onClose();};
 createFocusTrap(element,{onEscape:()=>void close()});
 return <Portal mount={props.document.body}><div class={css.scrim}><div ref={setElement} class={css.manager} role="dialog" aria-modal="true" aria-label={t('sig-edit')} tabindex="-1" data-testid="signature-manager">
 <button type="button" class="btn btn--ghost" onClick={()=>void close()}>{t('common-close')}</button>
 <SettingsNavigationContext.Provider value={navigation}><Suspense fallback={<InlineLoading/>}><Signatures/></Suspense></SettingsNavigationContext.Provider>
 </div></div></Portal>;
}
