// The Layout ribbon tab: columns, content width, spacing, page colour, themes,
// default font and direction. It edits a MessageLayout held by the composer
// (messageLayout.ts writes it around the part above a reply quote) and inserts
// column tables into whichever editor is active. Every group is behind its
// `layout.*` feature flag; plain-text mode disables the controls with a tooltip.
import { For, Show, createSignal, type JSX } from 'solid-js';
import './compose-layout.css';
import { RibbonGroup, RibbonItem } from '../../bars/index.ts';
import { ColorField } from '../../color/ColorPicker.tsx';
import { Icon } from '../../Icon.tsx';
import { t, loadCatalog } from '../../../i18n/index.ts';
import { COMPOSER_FONTS } from '../fontCatalog.ts';
import { RibbonPopover } from '../ribbonParts.tsx';
import { useComposeFeature } from '../composeFeatures.ts';
import { COLUMN_PRESETS, MAX_COLUMNS, buildColumnsHtml, equalWidths, mergeColumnsInHtml, type ColumnPreset } from './columns.ts';
import { THEMES, SPACING, type SpacingPreset, type ThemeId } from './themes.ts';
import { resolveTheme, type MessageLayout } from './messageLayout.ts';

export interface LayoutGroupsProps {
  layout: () => MessageLayout;
  update: (change: (current: MessageLayout) => MessageLayout) => void;
  /** Insert HTML at the caret (columns). */
  insertHtml: (html: string) => void;
  /** The editor's current HTML and a way to replace it (Merge columns). */
  currentHtml: () => string;
  replaceHtml: (html: string) => void;
  /** Plain-text mode: nothing here can apply. */
  plain: () => boolean;
  /** Stored default font ("family|size|#color") and a way to save it. */
  savedFont?: () => string;
  saveFont?: (value: string) => void | Promise<unknown>;
  report?: (message: string) => void;
}

const WIDTHS = ['full', 600, 640, 700, 800] as const;
const PADDINGS = [0, 4, 8, 12, 16, 24] as const;
const LINE_LENGTHS: readonly { id: string; width: number | null }[] = [{ id: 'off', width: null }, { id: 'narrow', width: 520 }, { id: 'comfortable', width: 640 }, { id: 'wide', width: 760 }];
const SIZES = [10, 11, 12, 13, 14, 16, 18, 20, 24];

export function encodeFont(f: { family?: string | undefined; size?: number | undefined; color?: string | undefined }): string {
  return `${f.family ?? ''}|${f.size ?? ''}|${f.color ?? ''}`;
}
export function decodeFont(v: string): { family?: string; size?: number; color?: string } {
  const [family = '', size = '', color = ''] = v.split('|');
  const out: { family?: string; size?: number; color?: string } = {};
  if (family) out.family = family;
  if (/^\d+$/.test(size)) out.size = Number(size);
  if (/^#[0-9a-f]{6}$/i.test(color)) out.color = color;
  return out;
}

export function LayoutGroups(props: LayoutGroupsProps): JSX.Element {
  void loadCatalog('compose-layout');
  void loadCatalog('color-picker');
  const fColumns = useComposeFeature('layout.columns');
  const fWidth = useComposeFeature('layout.contentWidth');
  const fPage = useComposeFeature('layout.pageColor');
  const fThemes = useComposeFeature('layout.themes');
  const fFont = useComposeFeature('layout.defaultFont');
  const fSpacing = useComposeFeature('layout.spacing');
  const [count, setCount] = createSignal(2);
  const [gutter, setGutter] = createSignal(16);
  const [customWidth, setCustomWidth] = createSignal(600);
  const l = props.layout;
  const off = (): boolean => props.plain();
  const tip = (): string | undefined => (off() ? t('layout-plain-disabled') : undefined);
  const set = (patch: Partial<MessageLayout>): void => props.update((cur) => ({ ...cur, ...patch }));
  const clear = (...keys: (keyof MessageLayout)[]): void => props.update((cur) => { const n = { ...cur }; for (const k of keys) delete n[k]; return n; });
  const widthValue = (): string => { const w = l().width; return w === undefined ? 'theme' : String(w); };
  const insertColumns = (widths: number[]): void => props.insertHtml(buildColumnsHtml({ widths, gutter: gutter(), width: typeof l().width === 'number' ? (l().width as number) : 600 }));
  const merge = (): void => {
    const out = mergeColumnsInHtml(props.currentHtml());
    if (out.merged === 0) { props.report?.(t('layout-merge-none')); return; }
    props.replaceHtml(out.html);
  };
  const button = (label: string, run: () => void, testId: string, pressed?: () => boolean, glyph?: string): JSX.Element =>
    <button type="button" class="compose-rt__btn" disabled={off()} title={tip() ?? label} aria-label={label} aria-pressed={pressed?.()} data-testid={testId}
      onMouseDown={(e) => e.preventDefault()} onClick={run}>
      <Show when={glyph} fallback={<Icon name="table" />}><span aria-hidden="true">{glyph}</span></Show>
      <span class="compose-rib__text">{label}</span>
    </button>;
  const font = () => l().font ?? {};
  const fontStack = (): string => font().family ?? resolveTheme(l()).fontFamily;

  return <>
    <Show when={fColumns()}>
      <RibbonGroup label={t('layout-group-columns')} priority={100} icon="table" testId="layout-group-columns">
        <RibbonItem>{button(t('layout-columns-1'), () => insertColumns(COLUMN_PRESETS.one), 'layout-cols-1')}</RibbonItem>
        <RibbonItem>{button(t('layout-columns-2'), () => insertColumns(COLUMN_PRESETS.two), 'layout-cols-2')}</RibbonItem>
        <RibbonItem>{button(t('layout-columns-3'), () => insertColumns(COLUMN_PRESETS.three), 'layout-cols-3')}</RibbonItem>
        <For each={['sidebarLeft', 'sidebarRight'] as ColumnPreset[]}>{(p) =>
          <RibbonItem>{button(t(`layout-columns-${p}`), () => insertColumns(COLUMN_PRESETS[p]), `layout-cols-${p}`)}</RibbonItem>}</For>
        <RibbonItem>
          <RibbonPopover label={t('layout-columns-custom')} icon="preferences" keepSelection disabled={off()} testId="layout-cols-custom">
            <div class="compose-rib__form">
              <label>{t('layout-columns-count')}<input type="number" min="1" max={MAX_COLUMNS} value={count()} onInput={(e) => setCount(Math.min(MAX_COLUMNS, Math.max(1, Number(e.currentTarget.value) || 1)))} /></label>
              <label>{t('layout-columns-gutter')}<input type="number" min="0" max="48" value={gutter()} onInput={(e) => setGutter(Math.min(48, Math.max(0, Number(e.currentTarget.value) || 0)))} /></label>
              <p class="compose-rib__hint">{t('layout-columns-equal-note')}</p>
              <button type="button" class="compose-rt__btn" onClick={() => insertColumns(equalWidths(count()))} data-testid="layout-cols-custom-insert">{t('layout-columns-insert')}</button>
            </div>
          </RibbonPopover>
        </RibbonItem>
        <RibbonItem>{button(t('layout-columns-merge'), merge, 'layout-cols-merge')}</RibbonItem>
      </RibbonGroup>
    </Show>

    <Show when={fWidth()}>
      <RibbonGroup label={t('layout-group-width')} priority={90} icon="eye" testId="layout-group-width">
        <RibbonItem>
          <label class="compose-rib__field"><span class="compose-rib__text">{t('layout-width')}</span>
            <select disabled={off()} title={tip()} value={widthValue()} data-testid="layout-width"
              onChange={(e) => { const v = e.currentTarget.value; if (v === 'theme') clear('width'); else if (v === 'custom') set({ width: customWidth() }); else set({ width: v === 'full' ? 'full' : Number(v) }); }}>
              <option value="theme">{t('layout-width-theme')}</option>
              <For each={WIDTHS}>{(w) => <option value={String(w)}>{w === 'full' ? t('layout-width-full') : `${w} px`}</option>}</For>
              <option value="custom">{t('layout-width-custom')}</option>
            </select></label>
        </RibbonItem>
        <Show when={typeof l().width === 'number' && !(WIDTHS as readonly (string | number)[]).includes(l().width as number)}>
          <RibbonItem><label class="compose-rib__field"><span class="compose-rib__text">{t('layout-width-custom')}</span>
            <input type="number" min="240" max="1200" value={String(l().width)} onChange={(e) => { const n = Math.min(1200, Math.max(240, Number(e.currentTarget.value) || 600)); setCustomWidth(n); set({ width: n }); }} /></label></RibbonItem>
        </Show>
        <RibbonItem>
          <label class="compose-rib__field"><span class="compose-rib__text">{t('layout-align')}</span>
            <select disabled={off()} title={tip()} value={l().align ?? 'theme'} data-testid="layout-align" onChange={(e) => { const v = e.currentTarget.value; if (v === 'theme') clear('align'); else set({ align: v as 'left' | 'center' }); }}>
              <option value="theme">{t('layout-width-theme')}</option><option value="center">{t('layout-align-center')}</option><option value="left">{t('layout-align-left')}</option>
            </select></label>
        </RibbonItem>
        <RibbonItem>
          <label class="compose-rib__field"><span class="compose-rib__text">{t('layout-line-length')}</span>
            <select disabled={off()} title={tip()} value="off" data-testid="layout-line-length" onChange={(e) => { const p = LINE_LENGTHS.find((x) => x.id === e.currentTarget.value); if (p?.width) set({ width: p.width }); e.currentTarget.value = 'off'; }}>
              <For each={LINE_LENGTHS}>{(p) => <option value={p.id}>{t(`layout-line-${p.id}`)}</option>}</For>
            </select></label>
        </RibbonItem>
      </RibbonGroup>
    </Show>

    <Show when={fSpacing()}>
      <RibbonGroup label={t('layout-group-spacing')} priority={80} icon="paragraph" testId="layout-group-spacing">
        <RibbonItem>
          <label class="compose-rib__field"><span class="compose-rib__text">{t('layout-cell-padding')}</span>
            <select disabled={off()} title={tip()} value={String(l().padding ?? 'theme')} data-testid="layout-padding" onChange={(e) => { const v = e.currentTarget.value; if (v === 'theme') clear('padding'); else set({ padding: Number(v) }); }}>
              <option value="theme">{t('layout-width-theme')}</option>
              <For each={PADDINGS}>{(p) => <option value={String(p)}>{p} px</option>}</For>
            </select></label>
        </RibbonItem>
        <RibbonItem>
          <label class="compose-rib__field"><span class="compose-rib__text">{t('layout-paragraph-spacing')}</span>
            <select disabled={off()} title={tip()} value={l().spacing ?? 'theme'} data-testid="layout-spacing" onChange={(e) => { const v = e.currentTarget.value; if (v === 'theme') clear('spacing'); else set({ spacing: v as SpacingPreset }); }}>
              <option value="theme">{t('layout-width-theme')}</option>
              <For each={Object.keys(SPACING) as SpacingPreset[]}>{(s) => <option value={s}>{t(`layout-spacing-${s}`)}</option>}</For>
            </select></label>
        </RibbonItem>
      </RibbonGroup>
    </Show>

    <Show when={fPage()}>
      <RibbonGroup label={t('layout-group-page')} priority={70} icon="paint" testId="layout-group-page">
        <RibbonItem>
          <ColorField variant="ribbon" showValue={false} label={t('layout-page-color')} value={l().pageColor ?? null} fallback="#ffffff" emptyLabel={t('layout-automatic')} disabled={off()}
            onChange={(v) => (v ? set({ pageColor: v }) : clear('pageColor'))} testId="layout-page-color" />
          <span class="compose-rib__text">{t('layout-page-color')}</span>
        </RibbonItem>
        <RibbonItem>
          <ColorField variant="ribbon" showValue={false} label={t('layout-content-color')} value={l().contentColor ?? null} fallback="#ffffff" emptyLabel={t('layout-automatic')} disabled={off()}
            onChange={(v) => (v ? set({ contentColor: v }) : clear('contentColor'))} testId="layout-content-color" />
          <span class="compose-rib__text">{t('layout-content-color')}</span>
        </RibbonItem>
        <RibbonItem><label><input type="checkbox" disabled={off()} checked={l().border === true} onChange={(e) => (e.currentTarget.checked ? set({ border: true }) : clear('border'))} data-testid="layout-border" />{t('layout-border')}</label></RibbonItem>
        <RibbonItem><label><input type="checkbox" disabled={off()} checked={l().rounded === true} onChange={(e) => (e.currentTarget.checked ? set({ rounded: true }) : clear('rounded'))} data-testid="layout-rounded" />{t('layout-rounded')}</label></RibbonItem>
        <RibbonItem><p class="compose-rib__hint">{t('layout-page-sent-note')}</p></RibbonItem>
      </RibbonGroup>
    </Show>

    <Show when={fThemes()}>
      <RibbonGroup label={t('layout-group-themes')} priority={60} icon="image" testId="layout-group-themes">
        <RibbonItem>
          <RibbonPopover label={t('layout-themes-gallery')} icon="image" keepSelection disabled={off()} testId="layout-themes">
            <div class="layout-gallery" role="listbox" aria-label={t('layout-themes-gallery')}>
              <For each={THEMES}>{(th) =>
                <button type="button" role="option" aria-selected={l().theme === th.id} class="layout-gallery__card" data-testid={`layout-theme-${th.id}`}
                  onClick={() => set({ theme: th.id as ThemeId })}>
                  <span class="layout-gallery__page" style={{ background: th.pageColor ?? '#ffffff' }}>
                    <span class="layout-gallery__sheet" style={{ background: th.contentColor ?? '#ffffff', 'font-family': th.fontFamily, color: th.textColor, 'border-top': th.accentBar ? `4px solid ${th.accentBar}` : undefined, border: th.border }}>
                      <strong>Aa</strong><span style={{ color: th.linkColor }}>{t('layout-theme-sample')}</span>
                    </span>
                  </span>
                  <span class="layout-gallery__name">{t(th.labelKey)}</span>
                </button>}</For>
            </div>
          </RibbonPopover>
        </RibbonItem>
        <RibbonItem>
          <button type="button" class="compose-rt__btn" disabled={off() || l().theme === undefined} onClick={() => clear('theme', 'pageColor', 'contentColor', 'width', 'align', 'font', 'linkColor', 'spacing', 'padding', 'border', 'rounded')} data-testid="layout-theme-remove">
            <span class="compose-rib__text">{t('layout-theme-remove')}</span>
          </button>
        </RibbonItem>
      </RibbonGroup>
    </Show>

    <Show when={fFont()}>
      <RibbonGroup label={t('layout-group-font')} priority={50} icon="bold" testId="layout-group-font">
        <RibbonItem>
          <select aria-label={t('layout-font-family')} disabled={off()} title={tip()} value={fontStack()} data-testid="layout-font-family" onChange={(e) => (e.currentTarget.value ? set({ font: { ...font(), family: e.currentTarget.value } }) : undefined)}>
            <Show when={!COMPOSER_FONTS.some((f) => f.stack === fontStack())}><option value={fontStack()}>{fontStack().split(',')[0]?.replace(/["']/g, '')}</option></Show>
            <For each={COMPOSER_FONTS}>{(f) => <option value={f.stack}>{f.label}</option>}</For>
          </select>
          <select aria-label={t('layout-font-size')} disabled={off()} title={tip()} value={String(font().size ?? resolveTheme(l()).fontSize)} data-testid="layout-font-size" onChange={(e) => set({ font: { ...font(), size: Number(e.currentTarget.value) } })}>
            <For each={SIZES}>{(s) => <option value={String(s)}>{s} px</option>}</For>
          </select>
          <ColorField variant="ribbon" showValue={false} label={t('layout-font-color')} value={font().color ?? null} fallback="#000000" emptyLabel={t('layout-automatic')} disabled={off()}
            onChange={(v) => set({ font: { ...font(), ...(v ? { color: v } : {}) } })} testId="layout-font-color" />
        </RibbonItem>
        <Show when={props.saveFont}>
          <RibbonItem>
            <button type="button" class="compose-rt__btn" disabled={off()} data-testid="layout-font-default" title={t('layout-font-default-help')}
              onClick={() => void props.saveFont!(encodeFont({ family: fontStack(), size: font().size ?? resolveTheme(l()).fontSize, color: font().color }))}>
              <span class="compose-rib__text">{t('layout-font-default')}</span>
            </button>
          </RibbonItem>
        </Show>
      </RibbonGroup>
    </Show>

    <RibbonGroup label={t('layout-group-direction')} priority={40} icon="paragraph" testId="layout-group-direction">
      <RibbonItem>{button(t('layout-dir-ltr'), () => set({ dir: 'ltr' }), 'layout-dir-ltr', () => l().dir === 'ltr', 'LTR')}</RibbonItem>
      <RibbonItem>{button(t('layout-dir-rtl'), () => set({ dir: 'rtl' }), 'layout-dir-rtl', () => l().dir === 'rtl', 'RTL')}</RibbonItem>
      <RibbonItem>{button(t('layout-dir-auto'), () => clear('dir'), 'layout-dir-auto', () => l().dir === undefined, 'A')}</RibbonItem>
    </RibbonGroup>
  </>;
}
