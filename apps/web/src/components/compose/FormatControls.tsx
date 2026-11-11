// The Format tab's shared controls (FORMAT). Both visual editors build a
// `FormatAdapter` over their own commands and get the same buttons, split
// dropdowns, clear-formatting menu, line spacing menu and format painter from
// `createFormatControls`. Every control is wrapped in its compose feature flag, so
// a switched-off feature renders nothing (and the editors' shortcuts check the
// same flag).

import { createEffect, createSignal, For, onCleanup, Show, type Accessor, type JSX } from 'solid-js';
import { Icon, type IconName } from '../Icon.tsx';
import { loadCatalog, t } from '../../i18n/index.ts';
import { RibbonPopover } from './ribbonParts.tsx';
import { useComposeFeature } from './composeFeatures.ts';
import {
  BULLET_STYLES,
  CLEAR_VARIANTS,
  DEFAULT_LIST_STYLE,
  LINE_SPACING_CHOICES,
  ORDERED_STYLES,
  PASTE_MODES,
  paintSummaryText,
  validLineHeight,
  type ClearVariant,
  type ListKind,
  type ListStyleDef,
  type PaintSummary,
  type PasteMode,
} from './formatModel.ts';
import { EFFECTS, type EffectDef, type EffectKind } from './effects.ts';
import { FontDialog, type FontAdapter } from './FontDialog.tsx';
import { DEFAULT_SORT, type SortOptions } from './sortModel.ts';
import './compose-format.css';

export type PainterStatus = 'idle' | 'copied' | 'sticky';

/** The format painter's state, shared by both editors; the editor supplies how to read and apply a look. */
export interface Painter {
  summary: Accessor<PaintSummary | null>;
  sticky: Accessor<boolean>;
  copy: (sticky?: boolean) => void;
  apply: () => void;
  cancel: () => void;
}

export function createPainter<S>(source: {
  capture: () => { value: S; summary: PaintSummary } | null;
  apply: (value: S) => boolean;
}): Painter {
  const [held, setHeld] = createSignal<{ value: S; summary: PaintSummary } | null>(null);
  const [sticky, setSticky] = createSignal(false);
  return {
    summary: () => held()?.summary ?? null,
    sticky,
    copy(stickyMode = false) {
      const taken = source.capture();
      if (!taken) return;
      setHeld(taken);
      setSticky(stickyMode);
    },
    apply() {
      const taken = held();
      if (taken) source.apply(taken.value);
    },
    cancel() {
      setSticky(false);
    },
  };
}

/** What the controls need from an editor. All reads are reactive (they read the editor's state signal). */
export interface FormatAdapter {
  ready?: () => boolean;
  fontStep: (direction: 1 | -1, fine: boolean) => void;
  listType: () => { kind: ListKind; style: string } | null;
  applyList: (kind: ListKind, style: string) => void;
  listStart: (value: number | 'restart' | 'continue') => void;
  canContinue: () => boolean;
  indent: (direction: 1 | -1) => void;
  canOutdent: () => boolean;
  clear: (variant: ClearVariant) => void;
  lineSpacing: (value: string | null) => void;
  currentLineSpacing: () => string | null;
  spacing: (side: 'before' | 'after', points: number | null) => void;
  align: (value: 'left' | 'center' | 'right' | 'justify') => void;
  currentAlign: () => string | null;
  direction: (value: 'ltr' | 'rtl') => void;
  currentDirection: () => string | null;
  paste: (mode: PasteMode) => void;
  painter: Painter;
  font?: FontAdapter;
  effectOn: (kind: EffectKind, value?: string) => boolean;
  toggleEffect: (kind: EffectKind, value: string) => void;
  /** Sort the selection; null when there was nothing to sort. */
  sort: (options: SortOptions) => { kind: 'rows' | 'items' | 'blocks'; count: number; removed: number } | null;
}

const STORE = 'mw.compose.list.';
function remembered(kind: ListKind): string {
  try {
    const value = localStorage.getItem(`${STORE}${kind}`);
    const known = (kind === 'bullet' ? BULLET_STYLES : ORDERED_STYLES).some((entry) => entry.id === value);
    if (value && known) return value;
  } catch { /* storage may be blocked */ }
  return DEFAULT_LIST_STYLE[kind];
}
const [lastBullet, setLastBullet] = createSignal(remembered('bullet'));
const [lastOrdered, setLastOrdered] = createSignal(remembered('ordered'));
function remember(kind: ListKind, style: string): void {
  (kind === 'bullet' ? setLastBullet : setLastOrdered)(style);
  try { localStorage.setItem(`${STORE}${kind}`, style); } catch { /* ignore */ }
}

/** Render `children` only while the feature is on. */
export function Feature(props: { id: string; children: JSX.Element }): JSX.Element {
  const on = useComposeFeature(props.id);
  return <Show when={on()}>{props.children}</Show>;
}

/** Move focus through a menu's items with the arrow keys. */
function menuKeys(event: KeyboardEvent): void {
  const menu = event.currentTarget as HTMLElement;
  const items = Array.from(menu.querySelectorAll<HTMLElement>('[role^="menuitem"]:not([disabled])'));
  const at = items.indexOf(document.activeElement as HTMLElement);
  const move = (index: number): void => { event.preventDefault(); items[(index + items.length) % items.length]?.focus(); };
  if (event.key === 'ArrowDown') move(at + 1);
  else if (event.key === 'ArrowUp') move(at < 0 ? items.length - 1 : at - 1);
  else if (event.key === 'Home') move(0);
  else if (event.key === 'End') move(items.length - 1);
}

/** A ribbon dropdown whose popover holds a menu: focus lands on the checked (or first) item on open. */
export function MenuButton(props: {
  label: string;
  icon?: IconName;
  iconOnly?: boolean;
  testId: string;
  disabled?: boolean;
  buttonClass?: string;
  children: (close: () => void) => JSX.Element;
}): JSX.Element {
  const [open, setOpen] = createSignal(false);
  let root: HTMLDivElement | undefined;
  createEffect(() => {
    if (!open()) return;
    queueMicrotask(() => queueMicrotask(() => {
      const target = root?.querySelector<HTMLElement>('[role^="menuitem"][aria-checked="true"]') ?? root?.querySelector<HTMLElement>('[role^="menuitem"]');
      target?.focus();
    }));
  });
  return (
    <div ref={root} class="cfmt-menu-root">
      <RibbonPopover
        label={props.label}
        {...(props.icon ? { icon: props.icon } : {})}
        iconOnly={props.iconOnly ?? true}
        keepSelection
        buttonClass={props.buttonClass ?? 'compose-rt__btn cfmt-arrow'}
        buttonTestId={props.testId}
        {...(props.disabled === undefined ? {} : { disabled: props.disabled })}
        open={open()}
        onOpenChange={setOpen}
      >
        <div role="menu" aria-label={props.label} class="cfmt-menu" onKeyDown={menuKeys}>
          {props.children(() => setOpen(false))}
        </div>
      </RibbonPopover>
    </div>
  );
}

export const noFocus = (event: MouseEvent): void => event.preventDefault();

function ListPreview(props: { def: ListStyleDef }): JSX.Element {
  return (
    <span class="cfmt-preview" aria-hidden="true">
      <For each={props.def.sample}>{(marker) => <span class="cfmt-preview__row"><b>{marker}</b><i /></span>}</For>
    </span>
  );
}

export function createFormatControls(adapter: FormatAdapter) {
  void loadCatalog('compose-format');
  const ready = (): boolean => adapter.ready?.() ?? true;
  const [fontOpen, setFontOpen] = createSignal(false);
  const fontOn = useComposeFeature('format.fontDialog');
  const button = (key: string, icon: IconName, run: () => void, options: { testId: string; pressed?: () => boolean; disabled?: () => boolean; text?: boolean; extra?: JSX.Element; large?: boolean; tooltip?: string; onDblClick?: () => void; label?: string }): JSX.Element => (
    <button
      type="button"
      class={`compose-rt__btn${options.large ? ' compose-rib__large' : ''}`}
      disabled={!ready() || options.disabled?.()}
      aria-label={options.label ?? t(key)}
      aria-pressed={options.pressed ? options.pressed() : undefined}
      data-tooltip={options.tooltip ?? t(key)}
      data-testid={options.testId}
      onMouseDown={noFocus}
      onClick={run}
      onDblClick={options.onDblClick}
    >
      <Icon name={icon} />
      <Show when={options.text}><span class="compose-rib__text">{options.label ?? t(key)}</span></Show>
      {options.extra}
    </button>
  );

  const splitList = (kind: ListKind): JSX.Element => {
    const defs = kind === 'bullet' ? BULLET_STYLES : ORDERED_STYLES;
    const last = kind === 'bullet' ? lastBullet : lastOrdered;
    const active = (): boolean => adapter.listType()?.kind === kind;
    const current = (): string => (active() ? adapter.listType()!.style : last());
    const label = t(kind === 'bullet' ? 'cfmt-bullets' : 'cfmt-numbering');
    const choose = (style: string, close: () => void): void => {
      remember(kind, style);
      adapter.applyList(kind, style);
      close();
    };
    const inOrdered = (): boolean => adapter.listType()?.kind === 'ordered';
    return (
      <Feature id="format.lists">
        <div class="cfmt-split" role="group" aria-label={label}>
          {button(kind === 'bullet' ? 'cfmt-bullets' : 'cfmt-numbering', kind === 'bullet' ? 'bulletList' : 'numberList', () => { remember(kind, active() ? current() : last()); adapter.applyList(kind, active() ? current() : last()); }, { testId: kind === 'bullet' ? 'rt-bullet' : 'rt-ordered', pressed: active })}
          <Feature id="format.listTypes">
            <MenuButton label={t(kind === 'bullet' ? 'cfmt-bullet-types' : 'cfmt-numbering-types')} icon="chevronDown" testId={kind === 'bullet' ? 'rt-bullet-types' : 'rt-ordered-types'} disabled={!ready()}>
              {(close) => (
                <>
                  <div class="cfmt-menu__grid">
                    <For each={defs}>{(def) => (
                      <button type="button" role="menuitemradio" aria-checked={current() === def.id && active()} class="cfmt-menu__item" data-style={def.id} data-testid={`list-style-${def.id}`} onMouseDown={noFocus} onClick={() => choose(def.id, close)}>
                        <ListPreview def={def} />
                        <span class="cfmt-menu__label">{t(`cfmt-list-${def.id}`)}</span>
                      </button>
                    )}</For>
                  </div>
                  <Show when={kind === 'ordered'}>
                    <hr />
                    <button type="button" role="menuitem" class="cfmt-menu__row" disabled={!inOrdered()} data-testid="list-restart" onMouseDown={noFocus} onClick={() => { adapter.listStart('restart'); close(); }}>{t('cfmt-restart')}</button>
                    <button type="button" role="menuitem" class="cfmt-menu__row" disabled={!inOrdered() || !adapter.canContinue()} data-testid="list-continue" onMouseDown={noFocus} onClick={() => { adapter.listStart('continue'); close(); }}>{t('cfmt-continue')}</button>
                    <StartValue disabled={!inOrdered()} onSet={(value) => { adapter.listStart(value); close(); }} />
                  </Show>
                </>
              )}
            </MenuButton>
          </Feature>
        </div>
      </Feature>
    );
  };

  const clearMenu = (): JSX.Element => (
    <Feature id="format.clearFormatting">
      <div class="cfmt-split" role="group" aria-label={t('cfmt-clear')}>
        {button('cfmt-clear', 'eraser', () => adapter.clear('all'), { testId: 'rt-clear-formatting', tooltip: t('cfmt-clear-help') })}
        <MenuButton label={t('cfmt-clear-options')} icon="chevronDown" testId="rt-clear-formatting-options" disabled={!ready()}>
          {(close) => <For each={CLEAR_VARIANTS}>{(variant) => (
            <button type="button" role="menuitem" class="cfmt-menu__row" data-testid={`clear-${variant}`} onMouseDown={noFocus} onClick={() => { adapter.clear(variant); close(); }}>{t(`cfmt-clear-${variant}`)}</button>
          )}</For>}
        </MenuButton>
      </div>
    </Feature>
  );

  return {
    fontSteps: (): JSX.Element => (
      <Feature id="format.fontSizeStep">
        <div class="compose-rib__row">
          <button type="button" class="compose-rt__btn" disabled={!ready()} aria-label={t('cfmt-font-grow')} data-tooltip={t('cfmt-font-grow-help')} data-testid="rt-font-grow" onMouseDown={noFocus} onClick={(event) => adapter.fontStep(1, event.shiftKey)}><Icon name="fontGrow" /></button>
          <button type="button" class="compose-rt__btn" disabled={!ready()} aria-label={t('cfmt-font-shrink')} data-tooltip={t('cfmt-font-shrink-help')} data-testid="rt-font-shrink" onMouseDown={noFocus} onClick={(event) => adapter.fontStep(-1, event.shiftKey)}><Icon name="fontShrink" /></button>
        </div>
      </Feature>
    ),
    bulletList: (): JSX.Element => splitList('bullet'),
    numberList: (): JSX.Element => splitList('ordered'),
    indentButtons: (): JSX.Element => (
      <Feature id="format.indent">
        <div class="compose-rib__row">
          {button('cfmt-outdent', 'indentLess', () => adapter.indent(-1), { testId: 'rt-outdent', disabled: () => !adapter.canOutdent() })}
          {button('cfmt-indent', 'indentMore', () => adapter.indent(1), { testId: 'rt-indent' })}
        </div>
      </Feature>
    ),
    alignButtons: (): JSX.Element => (
      <div class="compose-rib__row" role="group" aria-label={t('mail-editor-alignment')}>
        <For each={[['left', 'alignLeft'], ['center', 'alignCenter'], ['right', 'alignRight'], ['justify', 'alignJustify']] as const}>{([value, icon]) => (
          button(`mail-editor-align-${value}`, icon, () => adapter.align(value), { testId: `rt-align-${value}`, pressed: () => (adapter.currentAlign() ?? 'left') === value })
        )}</For>
      </div>
    ),
    lineSpacing: (): JSX.Element => (
      <Feature id="format.lineSpacing">
        <LineSpacing adapter={adapter} ready={ready} />
      </Feature>
    ),
    direction: (): JSX.Element => (
      <Feature id="format.direction">
        <div class="compose-rib__row" role="group" aria-label={t('cfmt-direction')}>
          {button('cfmt-ltr', 'textLtr', () => adapter.direction('ltr'), { testId: 'rt-dir-ltr', pressed: () => (adapter.currentDirection() ?? 'ltr') === 'ltr' })}
          {button('cfmt-rtl', 'textRtl', () => adapter.direction('rtl'), { testId: 'rt-dir-rtl', pressed: () => adapter.currentDirection() === 'rtl' })}
        </div>
      </Feature>
    ),
    clearFormatting: clearMenu,
    /** The Font dialog launcher (Ctrl+D) and its dialog. */
    fontDialog: (): JSX.Element => (
      <Feature id="format.fontDialog">
        <button type="button" class="compose-rt__btn" disabled={!ready() || !adapter.font} aria-label={t('cfmt-font-dialog')} data-tooltip={t('cfmt-font-dialog-help')} data-testid="rt-font-dialog" aria-haspopup="dialog" onMouseDown={noFocus} onClick={() => setFontOpen(true)}><Icon name="preferences" /></button>
        <Show when={fontOpen() && adapter.font}>{(font) => <FontDialog adapter={font()} onClose={() => setFontOpen(false)} />}</Show>
      </Feature>
    ),
    openFontDialog: (): boolean => { if (!fontOn() || !adapter.font) return false; setFontOpen(true); return true; },
    effects: (): JSX.Element => <EffectsMenu adapter={adapter} ready={ready} />,
    sortMenu: (): JSX.Element => <Feature id="format.sort"><SortMenu adapter={adapter} ready={ready} /></Feature>,
    pasteLarge: (): JSX.Element => (
      <div class="cfmt-paste cfmt-split" role="group" aria-label={t('mail-editor-paste')}>
        {button('mail-editor-paste', 'paste', () => adapter.paste('keep'), { testId: 'rt-paste', large: true, text: true })}
        <Feature id="format.pasteOptions">
          <MenuButton label={t('cfmt-paste-options')} icon="chevronDown" testId="rt-paste-options" disabled={!ready()} buttonClass="compose-rt__btn cfmt-arrow">
            {(close) => <For each={PASTE_MODES}>{(mode) => (
              <button type="button" role="menuitem" class="cfmt-menu__row" data-testid={`paste-${mode}`} onMouseDown={noFocus} onClick={() => { adapter.paste(mode); close(); }}>{t(`cfmt-paste-${mode}`)}</button>
            )}</For>}
          </MenuButton>
        </Feature>
      </div>
    ),
    copyFormat: (): JSX.Element => button('cfmt-copy-format', 'brushCopy', () => adapter.painter.copy(false), { testId: 'rt-copy-format', text: true, pressed: adapter.painter.sticky, tooltip: t('cfmt-copy-format-help'), onDblClick: () => adapter.painter.copy(true) }),
    applyFormat: (): JSX.Element => button('cfmt-apply-format', 'brushApply', () => adapter.painter.apply(), { testId: 'rt-apply-format', text: true, disabled: () => adapter.painter.summary() === null, tooltip: t('cfmt-apply-format-help') }),
    formatChip: (): JSX.Element => (
      <span class="cfmt-chip" data-testid="rt-format-chip" aria-live="polite" data-sticky={adapter.painter.sticky() ? 'true' : 'false'}>
        <Show when={adapter.painter.summary()} fallback={<span class="cfmt-chip__empty">{t('cfmt-nothing-copied')}</span>}>
          {(summary) => <>{paintSummaryText(summary()) || t('cfmt-plain')}<Show when={adapter.painter.sticky()}><em>{t('cfmt-esc-stop')}</em></Show></>}
        </Show>
      </span>
    ),
  };
}

const SUPPORT_KEY = { most: 'cfmt-support-most', some: 'cfmt-support-some' } as const;

/** The Effects dropdown: every font effect, with how widely mail apps draw it. */
function EffectsMenu(props: { adapter: FormatAdapter; ready: () => boolean }): JSX.Element {
  const adapter = props.adapter;
  const entries = EFFECTS.flatMap((def) => (def.values ? def.values.map((value) => ({ def, value })) : [{ def, value: '' }]));
  const label = (def: EffectDef, value: string): string => {
    if (def.kind === 'underlineStyle') return t('cfmt-effect-underlineStyle', { style: t(`cfmt-underline-${value}`) });
    if (def.kind === 'scale') return t('cfmt-effect-scale', { percent: value });
    if (def.kind === 'spacing') return t(`cfmt-spacing-${value}`);
    if (def.kind === 'ligatures') return t('cfmt-effect-ligatures');
    if (def.kind === 'outline') return t('cfmt-effect-outline');
    return t(`cfmt-effect-${def.kind}`);
  };
  return (
    <MenuButton label={t('cfmt-effects')} icon="textEffects" iconOnly testId="rt-effects" disabled={!props.ready()} buttonClass="compose-rt__btn">
      {(close) => (
        <div class="cfmt-effects">
          <For each={entries}>{({ def, value }) => (
            <Feature id={def.feature}>
              <button
                type="button"
                role="menuitemcheckbox"
                aria-checked={adapter.effectOn(def.kind, value)}
                class="cfmt-menu__row cfmt-effect"
                data-testid={`effect-${def.kind}${value ? `-${value.replace('#', '')}` : ''}`}
                data-support={def.support}
                title={t(SUPPORT_KEY[def.support])}
                onMouseDown={noFocus}
                onClick={() => { adapter.toggleEffect(def.kind, value || def.defaultValue || ''); close(); }}
              >
                <span class="cfmt-effect__check" aria-hidden="true">{adapter.effectOn(def.kind, value) ? '✓' : ''}</span>
                <span class="cfmt-effect__label">{label(def, value)}</span>
                <span class="cfmt-effect__hint">{t(SUPPORT_KEY[def.support])}</span>
              </button>
            </Feature>
          )}</For>
        </div>
      )}
    </MenuButton>
  );
}

/** The Sort control: order, type and options for the selected lines, list items or table rows. */
function SortMenu(props: { adapter: FormatAdapter; ready: () => boolean }): JSX.Element {
  const [options, setOptions] = createSignal<SortOptions>({ ...DEFAULT_SORT });
  const [message, setMessage] = createSignal('');
  const patch = (change: Partial<SortOptions>): void => { setOptions({ ...options(), ...change }); };
  const run = (mode: SortOptions['mode'], close: () => void): void => {
    const outcome = props.adapter.sort({ ...options(), mode });
    setMessage(outcome === null ? t('cfmt-sort-nothing') : outcome.removed > 0 ? t('cfmt-sorted-removed', { count: outcome.count, removed: outcome.removed }) : t('cfmt-sorted', { count: outcome.count }));
    if (outcome !== null) close();
  };
  return (
    <MenuButton label={t('cfmt-sort')} icon="sort" iconOnly testId="rt-sort" disabled={!props.ready()} buttonClass="compose-rt__btn">
      {(close) => (
        <div class="cfmt-sort" role="group" aria-label={t('cfmt-sort')}>
          <label>{t('cfmt-sort-order')}
            <select data-testid="sort-order" value={options().order} onChange={(event) => patch({ order: event.currentTarget.value as SortOptions['order'] })}>
              <option value="asc">{t('cfmt-sort-asc')}</option><option value="desc">{t('cfmt-sort-desc')}</option>
            </select>
          </label>
          <label>{t('cfmt-sort-by')}
            <select data-testid="sort-type" value={options().type} onChange={(event) => patch({ type: event.currentTarget.value as SortOptions['type'] })}>
              <option value="text">{t('cfmt-sort-text')}</option><option value="number">{t('cfmt-sort-number')}</option><option value="date">{t('cfmt-sort-date')}</option>
            </select>
          </label>
          <label class="cfmt-check"><input type="checkbox" data-testid="sort-case" checked={options().caseSensitive} onChange={(event) => patch({ caseSensitive: event.currentTarget.checked })} />{t('cfmt-sort-case')}</label>
          <label class="cfmt-check"><input type="checkbox" data-testid="sort-natural" checked={options().natural} onChange={(event) => patch({ natural: event.currentTarget.checked })} />{t('cfmt-sort-natural')}</label>
          <label class="cfmt-check"><input type="checkbox" data-testid="sort-articles" checked={options().ignoreArticles} onChange={(event) => patch({ ignoreArticles: event.currentTarget.checked })} />{t('cfmt-sort-articles')}</label>
          <label class="cfmt-check"><input type="checkbox" data-testid="sort-duplicates" checked={options().removeDuplicates} onChange={(event) => patch({ removeDuplicates: event.currentTarget.checked })} />{t('cfmt-sort-duplicates')}</label>
          <label class="cfmt-check"><input type="checkbox" data-testid="sort-header" checked={options().header} onChange={(event) => patch({ header: event.currentTarget.checked })} />{t('cfmt-sort-header')}</label>
          <label>{t('cfmt-sort-column')}
            <input type="number" min="1" max="10" data-testid="sort-column" placeholder={t('cfmt-sort-column-caret')} value={options().column === null ? '' : options().column! + 1} onInput={(event) => { const v = event.currentTarget.value; patch({ column: /^\d+$/.test(v) && Number(v) >= 1 ? Number(v) - 1 : null }); }} />
          </label>
          <div class="cfmt-sort__actions">
            <button type="button" class="compose-rt__btn" data-testid="sort-run" onMouseDown={noFocus} onClick={() => run('sort', close)}>{t('cfmt-sort-run')}</button>
            <button type="button" class="compose-rt__btn" data-testid="sort-reverse" onMouseDown={noFocus} onClick={() => run('reverse', close)}>{t('cfmt-sort-reverse')}</button>
            <button type="button" class="compose-rt__btn" data-testid="sort-shuffle" onMouseDown={noFocus} onClick={() => run('shuffle', close)}>{t('cfmt-sort-shuffle')}</button>
            <button type="button" class="compose-rt__btn" data-testid="sort-unique" onMouseDown={noFocus} onClick={() => run('unique', close)}>{t('cfmt-sort-unique')}</button>
          </div>
          <span class="cfmt-sort__message" aria-live="polite" data-testid="sort-message">{message()}</span>
        </div>
      )}
    </MenuButton>
  );
}

function StartValue(props: { disabled: boolean; onSet: (value: number) => void }): JSX.Element {
  const [value, setValue] = createSignal('1');
  const valid = (): boolean => /^\d{1,4}$/.test(value()) && Number(value()) >= 1;
  return (
    <div class="cfmt-start" role="group" aria-label={t('cfmt-set-start')}>
      <label>{t('cfmt-set-start')}
        <input type="number" min="1" max="9999" value={value()} disabled={props.disabled} data-testid="list-start-input"
          onInput={(event) => setValue(event.currentTarget.value)}
          onKeyDown={(event) => { if (event.key === 'Enter' && valid()) { event.preventDefault(); props.onSet(Number(value())); } event.stopPropagation(); }} />
      </label>
      <button type="button" class="compose-rt__btn" disabled={props.disabled || !valid()} data-testid="list-start-apply" onMouseDown={noFocus} onClick={() => props.onSet(Number(value()))}>{t('cfmt-set')}</button>
    </div>
  );
}

function LineSpacing(props: { adapter: FormatAdapter; ready: () => boolean }): JSX.Element {
  const [custom, setCustom] = createSignal('');
  const adapter = props.adapter;
  const customValid = (): boolean => validLineHeight(custom());
  return (
    <MenuButton label={t('cfmt-line-spacing')} icon="lineSpacing" iconOnly testId="rt-line-spacing" disabled={!props.ready()} buttonClass="compose-rt__btn">
      {(close) => (
        <>
          <For each={LINE_SPACING_CHOICES}>{(value) => (
            <button type="button" role="menuitemradio" aria-checked={(adapter.currentLineSpacing() ?? '1') === value} class="cfmt-menu__row" data-testid={`line-${value}`} onMouseDown={noFocus} onClick={() => { adapter.lineSpacing(value === '1' ? null : value); close(); }}>{Number(value).toFixed(value === '1.15' ? 2 : 1)}</button>
          )}</For>
          <hr />
          <div class="cfmt-start" role="group" aria-label={t('cfmt-custom-spacing')}>
            <label>{t('cfmt-custom-spacing')}
              <input type="number" min="0.5" max="5" step="0.05" value={custom()} data-testid="line-custom-input" onInput={(event) => setCustom(event.currentTarget.value)}
                onKeyDown={(event) => { if (event.key === 'Enter' && customValid()) { event.preventDefault(); adapter.lineSpacing(custom()); close(); } event.stopPropagation(); }} />
            </label>
            <button type="button" class="compose-rt__btn" disabled={!customValid()} data-testid="line-custom-apply" onMouseDown={noFocus} onClick={() => { adapter.lineSpacing(custom()); close(); }}>{t('cfmt-set')}</button>
          </div>
          <hr />
          <For each={['before', 'after'] as const}>{(side) => (
            <div class="cfmt-start" role="group">
              <label>{t(`mail-editor-space-${side}`)}
                <select aria-label={t(`mail-editor-space-${side}`)} data-testid={`space-${side}`} onChange={(event) => { const v = event.currentTarget.value; adapter.spacing(side, v === '' ? null : Number(v)); }}>
                  <option value="">{t('mail-compose-font-inherit')}</option>
                  <For each={[0, 6, 12, 18, 24, 36]}>{(points) => <option value={points}>{points} pt</option>}</For>
                </select>
              </label>
            </div>
          )}</For>
        </>
      )}
    </MenuButton>
  );
}

export type FormatControls = ReturnType<typeof createFormatControls>;

/** Escape ends sticky painting. Mount once per editor surface. */
export function stickyPainterEscape(painter: Painter, surface: HTMLElement | Document): () => void {
  const onKey = (event: Event): void => {
    if ((event as KeyboardEvent).key === 'Escape' && painter.sticky()) painter.cancel();
  };
  surface.addEventListener('keydown', onKey, true);
  onCleanup(() => surface.removeEventListener('keydown', onKey, true));
  return () => surface.removeEventListener('keydown', onKey, true);
}
