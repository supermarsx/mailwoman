import { createEffect, onCleanup } from 'solid-js';
import { useSettingsNavigation } from '../navigation.tsx';

const FRAMES = 90;

/** Deep link (`SettingsRoute.card`): once the section is active and the card has
 * rendered, scroll it into view and outline it briefly. It waits for the card by
 * frame (a lazily loaded section renders late) and gives up after about a second;
 * nothing moves, appears or reflows, so the page does not shift. */
export function useCardLink(section: string, active: () => boolean, root: () => HTMLElement | undefined): void {
  const nav = useSettingsNavigation();
  let done = '';
  let frame: number | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  createEffect(() => {
    const want = nav?.route().card;
    const route = nav?.route().section;
    if (!want || route !== section || !active() || done === `${section}/${want}`) return;
    let left = FRAMES;
    const attempt = (): void => {
      frame = undefined;
      const el = root()?.querySelector<HTMLElement>(`[id="${CSS.escape(want)}"]`);
      if (!el) { if (--left > 0) frame = requestAnimationFrame(attempt); return; }
      done = `${section}/${want}`;
      const disclosure = el.querySelector<HTMLDetailsElement>(':scope > details');
      if (disclosure) disclosure.open = true;
      const reduce = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
      el.scrollIntoView?.({ block: 'start', behavior: reduce ? 'auto' : 'smooth' });
      el.dataset.cardFlash = '';
      timer = setTimeout(() => delete el.dataset.cardFlash, 1600);
    };
    frame = requestAnimationFrame(attempt);
  });
  onCleanup(() => { if (frame !== undefined) cancelAnimationFrame(frame); if (timer !== undefined) clearTimeout(timer); });
}
