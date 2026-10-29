// The boot splash (t52): the one placeholder a visitor sees between the first
// paint and the app's own UI.
//
// `index.html` carries a static `#mw-boot` element and loads `src/boot/boot.js`
// as a classic script at the top of `<head>`. That script runs before the first
// paint and, from what this module stored on the previous visit:
//   • puts the last-applied theme, density, accent, badge colour and font on
//     `<html>`, so the first painted frame already has the user's theme;
//   • sets `data-mw-boot` on `<html>` to `mail` (a session was open last time:
//     draw the mailbox shell) or `plain` (sign-in, consent: draw nothing but the
//     themed page). Without that attribute the CSS keeps `#mw-boot` hidden, so a
//     failed or missing boot script degrades to the old blank page;
//   • copies the geometry of the mailbox shell measured on the previous visit
//     into `--mwb-*` custom properties, so the placeholder has the user's own
//     sidebar width, bar heights, list width and row height.
//
// The splash is a fixed, opaque layer over `#root`. The real UI mounts under it
// and the splash fades out over it, so the hand-over moves no box (no layout
// shift) and there is exactly one placeholder stage, whatever the app renders
// underneath while it loads.
//
// Storage is best-effort: every access is guarded, and with nothing stored the
// boot script leaves the CSS defaults (the Dark pack, t32) in place, which is
// also what the app applies with nothing stored.

/** Theme snapshot read by `boot.js`. Keep the field names in step with it. */
export const BOOT_THEME_KEY = 'mw.boot.theme';
/** `'1'` while a mail session is open; read by `boot.js` to choose the variant. */
export const BOOT_SESSION_KEY = 'mw.boot.session';
/** Mailbox shell geometry measured on the previous visit. */
export const BOOT_GEOMETRY_KEY = 'mw.boot.geometry';

/** How long the splash takes to fade out over the mounted UI. */
const FADE_MS = 140;

export interface BootThemeSnapshot {
  readonly v: 1;
  /** The pack that was applied. */
  readonly theme: string;
  readonly appearance: 'light' | 'dark';
  readonly mode: 'fixed' | 'system' | 'schedule';
  readonly light: string;
  readonly dark: string;
  readonly schedule: { readonly darkStart: string; readonly darkEnd: string };
  readonly density: string;
  readonly accent: string;
  readonly badge: string;
  /** The `--mw-ui-font` override, or `''`. */
  readonly font: string;
  /** Custom-accent properties set inline for `theme` (they are per pack). */
  readonly props: Readonly<Record<string, string>>;
}

function store(): Storage | null {
  try {
    return typeof localStorage === 'undefined' ? null : localStorage;
  } catch {
    return null;
  }
}

function write(key: string, value: string | null): void {
  const s = store();
  if (s === null) return;
  try {
    if (value === null) s.removeItem(key);
    else s.setItem(key, value);
  } catch {
    /* quota / private mode: the next boot just has less to go on */
  }
}

/** Record what the theme layer just put on `<html>`, for the next boot. */
export function rememberBootTheme(snapshot: BootThemeSnapshot): void {
  write(BOOT_THEME_KEY, JSON.stringify(snapshot));
}

/** Remember whether a mail session is open, so the next boot draws the right shape. */
export function rememberBootSession(open: boolean): void {
  write(BOOT_SESSION_KEY, open ? '1' : null);
  if (!open) write(BOOT_GEOMETRY_KEY, null);
}

const px = (n: number): number => Math.max(0, Math.round(n));

function height(el: Element | null): number {
  return el === null ? 0 : px(el.getBoundingClientRect().height);
}

/**
 * Measure the mounted mailbox shell and store it for the next boot. Called once
 * the shell has settled; later layout changes are picked up on the next visit.
 */
export function rememberBootGeometry(): void {
  if (typeof document === 'undefined' || typeof window === 'undefined') return;
  const workspace = document.querySelector('.shell .mail-workspace');
  const body = document.querySelector('.shell .workspace-body');
  const pane = document.querySelector('.shell .mail-pane');
  if (workspace === null || body === null || pane === null || (pane as HTMLElement).hidden) return;
  const rtl = getComputedStyle(document.documentElement).direction === 'rtl';
  const ws = workspace.getBoundingClientRect();
  const topbar = document.querySelector('.shell .mail-topbar');
  const scroll = pane.querySelector('.list__scroll');
  const row = pane.querySelector('.list__row');
  const paneBox = pane.getBoundingClientRect();
  const geometry = {
    v: 1,
    wide: window.innerWidth >= 768,
    bar: height(document.querySelector('.shell .shell__bar')),
    // Everything before the workspace on the inline axis: the sidebar and its rule.
    side: px(rtl ? window.innerWidth - ws.right : ws.left),
    tabs: topbar === null ? 0 : px(topbar.getBoundingClientRect().top - ws.top),
    top: px(body.getBoundingClientRect().top - ws.top),
    // -1: the list fills the workspace, whatever its width at the next boot.
    list: paneBox.width >= body.getBoundingClientRect().width - 1 ? -1 : px(paneBox.width),
    lead: scroll === null ? 0 : px(scroll.getBoundingClientRect().top - paneBox.top),
    row: height(row),
  };
  // A measurement taken mid-mount (nothing laid out yet) would be worse than the defaults.
  if (geometry.top === 0 || paneBox.width === 0 || geometry.row === 0) return;
  write(BOOT_GEOMETRY_KEY, JSON.stringify(geometry));
}

function reducedMotion(): boolean {
  const root = document.documentElement;
  if (root.hasAttribute('data-reduced-motion')) return true;
  try {
    return typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
  } catch {
    return false;
  }
}

let released = false;

/** True while the boot splash is still on screen (or fading out). */
export function bootSplashActive(): boolean {
  return !released && typeof document !== 'undefined' && document.documentElement.hasAttribute('data-mw-boot');
}

/**
 * Fade the splash out over whatever has mounted under it, then remove it.
 * Idempotent: the first caller wins, later calls do nothing.
 */
export function releaseBootSplash(): void {
  if (released || typeof document === 'undefined') return;
  released = true;
  const root = document.documentElement;
  const splash = document.getElementById('mw-boot');
  const finish = (): void => {
    splash?.remove();
    root.removeAttribute('data-mw-boot');
    root.removeAttribute('data-mw-boot-leaving');
  };
  if (splash === null || !root.hasAttribute('data-mw-boot') || reducedMotion()) {
    finish();
    return;
  }
  // Two frames: the UI underneath gets painted once before the fade starts.
  requestAnimationFrame(() => requestAnimationFrame(() => {
    root.setAttribute('data-mw-boot-leaving', '');
    setTimeout(finish, FADE_MS + 20);
  }));
}

/** Test seam: forget that the splash was released. */
export function resetBootSplashForTests(): void {
  released = false;
}

/**
 * Give every pulsing placeholder under `el` the same phase as the document
 * timeline. The splash does the same in `boot.js`, so a placeholder that takes
 * over from the splash pulses in step with it instead of restarting.
 */
export function alignPulse(el: Element): void {
  if (typeof el.getAnimations !== 'function') return;
  try {
    for (const animation of el.getAnimations({ subtree: true })) {
      if (animation.effect?.getTiming().iterations === Infinity) animation.startTime = 0;
    }
  } catch {
    /* an engine that refuses to seek a CSS animation keeps its own phase */
  }
}
