import { createSignal, createUniqueId, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import * as css from './tooltip.css.ts';

const controls = 'button,a[href],input,select,textarea,[role="button"],[role="tab"],[role="switch"],[role="checkbox"],[role="separator"],[title],[data-tooltip],[data-tooltip-title]';
const excluded = '[contenteditable]:not([contenteditable="false"]),[data-tooltip="off"]';

/** App chrome only. Explicit descriptions take precedence over adopted native titles. */
export function tooltipDescription(element: HTMLElement): string | null {
  if (element.closest(excluded)) return null;
  const explicit = element.getAttribute('data-tooltip');
  if (explicit?.trim()) return explicit.trim();
  const title = element.getAttribute('title') ?? element.getAttribute('data-tooltip-title');
  if (title?.trim()) return title.trim();
  if (!element.textContent?.trim()) return element.getAttribute('aria-label')?.trim() || null;
  // Existing icon controls also use a visually hidden span as their accessible name.
  const labels = Array.from(element.childNodes).filter(node => !(node instanceof SVGElement) && node.textContent?.trim());
  if (element.querySelector('svg') && labels.length && labels.every(node => {
    if (!(node instanceof HTMLElement)) return false;
    const style = getComputedStyle(node);
    return style.position === 'absolute' && style.width === '1px' && style.height === '1px' && style.overflow === 'hidden';
  })) return labels.map(node => node.textContent?.trim()).join(' ');
  return null;
}

/** One delegated, non-modal layer; never wraps controls or intercepts activation. */
export function TooltipLayer(): JSX.Element {
  const id = `tooltip-${createUniqueId()}`;
  const [text, setText] = createSignal<string | null>(null);
  let target: HTMLElement | null = null;
  let bubble: HTMLDivElement | undefined;
  let nativeTitle: string | null = null;
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let checkTimer: ReturnType<typeof setInterval> | undefined;
  let frame = 0;
  let pointerUntil = 0;
  let dismissed: HTMLElement | null = null;
  let dismissedTitle: string | null = null;

  const clearHide = () => { clearTimeout(hideTimer); };
  const scheduleHide = () => { clearHide(); hideTimer = setTimeout(() => hide(), 100); };
  const restoreDismissed = () => {
    if (dismissed && dismissedTitle !== null && !dismissed.hasAttribute('title')) dismissed.setAttribute('title', dismissedTitle);
    dismissed = null; dismissedTitle = null;
  };
  const hide = (keepNativeSuppressed = false) => {
    clearHide(); clearInterval(checkTimer);
    cancelAnimationFrame(frame);
    if (target) {
      const described = (target.getAttribute('aria-describedby') ?? '').split(/\s+/).filter((part) => part && part !== id);
      if (described.length) target.setAttribute('aria-describedby', described.join(' '));
      else target.removeAttribute('aria-describedby');
      if (keepNativeSuppressed) { dismissed = target; dismissedTitle = nativeTitle; }
      else if (nativeTitle !== null && !target.hasAttribute('title')) target.setAttribute('title', nativeTitle);
    }
    target = null; nativeTitle = null; setText(null);
  };
  const position = () => {
    if (!target?.isConnected) { hide(); return; }
    if (!bubble) return;
    const anchor = target.getBoundingClientRect();
    const bounds = bubble.getBoundingClientRect();
    const viewport = window.visualViewport;
    const left = viewport?.offsetLeft ?? 0;
    const top = viewport?.offsetTop ?? 0;
    const width = viewport?.width ?? window.innerWidth;
    const height = viewport?.height ?? window.innerHeight;
    if (anchor.bottom < top || anchor.top > top + height || anchor.right < left || anchor.left > left + width) { hide(); return; }
    const x = Math.max(left + 8, Math.min(anchor.left + (anchor.width - bounds.width) / 2, left + width - bounds.width - 8));
    const preferred = anchor.top - bounds.height - 8;
    const y = Math.max(top + 8, Math.min(preferred >= top + 8 ? preferred : anchor.bottom + 8, top + height - bounds.height - 8));
    bubble.style.left = `${Math.round(x)}px`; bubble.style.top = `${Math.round(y)}px`;
  };
  const schedulePosition = () => { cancelAnimationFrame(frame); frame = requestAnimationFrame(position); };
  const find = (event: Event) => event.target instanceof Element ? event.target.closest<HTMLElement>(controls) : null;
  const show = (element: HTMLElement | null) => {
    if (!element || Date.now() < pointerUntil) return;
    if (element === dismissed) return;
    restoreDismissed();
    if (target === element) { clearHide(); return; }
    hide();
    const description = tooltipDescription(element);
    if (!description) return;
    target = element; nativeTitle = element.getAttribute('title');
    element.removeAttribute('title');
    {
      if (!element.isConnected || target !== element) { hide(); return; }
      const existing = element.getAttribute('aria-describedby');
      element.setAttribute('aria-describedby', [existing, id].filter(Boolean).join(' '));
      setText(description);
      queueMicrotask(position);
      checkTimer = setInterval(() => { if (!element.isConnected) hide(); }, 200);
    }
  };
  onMount(() => {
    // Adopt titles before hover, including non-control badges and disabled
    // controls. Keeping native title attributes causes duplicate browser hints.
    const adopt = (root: ParentNode) => {
      const elements = [...root.querySelectorAll<HTMLElement>('[title]')];
      if (root instanceof HTMLElement && root.hasAttribute('title')) elements.push(root);
      for (const element of elements) {
        if (!(element instanceof HTMLElement) || element.closest(excluded)) continue;
        const title = element.getAttribute('title') ?? '';
        // A title-only icon control must retain its accessible name after the
        // native tooltip attribute is removed. Do not override visible labels.
        if (title && element.matches('button,a[href],[role="button"]') && !element.textContent?.trim() && !element.hasAttribute('aria-label') && !element.hasAttribute('aria-labelledby')) element.setAttribute('aria-label', title);
        element.setAttribute('data-tooltip-title', title);
        element.removeAttribute('title');
      }
    };
    adopt(document);
    const observer = new MutationObserver(records => {
      for (const record of records) {
        if (record.type === 'attributes' && record.target instanceof HTMLElement) adopt(record.target);
        for (const node of record.addedNodes) if (node instanceof HTMLElement) adopt(node);
      }
      if (target) {
        if (!target.isConnected) hide();
        else { const description = tooltipDescription(target); if (description) { setText(description); schedulePosition(); } else hide(); }
      }
    });
    observer.observe(document.body, {subtree:true,childList:true,attributes:true,attributeFilter:['title','data-tooltip','aria-label']});
    const over = (event: PointerEvent) => { if (event.pointerType !== 'touch') { const element=find(event); if (element!==dismissed) pointerUntil=0; show(element); } };
    const out = (event: PointerEvent) => {
      if (!(event.target instanceof Node) || !(target?.contains(event.target) || bubble?.contains(event.target) || dismissed?.contains(event.target))) return;
      if (dismissed && !(event.relatedTarget instanceof Node && dismissed.contains(event.relatedTarget))) restoreDismissed();
      if (event.relatedTarget instanceof Node && (target?.contains(event.relatedTarget) || bubble?.contains(event.relatedTarget))) return;
      if (target !== document.activeElement) scheduleHide();
    };
    const focus = (event: FocusEvent) => show(find(event));
    const blur = () => { restoreDismissed(); hide(); };
    const key = (event: KeyboardEvent) => { pointerUntil = 0; if (event.key === 'Escape') hide(true); };
    // Pointer activation can focus a button after pointerdown. Do not reopen
    // its tooltip over the new page/tab as a side effect of that focus.
    const down = () => { pointerUntil = Date.now() + 1000; hide(true); };
    document.addEventListener('pointerover', over, true); document.addEventListener('pointerout', out, true);
    document.addEventListener('focusin', focus); document.addEventListener('focusout', blur);
    document.addEventListener('keydown', key); document.addEventListener('pointerdown', down, true);
    document.addEventListener('scroll', schedulePosition, true); window.addEventListener('resize', schedulePosition);
    window.visualViewport?.addEventListener('resize', schedulePosition);
    window.visualViewport?.addEventListener('scroll', schedulePosition);
    onCleanup(() => {
      observer.disconnect();
      restoreDismissed(); hide();
      for (const element of document.querySelectorAll<HTMLElement>('[data-tooltip-title]')) { if (!element.hasAttribute('title')) element.setAttribute('title', element.getAttribute('data-tooltip-title') ?? ''); element.removeAttribute('data-tooltip-title'); }
      document.removeEventListener('pointerover', over, true); document.removeEventListener('pointerout', out, true);
      document.removeEventListener('focusin', focus); document.removeEventListener('focusout', blur);
      document.removeEventListener('keydown', key); document.removeEventListener('pointerdown', down, true);
      document.removeEventListener('scroll', schedulePosition, true); window.removeEventListener('resize', schedulePosition);
      window.visualViewport?.removeEventListener('resize', schedulePosition);
      window.visualViewport?.removeEventListener('scroll', schedulePosition);
    });
  });
  return <Portal><Show when={text()}>{(description) => <div ref={bubble} id={id} role="tooltip" class={css.tooltip} onPointerEnter={clearHide} onPointerLeave={scheduleHide}>{description()}</div>}</Show></Portal>;
}
