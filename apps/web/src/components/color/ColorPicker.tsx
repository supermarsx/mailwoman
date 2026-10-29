// The app colour picker (t81): one themed control for every colour a person
// picks, in place of the browser's own `<input type="color">` dialog.
//
// `ColorField` is a trigger that shows the current colour and opens an anchored
// popover with the theme's accent swatches, recent colours, a saturation and
// brightness area, a hue strip, an optional opacity strip, hex and RGB fields,
// the screen eyedropper where the browser has one, an optional "none" choice
// ("Automatic", "No colour") and a WCAG contrast hint for colours used as text
// or badges.
//
// - Placement: `placePopover` puts it under the trigger, or above when it fits
//   there instead, shifted to stay inside the (visual) viewport; it follows the
//   trigger on scroll and resize. It is a child of the trigger's wrapper, so a
//   host popover (ribbon, dialog) keeps counting focus inside it as its own,
//   and it is shown in the top layer (`popover="manual"`) where supported, so
//   no ancestor's overflow or stacking can clip it. It lives in the trigger's
//   own document, which makes it work in detached windows too.
// - Keyboard: arrows move the area (saturation/brightness) and the strips,
//   Shift or PageUp/PageDown in steps of 10, Home/End to the ends; Tab stays
//   inside the popover; Enter applies and closes; Escape puts the value back
//   and closes; focus returns to the trigger.
// - Values: dragging and typing preview live through `onPreview`; the value is
//   committed through `onChange` on Enter, Done or a click outside.
// - CSP: no style attribute is ever written; dynamic colours and positions are
//   set through the CSSOM (`element.style.setProperty`).

import { createEffect, createMemo, createSignal, createUniqueId, For, onCleanup, onMount, Show, useContext, type JSX } from 'solid-js';
import { AppContext } from '../../state/context.ts';
import { Icon, type IconName } from '../Icon.tsx';
import { loadCatalog, t } from '../../i18n/index.ts';
import { ACCENT_SHADES } from '../../theme/accents.ts';
import { ACCENTS, type AccentId } from '../../theme/accentMath.ts';
import { isThemeName, THEME_REGISTRY } from '../../theme/registry.ts';
import type { ThemeName } from '../../theme/contract.css.ts';
import type { ColorKey } from '../../theme/contrast.ts';
import { clampHue, clampUnit, contrastHint, flatten, hsvToRgb, normalizeHex, parseHex, rgbToHsv, toHex, type ContrastUse, type Hsva } from './colorMath.ts';
import { placePopover } from './placement.ts';
import { recentColors, rememberColor } from './recent.ts';
import * as css from './colorPicker.css.ts';

export interface ColorContrast {
  /** A theme colour key (`bg`, `surface`, `text` …) of the active theme, or a hex colour. */
  against: ColorKey | string | (() => ColorKey | string);
  /** `text` judges 4.5:1 (3:1 = large text), `non-text` judges 3:1 (badges, boundaries). */
  use: ContrastUse;
  /** What `against` is, for the hint ("the background"); defaults to the background. A function is read when shown. */
  againstLabel?: string | (() => string);
}

export interface ColorFieldProps {
  /** `#rrggbb` (or `#rrggbbaa` with `alpha`); `''`/`null` = the "none" choice. */
  value: string | null | undefined;
  /** Commit: Enter, Done, a click outside, or a swatch double-click. */
  onChange: (value: string | null) => void;
  /** Live preview while dragging or typing; called with the opening value on Escape. */
  onPreview?: (value: string | null) => void;
  /** Accessible name of the trigger and the popover. */
  label: string;
  /** Offer a "none" choice named this ("Automatic", "No colour", "Default"). */
  emptyLabel?: string;
  /** The colour the area starts from when the value is none. */
  fallback?: string;
  alpha?: boolean;
  contrast?: ColorContrast;
  disabled?: boolean;
  /** Whose recent colours: an account id or `admin:<name>`; defaults to the mail app's account. */
  recentOwner?: string;
  /** `ribbon`: a compact icon button for editor ribbons (the host's button class). */
  variant?: 'field' | 'ribbon';
  buttonClass?: string;
  icon?: IconName;
  /** Show the hex value as text beside the swatch (field variant). */
  showValue?: boolean;
  /** Keep the editor's selection: the trigger does not take focus on mouse down. */
  keepSelection?: boolean;
  id?: string;
  testId?: string;
  describedBy?: string;
}

const STEP = 0.01;
const BIG = 0.1;
const jsdom = typeof navigator !== 'undefined' && /jsdom/i.test(navigator.userAgent);
const popoverSupported = typeof HTMLElement !== 'undefined' && typeof (HTMLElement.prototype as { showPopover?: unknown }).showPopover === 'function';

function activeTheme(doc: Document): ThemeName {
  const name = doc.documentElement.getAttribute('data-theme');
  return isThemeName(name) ? name : 'light';
}

/** The hex value of `against` in the active theme, or `null` when it is no colour. */
export function resolveAgainst(against: ColorKey | string, theme: ThemeName): string | null {
  const palette = THEME_REGISTRY[theme].palette as Record<string, string>;
  const value = against in palette ? palette[against] : against;
  return value ?? null;
}

/** Set (or clear) a custom property through the CSSOM; never a style attribute. */
function paint(el: HTMLElement | undefined, property: string, value: string | null): void {
  if (!el) return;
  if (value === null) el.style.removeProperty(property);
  else el.style.setProperty(property, value);
}

function focusables(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>('button:not([disabled]),input:not([disabled]),[tabindex="0"]')).filter((el) => !el.closest('[hidden]'));
}

export function ColorField(props: ColorFieldProps): JSX.Element {
  const uid = createUniqueId();
  const popId = `mw-color-${uid}`;
  const titleId = `${popId}-title`;
  const hexId = `${popId}-hex`;
  const errorId = `${popId}-error`;
  const [open, setOpen] = createSignal(false);
  const [hsva, setHsva] = createSignal<Hsva>({ h: 0, s: 0, v: 0, a: 1 });
  const [none, setNone] = createSignal(false);
  const [hexText, setHexText] = createSignal('');
  const [hexInvalid, setHexInvalid] = createSignal(false);
  const [recent, setRecent] = createSignal<string[]>([]);
  const [theme, setTheme] = createSignal<ThemeName>('light');
  let rootEl: HTMLDivElement | undefined;
  let triggerEl: HTMLButtonElement | undefined;
  let popEl: HTMLDivElement | undefined;
  let areaEl: HTMLDivElement | undefined;
  let areaCanvas: HTMLCanvasElement | undefined;
  let hueCanvas: HTMLCanvasElement | undefined;
  let alphaCanvas: HTMLCanvasElement | undefined;
  let chipEl: HTMLSpanElement | undefined;
  let previewEl: HTMLSpanElement | undefined;
  let areaThumb: HTMLSpanElement | undefined;
  let hueThumb: HTMLSpanElement | undefined;
  let alphaThumb: HTMLSpanElement | undefined;
  let initial: string | null = null;
  let detach: (() => void) | null = null;

  onMount(() => {
    void loadCatalog('color-picker');
    void loadCatalog('appearance');
  });

  const alpha = (): boolean => props.alpha === true;
  const current = (): string | null => {
    const raw = props.value;
    if (!raw) return null;
    const parsed = parseHex(raw, alpha());
    return parsed ? toHex(parsed, alpha()) : null;
  };
  const draft = createMemo((): string | null => (none() ? null : toHex(hsvToRgb(hsva()), alpha())));
  // The mail app's account when no owner is given (the admin console passes `admin:<name>`).
  const app = useContext(AppContext);
  const owner = (): string => props.recentOwner ?? app?.accountId?.() ?? 'local';
  const doc = (): Document => rootEl?.ownerDocument ?? document;
  const view = (): Window => doc().defaultView ?? window;

  // The trigger swatch shows the committed value (or the live draft while open).
  createEffect(() => {
    const shown = open() ? draft() : current();
    paint(chipEl, 'background-color', shown);
  });
  createEffect(() => paint(previewEl, 'background-color', draft()));

  const setFromHex = (hex: string, keepHue = true): boolean => {
    const parsed = parseHex(hex, alpha());
    if (!parsed) return false;
    const next = rgbToHsv(parsed);
    const prev = hsva();
    setHsva(keepHue && next.s === 0 ? { ...next, h: prev.h } : next);
    setNone(false);
    return true;
  };

  const preview = (): void => {
    setHexText(draft() ?? '');
    setHexInvalid(false);
    props.onPreview?.(draft());
  };
  const update = (patch: Partial<Hsva>): void => {
    setHsva({ ...hsva(), ...patch });
    setNone(false);
    preview();
  };

  function place(): void {
    if (!triggerEl || !popEl || !open()) return;
    const w = view();
    const vv = w.visualViewport;
    const anchor = triggerEl.getBoundingClientRect();
    const size = { width: popEl.offsetWidth, height: popEl.scrollHeight };
    const rtl = w.getComputedStyle(triggerEl).direction === 'rtl';
    const spot = placePopover(
      { left: anchor.left, top: anchor.top, width: anchor.width, height: anchor.height },
      size,
      { left: vv?.offsetLeft ?? 0, top: vv?.offsetTop ?? 0, width: vv?.width ?? w.innerWidth, height: vv?.height ?? w.innerHeight },
      { rtl },
    );
    paint(popEl, 'left', `${spot.left}px`);
    paint(popEl, 'top', `${spot.top}px`);
    paint(popEl, 'max-height', `${spot.maxHeight}px`);
    popEl.setAttribute('data-side', spot.side);
  }

  function drawCanvases(): void {
    if (jsdom) return;
    const ratio = view().devicePixelRatio || 1;
    const size = (canvas: HTMLCanvasElement | undefined): CanvasRenderingContext2D | null => {
      if (!canvas) return null;
      const w = Math.max(1, Math.round(canvas.clientWidth * ratio));
      const h = Math.max(1, Math.round(canvas.clientHeight * ratio));
      if (canvas.width !== w) canvas.width = w;
      if (canvas.height !== h) canvas.height = h;
      return canvas.getContext('2d');
    };
    const { h } = hsva();
    const area = size(areaCanvas);
    if (area && areaCanvas) {
      const { width, height } = areaCanvas;
      area.fillStyle = toHex(hsvToRgb({ h, s: 1, v: 1, a: 1 }));
      area.fillRect(0, 0, width, height);
      const white = area.createLinearGradient(0, 0, width, 0);
      white.addColorStop(0, 'rgba(255,255,255,1)');
      white.addColorStop(1, 'rgba(255,255,255,0)');
      area.fillStyle = white;
      area.fillRect(0, 0, width, height);
      const black = area.createLinearGradient(0, 0, 0, height);
      black.addColorStop(0, 'rgba(0,0,0,0)');
      black.addColorStop(1, 'rgba(0,0,0,1)');
      area.fillStyle = black;
      area.fillRect(0, 0, width, height);
    }
    const hue = size(hueCanvas);
    if (hue && hueCanvas) {
      const g = hue.createLinearGradient(0, 0, hueCanvas.width, 0);
      for (let deg = 0; deg <= 360; deg += 30) g.addColorStop(deg / 360, toHex(hsvToRgb({ h: deg, s: 1, v: 1, a: 1 })));
      hue.fillStyle = g;
      hue.fillRect(0, 0, hueCanvas.width, hueCanvas.height);
    }
    const strip = size(alphaCanvas);
    if (strip && alphaCanvas) {
      const { width, height } = alphaCanvas;
      const palette = THEME_REGISTRY[theme()].palette;
      const cell = Math.max(4, Math.round(height / 2));
      for (let x = 0; x < width; x += cell) for (let y = 0; y < height; y += cell) {
        strip.fillStyle = (x / cell + y / cell) % 2 === 0 ? palette.surface : palette.border;
        strip.fillRect(x, y, cell, cell);
      }
      const rgb = hsvToRgb({ ...hsva(), a: 1 });
      const g = strip.createLinearGradient(0, 0, width, 0);
      g.addColorStop(0, `rgba(${rgb.r},${rgb.g},${rgb.b},0)`);
      g.addColorStop(1, `rgba(${rgb.r},${rgb.g},${rgb.b},1)`);
      strip.fillStyle = g;
      strip.fillRect(0, 0, width, height);
    }
  }

  // Thumbs and canvases follow the value.
  createEffect(() => {
    const { h, s, v, a } = hsva();
    const off = none();
    if (!open()) return;
    paint(areaThumb, 'left', `${s * 100}%`);
    paint(areaThumb, 'top', `${(1 - v) * 100}%`);
    paint(hueThumb, 'left', `${(h / 360) * 100}%`);
    paint(alphaThumb, 'left', `${a * 100}%`);
    if (areaThumb) areaThumb.hidden = off;
    drawCanvases();
  });

  function show(): void {
    if (props.disabled || open()) return;
    initial = current();
    setTheme(activeTheme(doc()));
    const start = initial ?? props.fallback ?? (props.emptyLabel ? null : '#000000');
    if (start) setFromHex(start, false);
    else setHsva({ h: 0, s: 0, v: 0, a: 1 });
    setNone(initial === null && !!props.emptyLabel);
    setHexText(initial ?? '');
    setHexInvalid(false);
    setRecent(recentColors(owner()));
    setOpen(true);
    if (popEl && popoverSupported && popEl.hasAttribute('popover')) {
      try { popEl.showPopover(); } catch { /* not connected or already shown */ }
    }
    place();
    drawCanvases();
    view().requestAnimationFrame(() => { place(); drawCanvases(); areaEl?.focus({ preventScroll: true }); });
    const d = doc();
    const w = view();
    const outside = (event: PointerEvent): void => {
      const target = event.target;
      if (target && typeof (target as Node).nodeType === 'number' && rootEl?.contains(target as Node)) return;
      hide('commit', false);
    };
    const follow = (): void => place();
    d.addEventListener('pointerdown', outside, true);
    d.addEventListener('scroll', follow, true);
    w.addEventListener('resize', follow);
    w.visualViewport?.addEventListener('resize', follow);
    detach = () => {
      d.removeEventListener('pointerdown', outside, true);
      d.removeEventListener('scroll', follow, true);
      w.removeEventListener('resize', follow);
      w.visualViewport?.removeEventListener('resize', follow);
    };
  }

  function hide(mode: 'commit' | 'cancel', restoreFocus = true): void {
    if (!open()) return;
    detach?.();
    detach = null;
    const value = draft();
    if (popEl && popoverSupported && popEl.hasAttribute('popover')) {
      try { popEl.hidePopover(); } catch { /* already hidden */ }
    }
    setOpen(false);
    if (mode === 'cancel') {
      if (props.onPreview && value !== initial) props.onPreview(initial);
    } else {
      if (value !== null) setRecent(rememberColor(owner(), value));
      if (value !== initial) props.onChange(value);
    }
    if (restoreFocus) triggerEl?.focus({ preventScroll: true });
  }
  onCleanup(() => detach?.());

  // --- pointer and keyboard on the area and strips ---------------------------

  function fromPointer(el: HTMLElement, event: PointerEvent, apply: (x: number, y: number) => void): void {
    const rect = el.getBoundingClientRect();
    const x = rect.width ? clampUnit((event.clientX - rect.left) / rect.width) : 0;
    const y = rect.height ? clampUnit((event.clientY - rect.top) / rect.height) : 0;
    apply(x, y);
  }
  function drag(apply: (x: number, y: number) => void) {
    return (event: PointerEvent): void => {
      if (event.button !== 0) return;
      const el = event.currentTarget as HTMLElement;
      event.preventDefault();
      el.focus({ preventScroll: true });
      try { el.setPointerCapture(event.pointerId); } catch { /* synthetic event */ }
      fromPointer(el, event, apply);
      const move = (e: PointerEvent): void => fromPointer(el, e, apply);
      const up = (): void => {
        el.removeEventListener('pointermove', move);
        el.removeEventListener('pointerup', up);
        el.removeEventListener('pointercancel', up);
      };
      el.addEventListener('pointermove', move);
      el.addEventListener('pointerup', up);
      el.addEventListener('pointercancel', up);
    };
  }

  /** Arrow/Page/Home/End step of a 0..1 axis; `null` when the key is not for that axis. */
  function step(key: string, shift: boolean, axis: 'x' | 'y'): number | null {
    const delta = shift ? BIG : STEP;
    if (axis === 'x') {
      if (key === 'ArrowRight') return delta;
      if (key === 'ArrowLeft') return -delta;
    } else {
      if (key === 'ArrowUp') return delta;
      if (key === 'ArrowDown') return -delta;
    }
    return null;
  }

  function areaKey(event: KeyboardEvent): void {
    const { s, v } = hsva();
    const dx = step(event.key, event.shiftKey, 'x');
    const dy = step(event.key, event.shiftKey, 'y');
    let next: Partial<Hsva> | null = null;
    if (dx !== null) next = { s: clampUnit(s + dx) };
    else if (dy !== null) next = { v: clampUnit(v + dy) };
    else if (event.key === 'PageUp') next = { v: clampUnit(v + BIG) };
    else if (event.key === 'PageDown') next = { v: clampUnit(v - BIG) };
    else if (event.key === 'Home') next = { s: 0 };
    else if (event.key === 'End') next = { s: 1 };
    if (!next) return;
    event.preventDefault();
    event.stopPropagation();
    update(next);
  }
  function hueKey(event: KeyboardEvent): void {
    const { h } = hsva();
    const delta = event.shiftKey ? 10 : 1;
    const map: Record<string, number> = { ArrowRight: h + delta, ArrowUp: h + delta, ArrowLeft: h - delta, ArrowDown: h - delta, PageUp: h + 10, PageDown: h - 10, Home: 0, End: 360 };
    const next = map[event.key];
    if (next === undefined) return;
    event.preventDefault();
    event.stopPropagation();
    update({ h: clampHue(next) });
  }
  function alphaKey(event: KeyboardEvent): void {
    const { a } = hsva();
    const delta = event.shiftKey ? BIG : STEP;
    const map: Record<string, number> = { ArrowRight: a + delta, ArrowUp: a + delta, ArrowLeft: a - delta, ArrowDown: a - delta, PageUp: a + BIG, PageDown: a - BIG, Home: 0, End: 1 };
    const next = map[event.key];
    if (next === undefined) return;
    event.preventDefault();
    event.stopPropagation();
    update({ a: Math.round(clampUnit(next) * 100) / 100 });
  }

  function popKey(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      hide('cancel');
      return;
    }
    if (event.key === 'Enter' && !(event.target instanceof HTMLButtonElement)) {
      event.preventDefault();
      event.stopPropagation();
      if (hexInvalid()) return;
      hide('commit');
      return;
    }
    if (event.key === 'Tab' && popEl) {
      const items = focusables(popEl);
      if (items.length === 0) return;
      const first = items[0]!;
      const last = items[items.length - 1]!;
      const active = doc().activeElement;
      if (event.shiftKey && (active === first || !popEl.contains(active))) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && (active === last || !popEl.contains(active))) {
        event.preventDefault();
        first.focus();
      }
      event.stopPropagation();
    }
  }

  // --- fields -----------------------------------------------------------------

  function onHex(text: string): void {
    setHexText(text);
    if (text.trim() === '' && props.emptyLabel) {
      setNone(true);
      setHexInvalid(false);
      props.onPreview?.(null);
      return;
    }
    if (setFromHex(text)) {
      setHexInvalid(false);
      props.onPreview?.(draft());
    } else setHexInvalid(true);
  }
  const rgb = createMemo(() => hsvToRgb(hsva()));
  function onChannel(channel: 'r' | 'g' | 'b', text: string): void {
    const n = Number(text);
    if (text.trim() === '' || !Number.isInteger(n) || n < 0 || n > 255) return;
    const next = { ...rgb(), [channel]: n };
    const hsv = rgbToHsv(next);
    setHsva(hsv.s === 0 ? { ...hsv, h: hsva().h } : hsv);
    setNone(false);
    preview();
  }

  async function eyedrop(): Promise<void> {
    const Ctor = (view() as unknown as { EyeDropper?: new () => { open: () => Promise<{ sRGBHex: string }> } }).EyeDropper;
    if (!Ctor) return;
    try {
      const result = await new Ctor().open();
      if (setFromHex(result.sRGBHex)) preview();
    } catch {
      /* The person cancelled the eyedropper. */
    }
    popEl?.querySelector<HTMLElement>('[data-color-eyedropper]')?.focus();
  }
  const hasEyedropper = (): boolean => typeof (view() as unknown as { EyeDropper?: unknown }).EyeDropper === 'function';

  const presets = createMemo(() => {
    const shades = ACCENT_SHADES[theme()];
    const palette = THEME_REGISTRY[theme()].palette;
    const accents = ACCENTS.map((def) => ({ hex: shades[def.id as AccentId].accent, name: t(def.labelId) }));
    return [
      ...accents,
      { hex: palette.text, name: t('color-picker-theme-text') },
      { hex: palette.surface, name: t('color-picker-theme-surface') },
    ]
      .map((p) => ({ ...p, hex: normalizeHex(p.hex) ?? '' }))
      .filter((p, i, all) => p.hex !== '' && all.findIndex((q) => q.hex === p.hex) === i);
  });

  const pick = (hex: string): void => {
    if (setFromHex(hex, false)) preview();
  };
  const pickNone = (): void => {
    setNone(true);
    setHexText('');
    setHexInvalid(false);
    props.onPreview?.(null);
  };

  const contrast = createMemo(() => {
    const c = props.contrast;
    const value = draft();
    if (!c || !value) return null;
    const raw = typeof c.against === 'function' ? c.against() : c.against;
    const against = resolveAgainst(raw, theme());
    if (!against) return null;
    const backdrop = parseHex(against);
    const color = parseHex(value, true);
    if (!backdrop || !color) return null;
    const flat = color.a < 1 ? toHex(flatten(color, backdrop)) : toHex(color);
    return contrastHint(flat, toHex(backdrop), c.use);
  });

  const againstText = (): string => {
    const label = props.contrast?.againstLabel;
    return typeof label === 'function' ? label() : label ?? t('color-picker-against-background');
  };
  const valueText = (): string => current() ?? props.emptyLabel ?? t('color-picker-unset');

  const triggerClass = (): string =>
    props.variant === 'ribbon' ? `${props.buttonClass ?? 'compose-rt__btn'} ${css.triggerBare}` : props.buttonClass ?? css.trigger;

  return (
    <div ref={rootEl} class={css.root} data-testid={props.testId} data-color-field="">
      <button
        ref={triggerEl}
        id={props.id}
        type="button"
        class={triggerClass()}
        disabled={props.disabled}
        aria-haspopup="dialog"
        aria-expanded={open() ? 'true' : 'false'}
        aria-controls={popId}
        aria-label={t('color-picker-trigger', { label: props.label, value: valueText() })}
        aria-describedby={props.describedBy}
        data-tooltip={props.variant === 'ribbon' ? props.label : undefined}
        data-detached-local=""
        data-color-value={current() ?? ''}
        onMouseDown={(event) => { if (props.keepSelection) event.preventDefault(); }}
        onClick={() => (open() ? hide('commit') : show())}
      >
        <Show
          when={props.variant === 'ribbon'}
          fallback={<>
            <span ref={chipEl} class={css.chip} aria-hidden="true" />
            <Show when={props.showValue !== false}><span>{valueText()}</span></Show>
          </>}
        >
          <span class={css.stack} aria-hidden="true">
            <Icon name={props.icon ?? 'paint'} />
            <span ref={chipEl} class={css.bar} />
          </span>
        </Show>
      </button>
      <div
        ref={(el) => {
          popEl = el;
          if (popoverSupported) el.setAttribute('popover', 'manual');
        }}
        id={popId}
        class={css.popover}
        role="dialog"
        aria-modal="false"
        aria-labelledby={titleId}
        hidden={!open()}
        data-color-popover=""
        onKeyDown={popKey}
      >
        <Show when={open()}>
          <p id={titleId} class={css.title}>{props.label}</p>
          <div
            ref={areaEl}
            class={css.area}
            role="slider"
            tabIndex={0}
            aria-label={t('color-picker-area')}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={Math.round(hsva().s * 100)}
            aria-valuetext={t('color-picker-area-value', { s: Math.round(hsva().s * 100), v: Math.round(hsva().v * 100) })}
            data-color-area=""
            onPointerDown={drag((x, y) => update({ s: x, v: 1 - y }))}
            onKeyDown={areaKey}
          >
            <canvas ref={areaCanvas} class={css.canvas} aria-hidden="true" />
            <span ref={areaThumb} class={css.thumb} aria-hidden="true" />
          </div>
          <div
            class={css.strip}
            role="slider"
            tabIndex={0}
            aria-label={t('color-picker-hue')}
            aria-valuemin={0}
            aria-valuemax={360}
            aria-valuenow={Math.round(hsva().h)}
            aria-valuetext={t('color-picker-hue-value', { h: Math.round(hsva().h) })}
            data-color-hue=""
            onPointerDown={drag((x) => update({ h: x * 360 }))}
            onKeyDown={hueKey}
          >
            <canvas ref={hueCanvas} class={css.canvas} aria-hidden="true" />
            <span ref={hueThumb} class={css.stripThumb} aria-hidden="true" />
          </div>
          <Show when={alpha()}>
            <div
              class={css.strip}
              role="slider"
              tabIndex={0}
              aria-label={t('color-picker-alpha')}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={Math.round(hsva().a * 100)}
              aria-valuetext={t('color-picker-alpha-value', { a: Math.round(hsva().a * 100) })}
              data-color-alpha=""
              onPointerDown={drag((x) => update({ a: Math.round(x * 100) / 100 }))}
              onKeyDown={alphaKey}
            >
              <canvas ref={alphaCanvas} class={css.canvas} aria-hidden="true" />
              <span ref={alphaThumb} class={css.stripThumb} aria-hidden="true" />
            </div>
          </Show>
          <div class={css.row}>
            <span ref={previewEl} class={css.preview} role="img" aria-label={t('color-picker-preview', { value: draft() ?? props.emptyLabel ?? t('color-picker-unset') })} data-color-preview="" />
            <div class={`${css.fields} ${css.grow}`}>
              <label class={css.field}>
                {t('color-picker-hex')}
                <input
                  id={hexId}
                  class={css.input}
                  type="text"
                  inputMode="text"
                  autocomplete="off"
                  spellcheck={false}
                  maxLength={alpha() ? 9 : 7}
                  value={hexText()}
                  aria-invalid={hexInvalid() ? 'true' : 'false'}
                  aria-describedby={hexInvalid() ? errorId : undefined}
                  data-color-hex=""
                  onInput={(event) => onHex(event.currentTarget.value)}
                  onBlur={() => { if (!hexInvalid()) setHexText(draft() ?? ''); }}
                />
              </label>
              <For each={['r', 'g', 'b'] as const}>
                {(channel) => (
                  <label class={css.field}>
                    <span aria-hidden="true">{t(`color-picker-${channel}-short`)}</span>
                    <input
                      class={css.input}
                      type="number"
                      min="0"
                      max="255"
                      step="1"
                      inputMode="numeric"
                      aria-label={t(`color-picker-${channel}`)}
                      value={none() ? '' : rgb()[channel]}
                      data-color-channel={channel}
                      onInput={(event) => onChannel(channel, event.currentTarget.value)}
                    />
                  </label>
                )}
              </For>
            </div>
          </div>
          <Show when={hexInvalid()}>
            <p id={errorId} class={css.error} role="alert">{t(alpha() ? 'color-picker-hex-invalid-alpha' : 'color-picker-hex-invalid')}</p>
          </Show>
          <Show when={contrast()}>
            {(hint) => (
              <p class={css.hint} data-color-contrast={hint().level} aria-live="polite">
                <span>{t('color-picker-contrast', { ratio: hint().ratio.toFixed(2), against: againstText() })}</span>
                <span class={css.badge} data-level={hint().level}>{t(`color-picker-contrast-${hint().level}`)}</span>
              </p>
            )}
          </Show>
          <div class={css.swatches} role="group" aria-label={t('color-picker-theme')}>
            <p class={css.groupLabel} aria-hidden="true">{t('color-picker-theme')}</p>
            <Show when={props.emptyLabel}>
              {(label) => (
                <button type="button" class={css.swatch} aria-pressed={none()} aria-label={label()} data-tooltip={label()} data-color-none="" onClick={pickNone}>
                  <Icon name="disable" />
                </button>
              )}
            </Show>
            <For each={presets()}>{(p) => <Swatch hex={p.hex} name={p.name} pressed={draft() === p.hex} onPick={pick} onCommit={() => { pick(p.hex); hide('commit'); }} />}</For>
          </div>
          <Show when={recent().length > 0}>
            <div class={css.swatches} role="group" aria-label={t('color-picker-recent')}>
              <p class={css.groupLabel} aria-hidden="true">{t('color-picker-recent')}</p>
              <For each={recent().filter((hex) => alpha() || hex.length === 7)}>{(hex) => <Swatch hex={hex} name={hex} pressed={draft() === hex} onPick={pick} onCommit={() => { pick(hex); hide('commit'); }} />}</For>
            </div>
          </Show>
          <div class={css.actions}>
            <Show when={hasEyedropper()}>
              <button type="button" class={css.button} data-color-eyedropper="" onClick={() => void eyedrop()}>
                <Icon name="eyedropper" />
                <span>{t('color-picker-eyedropper')}</span>
              </button>
            </Show>
            <span class={css.grow} />
            <button type="button" class={css.button} data-color-cancel="" onClick={() => hide('cancel')}>{t('color-picker-cancel')}</button>
            <button type="button" class={css.primary} data-color-done="" disabled={hexInvalid()} onClick={() => hide('commit')}>{t('color-picker-done')}</button>
          </div>
        </Show>
      </div>
    </div>
  );
}

function Swatch(props: { hex: string; name: string; pressed: boolean; onPick: (hex: string) => void; onCommit: () => void }): JSX.Element {
  let el: HTMLButtonElement | undefined;
  createEffect(() => paint(el, 'background-color', props.hex));
  const label = (): string => (props.name === props.hex ? props.hex : t('color-picker-swatch', { name: props.name, hex: props.hex }));
  return (
    <button
      ref={el}
      type="button"
      class={css.swatch}
      aria-pressed={props.pressed}
      aria-label={label()}
      data-tooltip={label()}
      data-color-swatch={props.hex}
      onClick={() => props.onPick(props.hex)}
      onDblClick={() => props.onCommit()}
    />
  );
}
