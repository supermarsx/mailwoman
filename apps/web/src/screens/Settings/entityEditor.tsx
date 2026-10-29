import { createSignal, createUniqueId, onCleanup, onMount, Show, type Accessor, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { createFocusTrap } from '../../components/a11y';
import { t } from '../../i18n';
import { useSettingsNavigation, type SettingsRoute } from './navigation.tsx';
import * as css from './styles.css.ts';
import * as layout from './editor.css.ts';
export function useEntityNavigation(section: string) {
 const shared=useSettingsNavigation(); const [local,setLocal]=createSignal<SettingsRoute>({section});
 return { shared, route:()=>shared?.route()??local(), navigate:async(next:SettingsRoute):Promise<boolean>=>{if(shared)return shared.navigate(next);setLocal(next);return true;} };
}
type Flag=Accessor<boolean>;
export function useEditorGuard(dirty:Flag, busy:Flag) {
 const nav=useSettingsNavigation();const [pending,setPending]=createSignal<((leave:boolean)=>void)>();
 const confirm=():Promise<boolean>=>{if(!dirty())return Promise.resolve(true);if(busy())return Promise.resolve(false);return new Promise(resolve=>setPending(()=>resolve));};
 if(nav)onCleanup(nav.beforeLeave(confirm,dirty));
 const answer=(leave:boolean):void=>{pending()?.(leave);setPending(undefined);};
 onMount(()=>{const handler=(event:BeforeUnloadEvent):void=>{if(dirty()){event.preventDefault();event.returnValue='';}};window.addEventListener('beforeunload',handler);onCleanup(()=>window.removeEventListener('beforeunload',handler));});
 onCleanup(()=>pending()?.(false));
 return {confirm,leave:async(action:()=>void|Promise<void>):Promise<void>=>{if(nav||await confirm())await action();},dialog:():JSX.Element=><Show when={pending()}><DiscardDialog answer={answer}/></Show>};
}
function DiscardDialog(props:{answer:(leave:boolean)=>void}):JSX.Element {
 const id=createUniqueId();const [element,setElement]=createSignal<HTMLDivElement>();let keep:HTMLButtonElement|undefined;
 createFocusTrap(element,{onEscape:()=>props.answer(false),initialFocus:()=>keep});
 return <Portal><div class={layout.scrim}/><div ref={setElement} class={layout.dialog} role="alertdialog" aria-modal="true" aria-labelledby={`${id}-title`} aria-describedby={`${id}-body`} tabindex="-1"><h2 id={`${id}-title`} class={css.heading}>{t('settings-editor-discard-title')}</h2><p id={`${id}-body`}>{t('settings-editor-discard-detail')}</p><div class={css.actions}><button ref={keep} class={css.button} onClick={()=>props.answer(false)}>{t('settings-editor-keep')}</button><button class={css.danger} onClick={()=>props.answer(true)}>{t('settings-editor-discard')}</button></div></div></Portal>;
}
