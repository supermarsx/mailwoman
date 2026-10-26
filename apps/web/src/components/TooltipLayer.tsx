import { createSignal, createUniqueId, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import * as css from './tooltip.css.ts';

const controls = 'button,a[href],input,select,textarea,[role="button"],[role="tab"],[role="switch"],[role="checkbox"],[role="separator"],[title],[data-tooltip],[data-tooltip-title]';
const excluded = '[contenteditable]:not([contenteditable="false"]),[data-tooltip="off"]';

// Adopted elements and popup-created elements can have different realm prototypes.
const isElement = (value:unknown):value is HTMLElement => !!value && typeof value==='object' && (value as Node).nodeType===1;
const isNode = (value:unknown):value is Node => !!value && typeof value==='object' && typeof (value as Node).nodeType==='number';

/** App chrome only. Explicit descriptions take precedence over adopted native titles. */
export function tooltipDescription(element: HTMLElement): string | null {
  if (element.closest(excluded)) return null;
  const explicit = element.getAttribute('data-tooltip');
  if (explicit?.trim()) return explicit.trim();
  const title = element.getAttribute('title') ?? element.getAttribute('data-tooltip-title');
  if (title?.trim()) return title.trim();
  if (!element.textContent?.trim()) return element.getAttribute('aria-label')?.trim() || null;
  // Existing icon controls also use a visually hidden span as their accessible name.
  const labels = Array.from(element.childNodes).filter(node => (!isElement(node) || node.namespaceURI !== 'http://www.w3.org/2000/svg') && node.textContent?.trim());
  if (element.querySelector('svg') && labels.length && labels.every(node => {
    if (!isElement(node)) return false;
    const style = (node.ownerDocument.defaultView ?? window).getComputedStyle(node);
    return style.position === 'absolute' && style.width === '1px' && style.height === '1px' && style.overflow === 'hidden';
  })) return labels.map(node => node.textContent?.trim()).join(' ');
  return null;
}

/** One delegated, non-modal layer; never wraps controls or intercepts activation. */
export function TooltipLayer(props:{document?:Document} = {}): JSX.Element {
  const scopeDocument=props.document ?? document;
  const scopeWindow=scopeDocument.defaultView ?? window;
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
    scopeWindow.cancelAnimationFrame(frame);
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
    const viewport = scopeWindow.visualViewport;
    const left = viewport?.offsetLeft ?? 0;
    const top = viewport?.offsetTop ?? 0;
    const width = viewport?.width ?? scopeWindow.innerWidth;
    const height = viewport?.height ?? scopeWindow.innerHeight;
    if (anchor.bottom < top || anchor.top > top + height || anchor.right < left || anchor.left > left + width) { hide(); return; }
    const x = Math.max(left + 8, Math.min(anchor.left + (anchor.width - bounds.width) / 2, left + width - bounds.width - 8));
    const preferred = anchor.top - bounds.height - 8;
    const y = Math.max(top + 8, Math.min(preferred >= top + 8 ? preferred : anchor.bottom + 8, top + height - bounds.height - 8));
    bubble.style.left = `${Math.round(x)}px`; bubble.style.top = `${Math.round(y)}px`;
  };
  const schedulePosition = () => { scopeWindow.cancelAnimationFrame(frame); frame = scopeWindow.requestAnimationFrame(position); };
  const find = (event: Event) => isElement(event.target) ? event.target.closest<HTMLElement>(controls) : null;
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
      if (isElement(root) && root.hasAttribute('title')) elements.push(root);
      for (const element of elements) {
        if (!(isElement(element)) || element.closest(excluded)) continue;
        const title = element.getAttribute('title') ?? '';
        // A title-only icon control must retain its accessible name after the
        // native tooltip attribute is removed. Do not override visible labels.
        if (title && element.matches('button,a[href],[role="button"]') && !element.textContent?.trim() && !element.hasAttribute('aria-label') && !element.hasAttribute('aria-labelledby')) element.setAttribute('aria-label', title);
        element.setAttribute('data-tooltip-title', title);
        element.removeAttribute('title');
      }
    };
    adopt(scopeDocument);
    const observer = new MutationObserver(records => {
      for (const record of records) {
        if (record.type === 'attributes' && isElement(record.target)) adopt(record.target);
        for (const node of record.addedNodes) if (isElement(node)) adopt(node);
      }
      if (target) {
        if (!target.isConnected) hide();
        else { const description = tooltipDescription(target); if (description) { setText(description); schedulePosition(); } else hide(); }
      }
    });
    observer.observe(scopeDocument.body, {subtree:true,childList:true,attributes:true,attributeFilter:['title','data-tooltip','aria-label']});
    const over = (event: PointerEvent) => { if (event.pointerType !== 'touch') { const element=find(event); if (element!==dismissed) pointerUntil=0; show(element); } };
    const out = (event: PointerEvent) => {
      if (!(isNode(event.target)) || !(target?.contains(event.target) || bubble?.contains(event.target) || dismissed?.contains(event.target))) return;
      if (dismissed && !(isNode(event.relatedTarget) && dismissed.contains(event.relatedTarget))) restoreDismissed();
      if (isNode(event.relatedTarget) && (target?.contains(event.relatedTarget) || bubble?.contains(event.relatedTarget))) return;
      if (target !== scopeDocument.activeElement) scheduleHide();
    };
    const focus = (event: FocusEvent) => show(find(event));
    const blur = () => { restoreDismissed(); hide(); };
    const key = (event: KeyboardEvent) => { pointerUntil = 0; if (event.key === 'Escape') hide(true); };
    // Pointer activation can focus a button after pointerdown. Do not reopen
    // its tooltip over the new page/tab as a side effect of that focus.
    const down = () => { pointerUntil = Date.now() + 1000; hide(true); };
    scopeDocument.addEventListener('pointerover', over, true); scopeDocument.addEventListener('pointerout', out, true);
    scopeDocument.addEventListener('focusin', focus); scopeDocument.addEventListener('focusout', blur);
    scopeDocument.addEventListener('keydown', key); scopeDocument.addEventListener('pointerdown', down, true);
    scopeDocument.addEventListener('scroll', schedulePosition, true); scopeWindow.addEventListener('resize', schedulePosition);
    scopeWindow.visualViewport?.addEventListener('resize', schedulePosition);
    scopeWindow.visualViewport?.addEventListener('scroll', schedulePosition);
    onCleanup(() => {
      observer.disconnect();
      restoreDismissed(); hide();
      for (const element of scopeDocument.querySelectorAll<HTMLElement>('[data-tooltip-title]')) { if (!element.hasAttribute('title')) element.setAttribute('title', element.getAttribute('data-tooltip-title') ?? ''); element.removeAttribute('data-tooltip-title'); }
      scopeDocument.removeEventListener('pointerover', over, true); scopeDocument.removeEventListener('pointerout', out, true);
      scopeDocument.removeEventListener('focusin', focus); scopeDocument.removeEventListener('focusout', blur);
      scopeDocument.removeEventListener('keydown', key); scopeDocument.removeEventListener('pointerdown', down, true);
      scopeDocument.removeEventListener('scroll', schedulePosition, true); scopeWindow.removeEventListener('resize', schedulePosition);
      scopeWindow.visualViewport?.removeEventListener('resize', schedulePosition);
      scopeWindow.visualViewport?.removeEventListener('scroll', schedulePosition);
    });
  });
  return <Portal mount={scopeDocument.body}><Show when={text()}>{(description) => <div ref={bubble} id={id} role="tooltip" class={css.tooltip} onPointerEnter={clearHide} onPointerLeave={scheduleHide}>{description()}</div>}</Show></Portal>;
}
