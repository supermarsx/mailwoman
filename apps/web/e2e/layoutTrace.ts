import type {Page} from '@playwright/test';

/**
 * Diagnostic: wraps the script-visible layout readers in EVERY frame of the page
 * (Playwright init scripts run in nested and sandboxed frames too) and logs
 * `LAYOUTREAD <api> ready=<readyState> <proto> <caller frames>` through the console
 * for each call. Used to find the call sites behind Firefox's "Layout was forced
 * before the page was fully loaded" warning.
 */
export async function traceLayoutReads(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const NL = String.fromCharCode(10);
    const log = (what: string): void => {
      const stack = (new Error().stack ?? '').split(NL).slice(2, 4).join(' | ').slice(0, 240);
      console.log('LAYOUTREAD ' + what + ' ready=' + document.readyState + ' ' + location.protocol + ' ' + stack);
    };
    const wrap = (proto: object, name: string): void => {
      const d = Object.getOwnPropertyDescriptor(proto, name);
      if (!d) return;
      if (d.get) { const g = d.get; Object.defineProperty(proto, name, {...d, get(this: unknown) { log(name); return g.call(this); }}); }
      else if (typeof d.value === 'function') { const f = d.value as (...a: unknown[]) => unknown; Object.defineProperty(proto, name, {...d, value: function (this: unknown, ...a: unknown[]) { log(name); return f.apply(this, a); }}); }
    };
    for (const name of ['getBoundingClientRect', 'getClientRects', 'scrollIntoView', 'scrollTo', 'scrollBy', 'scrollIntoViewIfNeeded', 'scrollHeight', 'scrollWidth', 'scrollTop', 'scrollLeft', 'clientWidth', 'clientHeight', 'clientTop', 'clientLeft']) wrap(Element.prototype, name);
    for (const name of ['offsetWidth', 'offsetHeight', 'offsetTop', 'offsetLeft', 'focus', 'innerText']) wrap(HTMLElement.prototype, name);
    for (const name of ['scrollX', 'scrollY', 'innerHeight', 'innerWidth', 'scrollTo', 'scrollBy', 'getComputedStyle', 'pageXOffset', 'pageYOffset']) wrap(window, name);
    wrap(Range.prototype, 'getBoundingClientRect'); wrap(Range.prototype, 'getClientRects');
    wrap(Document.prototype, 'elementFromPoint'); wrap(Document.prototype, 'caretPositionFromPoint');
    wrap(CSSStyleDeclaration.prototype, 'getPropertyValue');
  });
}
