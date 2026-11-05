// The themed image tooltip inside the hash-authorized frame handler (jsdom).
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

const source = readFileSync(resolve(__dirname, '../../../../crates/mw-server/src/reader-address-handler.js'), 'utf8');
const gif = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
const base = 'https://mail.example.test/api/rc/';
const booted: Array<Map<string, (event: Record<string, unknown>) => void>> = [];

function boot(html: string) {
  registered.clear();
  const add = document.addEventListener.bind(document), remove = document.removeEventListener.bind(document);
  vi.spyOn(document, 'addEventListener').mockImplementation(((type: string, fn: never, o?: boolean) => { (registered.get(type) ?? registered.set(type, new Set()).get(type)!).add(fn); add(type, fn, o); }) as typeof document.addEventListener);
  vi.spyOn(document, 'removeEventListener').mockImplementation(((type: string, fn: never, o?: boolean) => { registered.get(type)?.delete(fn); remove(type, fn, o); }) as typeof document.removeEventListener);
  document.documentElement.querySelectorAll('.mw-frame-tip').forEach(n => n.remove());
  document.head.innerHTML = '';
  document.body.innerHTML = html;
  document.body.setAttribute('data-mw-tip-blocked', 'Blocked: use Show images');
  document.body.setAttribute('data-mw-tip-loading', 'Loading image…');
  document.body.setAttribute('data-mw-tip-error', 'Image unavailable');
  const parent = { postMessage: vi.fn() };
  const port = { start: vi.fn(), postMessage: vi.fn(), close: vi.fn(), onmessage: null as null | ((e: { data: unknown }) => void) };
  const listeners = new Map<string, (event: Record<string, unknown>) => void>();
  new Function('document', 'parent', 'addEventListener', 'removeEventListener', source)(document, parent, (t: string, fn: (e: Record<string, unknown>) => void) => listeners.set(t, fn), (t: string) => listeners.delete(t));
  booted.push(listeners);
  const nonce = 'c'.repeat(48);
  listeners.get('message')!({ source: parent, origin: 'null', data: { type: 'mw-reader-address-init', nonce, revision: 1, prefixes: [base + 'i/'] }, ports: [port] });
  const patch = (ops: unknown[]) => port.onmessage!({ data: { type: 'patch', nonce, revision: 1, id: 1, ops } });
  return { listeners, patch };
}
// jsdom marks every dispatched event untrusted and isTrusted cannot be redefined,
// so trusted events are delivered straight to the document listeners the handler registered.
const registered = new Map<string, Set<(event: Record<string, unknown>) => void>>();
const fire = (el: Element, type: string, extra: Record<string, unknown> = {}) => {
  const event = { type, target: el, isTrusted: true, pointerType: 'mouse', relatedTarget: null, ...extra };
  for (const fn of Array.from(registered.get(type) ?? [])) fn(event);
};
const tip = () => document.documentElement.querySelector<HTMLElement>('.mw-frame-tip');
const hover = (el: Element) => { fire(el, 'pointerover'); vi.advanceTimersByTime(450); };
const leave = (el: Element) => fire(el, 'pointerout', { relatedTarget: document.body });

beforeEach(() => { vi.useFakeTimers(); vi.stubGlobal('scrollTo', vi.fn()); });
afterEach(() => { for (const l of booted.splice(0)) l.get('pagehide')?.({}); vi.restoreAllMocks(); vi.useRealTimers(); });

it('waits about 400 ms, then shows a role=tooltip bubble with the alt, wired by aria-describedby', () => {
  boot('<img data-mw-ph="box" data-mw-blocked-src="mw-rc:1" alt="Company logo" role="img" aria-label="Company logo">');
  const img = document.querySelector('img')!;
  fire(img, 'pointerover');
  vi.advanceTimersByTime(399);
  expect(tip()).toBeNull();
  vi.advanceTimersByTime(2);
  const bubble = tip()!;
  expect(bubble.getAttribute('role')).toBe('tooltip');
  expect(bubble.textContent).toBe('Company logo');
  expect(img.getAttribute('aria-describedby')).toBe(bubble.id);
  expect(document.body.contains(bubble)).toBe(false); // child of <html>: body indices used by patches never shift
  expect(img.getAttribute('aria-label')).toBe('Company logo');
  expect(img.hasAttribute('title')).toBe(false);
  leave(img);
  expect(tip()).toBeNull();
  expect(img.hasAttribute('aria-describedby')).toBe(false);
});

it('uses the app-provided string for each state when there is no alt or title', () => {
  boot(`<img data-mw-ph="box"><img data-mw-ph="box" src="${base}i/t" data-mw-image-state="loading"><img src="${base}i/u" data-mw-image-state="error">`);
  const [blocked, loading, error] = Array.from(document.querySelectorAll('img'));
  // jsdom never loads a source, so the handler's own bookkeeping reports "loading" for all; pin the states.
  loading!.setAttribute('data-mw-image-state', 'loading'); error!.setAttribute('data-mw-image-state', 'error');
  for (const [img, text] of [[blocked, 'Blocked: use Show images'], [loading, 'Loading image…'], [error, 'Image unavailable']] as const) {
    hover(img!);
    expect(tip()!.textContent).toBe(text);
    leave(img!);
  }
});

it('shows the sender title (data-mw-tip) first, even on a loaded image, and nothing for an untitled loaded one', () => {
  boot(`<img src="${base}i/a" data-mw-image-state="loaded" data-mw-tip="Chart 2"><img src="${base}i/b" data-mw-image-state="loaded" alt="x">`);
  const [titled, plain] = Array.from(document.querySelectorAll('img'));
  titled!.setAttribute('data-mw-image-state', 'loaded'); plain!.setAttribute('data-mw-image-state', 'loaded');
  hover(titled!);
  expect(tip()!.textContent).toBe('Chart 2');
  leave(titled!);
  hover(plain!);
  expect(tip()).toBeNull();
});

it('ignores touch and synthetic events; Escape, pointerdown and scroll dismiss', () => {
  const { listeners } = boot('<img data-mw-ph="box" alt="A">');
  const img = document.querySelector('img')!;
  fire(img, 'pointerover', { pointerType: 'touch' }); vi.advanceTimersByTime(500);
  img.dispatchEvent(new Event('pointerover', { bubbles: true })); vi.advanceTimersByTime(500);
  expect(tip()).toBeNull();
  for (const dismiss of [() => fire(document.body, 'keydown', { key: 'Escape' }), () => fire(img, 'pointerdown'), () => listeners.get('scroll')!({})]) {
    hover(img);
    expect(tip()).not.toBeNull();
    dismiss();
    expect(tip()).toBeNull();
    leave(img);
  }
});

it('leaving before the delay cancels it', () => {
  boot('<img data-mw-ph="box" alt="A">');
  const img = document.querySelector('img')!;
  fire(img, 'pointerover'); vi.advanceTimersByTime(200);
  leave(img);
  vi.advanceTimersByTime(500);
  expect(tip()).toBeNull();
});

it('opens at once on keyboard focus of an image-only link and closes on blur', () => {
  boot('<a href="https://e.test"><img data-mw-ph="box" alt="Go"></a>');
  const link = document.querySelector('a')!;
  link.focus();
  fire(link, 'focusin');
  expect(tip()!.textContent).toBe('Go');
  expect(link.getAttribute('aria-describedby')).toBe('mw-frame-tip');
  fire(link, 'focusout');
  expect(tip()).toBeNull();
  expect(link.hasAttribute('aria-describedby')).toBe(false);
});

it('an in-place patch updates the open bubble to loading, and settling to loaded removes it', async () => {
  const { patch } = boot(`<img data-mw-ph="box" data-mw-blocked-src="mw-rc:1" src="${gif}">`);
  const img = document.querySelector('img')!;
  hover(img);
  expect(tip()!.textContent).toBe('Blocked: use Show images');
  Object.defineProperty(img, 'complete', { value: false, configurable: true });
  patch([{ s: 'b', i: 0, a: 'src', v: base + 'i/tok' }]);
  await Promise.resolve();
  expect(img.getAttribute('data-mw-image-state')).toBe('loading');
  expect(tip()!.textContent).toBe('Loading image…');
  Object.defineProperty(img, 'complete', { value: true, configurable: true });
  Object.defineProperty(img, 'naturalWidth', { value: 10, configurable: true });
  img.dispatchEvent(new Event('load'));
  expect(img.getAttribute('data-mw-image-state')).toBe('loaded');
  expect(tip()).toBeNull();
  expect(img.hasAttribute('aria-describedby')).toBe(false);
});

it('flips below when there is no room above and stays inside the viewport', () => {
  boot('<img data-mw-ph="box" alt="Edge">');
  const img = document.querySelector('img')!;
  const rect = (l: number, t: number, w: number, h: number) => ({ left: l, top: t, right: l + w, bottom: t + h, width: w, height: h, x: l, y: t, toJSON() {} }) as DOMRect;
  img.getBoundingClientRect = () => rect(-30, 2, 40, 20);
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement) { return this.classList.contains('mw-frame-tip') ? rect(0, 0, 100, 30) : rect(-30, 2, 40, 20); });
  Object.defineProperty(document.documentElement, 'clientWidth', { value: 300, configurable: true });
  Object.defineProperty(document.documentElement, 'clientHeight', { value: 200, configurable: true });
  hover(img);
  const bubble = tip()!;
  expect(Number.parseInt(bubble.style.top, 10)).toBe(28); // 22 (bottom) + 6 gap, not above
  expect(Number.parseInt(bubble.style.left, 10)).toBe(4); // clamped, not off the left edge
  vi.restoreAllMocks();
});

it('removes its listeners on pagehide and adds no network, navigation or code-eval surface', () => {
  const { listeners } = boot('<img data-mw-ph="box" alt="A">');
  const img = document.querySelector('img')!;
  listeners.get('pagehide')!({});
  hover(img);
  expect(tip()).toBeNull();
  const added = source.slice(source.indexOf('Themed image tooltip'), source.indexOf("addEventListener('message', initialize)"));
  expect(added).not.toMatch(/fetch|XMLHttpRequest|location|window\.open|innerHTML|eval\(|new Function|importScripts|\.src\s*=/);
});
