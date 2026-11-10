import { createEffect, onCleanup } from 'solid-js';
import { useSettingsNavigation, type SettingsRoute } from '../navigation.tsx';

const FRAMES = 300;
const FLASH_MS = 2400;

/** Search links identify the exact control; older card links still work. Wait
 * for lazy sections, reveal enclosing disclosures, focus and mark the target.
 * Route identity makes selecting the same result again repeat the feedback. */
export function useCardLink(section: string, active: () => boolean, root: () => HTMLElement | undefined): void {
  const nav = useSettingsNavigation();
  let done:SettingsRoute|undefined;
  let frame: number | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let highlighted:HTMLElement|undefined;
  let temporaryFocus:HTMLElement|undefined;
  const clear=():void=>{
    if(frame!==undefined)cancelAnimationFrame(frame);frame=undefined;
    if(timer!==undefined)clearTimeout(timer);timer=undefined;
    if(highlighted)delete highlighted.dataset.cardFlash;highlighted=undefined;
    temporaryFocus?.removeAttribute('tabindex');temporaryFocus=undefined;
  };
  createEffect(() => {
    const route=nav?.route();
    clear();
    if (!route?.card || route.section !== section || !active() || done === route) return;
    let left = FRAMES;
    const attempt = (): void => {
      frame = undefined;
      if(nav?.route()!==route||!active())return;
      const host=root();
      const control=host?.querySelector<HTMLElement>(`[id="${CSS.escape(route.row??route.card!)}"]`);
      if (!host||!control) { if (--left > 0) frame = requestAnimationFrame(attempt); return; }
      for(let ancestor:HTMLElement|null=control;ancestor&&host.contains(ancestor);ancestor=ancestor.parentElement)if(ancestor instanceof HTMLDetailsElement)ancestor.open=true;
      if(control.closest('[hidden],[inert]')){if(--left>0)frame=requestAnimationFrame(attempt);return;}
      // Card-only links open their own disclosure too.
      if(!route.row){const disclosure=control.querySelector<HTMLDetailsElement>(':scope > details');if(disclosure)disclosure.open=true;}
      done=route;
      const target=route.row?(control.closest<HTMLElement>('[data-setting-row]')??control):control;
      const reduce = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
      target.scrollIntoView?.({ block: 'center', inline:'nearest', behavior: reduce ? 'auto' : 'smooth' });
      const focusable=control.matches('input:not(:disabled),select:not(:disabled),textarea:not(:disabled),button:not(:disabled),a[href]')?control:control.querySelector<HTMLElement>('input:not(:disabled),select:not(:disabled),textarea:not(:disabled),button:not(:disabled),a[href]');
      if(focusable)focusable.focus({preventScroll:true});
      else {if(!target.hasAttribute('tabindex')){target.tabIndex=-1;temporaryFocus=target;}target.focus({preventScroll:true});}
      highlighted=target;target.dataset.cardFlash='';
      timer=setTimeout(()=>{if(highlighted===target){delete target.dataset.cardFlash;highlighted=undefined;}temporaryFocus?.removeAttribute('tabindex');temporaryFocus=undefined;timer=undefined;},FLASH_MS);
    };
    frame = requestAnimationFrame(attempt);
  });
  onCleanup(clear);
}
