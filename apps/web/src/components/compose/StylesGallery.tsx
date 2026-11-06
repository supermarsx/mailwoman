// The Styles group (FORMAT): live-preview tiles for the first styles, a dropdown with
// every style of the chosen set, the four style sets, and the user's own styles
// (create from selection, update to match the selection, reset to Normal).
// Both editors give it an adapter over their own commands.

import { createSignal, For, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n/index.ts';
import { Icon } from '../Icon.tsx';
import { Feature, MenuButton, noFocus } from './FormatControls.tsx';
import { RibbonPopover } from './ribbonParts.tsx';
import { useComposeFeature } from './composeFeatures.ts';
import { createStyleStore } from './stylesStore.ts';
import {
  allStyles,
  builtinStyle,
  isBuiltinId,
  newStyleId,
  QUICK_STYLE_IDS,
  resolveStyle,
  saveCustomStyle,
  STYLE_SETS,
  type StyleDef,
  type StyleSet,
} from './styleModel.ts';

export interface StylesAdapter {
  ready: () => boolean;
  currentId: () => string;
  apply: (style: StyleDef) => void;
  /** The look at the selection, for Create and Update; null when there is nothing to read. */
  capture: () => Omit<StyleDef, 'id'> | null;
}

const OWN_NAMES = new Set(['normal', 'no-spacing', 'title', 'subtitle', 'heading1', 'heading2', 'heading3', 'heading4', 'heading5', 'heading6', 'quote', 'code']);
export const styleName = (style: StyleDef): string => style.name ?? t(OWN_NAMES.has(style.id) ? `mail-editor-style-${style.id}` : `cfmt-style-${style.id}`);

/** A tile or list row drawn in the style's own look (sizes are capped so a title does not fill the ribbon). Colours are left out: a grey subtitle would vanish on a dark ribbon. */
export function previewStyle(style: StyleDef, cap = 20): JSX.CSSProperties {
  const points = style.marks.fontSize ? parseFloat(style.marks.fontSize) : 11;
  return {
    'font-size': `${Math.min(cap, Math.round(points * (4 / 3)))}px`,
    ...(style.marks.bold ? { 'font-weight': 'bold' } : {}),
    ...(style.marks.italic ? { 'font-style': 'italic' } : {}),
    ...(style.marks.underline ? { 'text-decoration': 'underline' } : {}),
    ...(style.marks.highlight ? { 'background-color': style.marks.highlight } : {}),
    ...(style.marks.fontFamily ? { 'font-family': style.marks.fontFamily } : {}),
  };
}

export function createStylesGallery(adapter: StylesAdapter) {
  void loadCatalog('compose-format');
  const store = createStyleStore();
  const customOn = useComposeFeature('format.customStyles');
  const [message, setMessage] = createSignal('');
  const custom = (): StyleDef[] => (customOn() ? store.custom() : []);
  const styles = (): StyleDef[] => allStyles(store.set(), custom());
  const current = (): string => adapter.currentId();
  const currentStyle = (): StyleDef | undefined => styles().find((style) => style.id === current());
  const normal = (): StyleDef => resolveStyle(store.set(), custom(), 'normal') ?? builtinStyle('default', 'normal')!;

  const tile = (style: StyleDef): JSX.Element => (
    <button
      type="button"
      class="cfmt-tile"
      disabled={!adapter.ready()}
      aria-pressed={current() === style.id}
      aria-label={styleName(style)}
      data-tooltip={styleName(style)}
      data-testid={`style-tile-${style.id}`}
      onMouseDown={noFocus}
      onClick={() => adapter.apply(style)}
    >
      <span class="cfmt-tile__sample" style={previewStyle(style, 18)}>Aa</span>
      <span class="cfmt-tile__name">{styleName(style)}</span>
    </button>
  );

  const doCreate = (name: string, done: () => void): void => {
    const taken = custom().map((style) => style.id);
    const look = adapter.capture();
    if (!look) { setMessage(t('cfmt-style-nothing')); return; }
    const style: StyleDef = { id: newStyleId(name, taken), name: name.trim().slice(0, 40), ...look };
    const next = saveCustomStyle(custom(), style, store.max());
    if (!next) { setMessage(t('cfmt-style-limit', { max: store.max() })); return; }
    store.saveCustom(next);
    setMessage(t('cfmt-style-created', { name: style.name ?? '' }));
    done();
  };

  const doUpdate = (): void => {
    const target = currentStyle();
    const look = adapter.capture();
    if (!target || !look) { setMessage(t('cfmt-style-nothing')); return; }
    const style: StyleDef = { id: target.id, ...(target.name ? { name: target.name } : {}), ...look };
    const next = saveCustomStyle(custom(), style, store.max());
    if (!next) { setMessage(t('cfmt-style-limit', { max: store.max() })); return; }
    store.saveCustom(next);
    setMessage(t('cfmt-style-updated', { name: styleName(target) }));
  };

  const doDelete = (): void => {
    const target = currentStyle();
    if (!target || !custom().some((style) => style.id === target.id)) return;
    store.saveCustom(custom().filter((style) => style.id !== target.id));
    setMessage(t('cfmt-style-deleted', { name: styleName(target) }));
  };

  return {
    store,
    /** The first styles as tiles with live previews. */
    tiles: (): JSX.Element => (
      <div class="compose-rib__row cfmt-tiles" role="group" aria-label={t('mail-editor-style')}>
        <For each={QUICK_STYLE_IDS}>{(id) => <Show when={styles().find((style) => style.id === id)}>{(style) => tile(style())}</Show>}</For>
      </div>
    ),
    /** The dropdown with every style, the style sets and the deletion of a custom style. */
    menu: (): JSX.Element => (
      <div class="compose-rib__row">
        <MenuButton label={t('cfmt-all-styles')} icon="chevronDown" iconOnly={false} testId="rt-all-styles" disabled={!adapter.ready()} buttonClass="compose-rt__btn cfmt-allstyles">
          {(close) => (
            <div class="cfmt-styles-list">
              <For each={styles()}>{(style) => (
                <button type="button" role="menuitemradio" aria-checked={current() === style.id} class="cfmt-menu__row cfmt-style-row" data-testid={`style-${style.id}`} onMouseDown={noFocus} onClick={() => { adapter.apply(style); close(); }}>
                  <span class="cfmt-style-row__sample" style={previewStyle(style, 22)}>{styleName(style)}</span>
                  <Show when={style.block === 'inline'}><em class="cfmt-style-row__kind">{t('cfmt-style-inline')}</em></Show>
                </button>
              )}</For>
              <hr />
              <div class="cfmt-menu__label" id="cfmt-sets-label">{t('cfmt-style-set')}</div>
              <For each={STYLE_SETS}>{(set: StyleSet) => (
                <button type="button" role="menuitemradio" aria-checked={store.set() === set} class="cfmt-menu__row" disabled={store.setLocked()} data-testid={`style-set-${set}`} onMouseDown={noFocus} onClick={() => store.chooseSet(set)}>{t(`cfmt-style-set-${set}`)}</button>
              )}</For>
              <Show when={customOn()}>
                <hr />
                <button type="button" role="menuitem" class="cfmt-menu__row" disabled={!custom().some((style) => style.id === current())} data-testid="style-delete" onMouseDown={noFocus} onClick={() => { doDelete(); close(); }}>
                  {isBuiltinId(current()) ? t('cfmt-style-restore') : t('cfmt-style-delete')}
                </button>
              </Show>
            </div>
          )}
        </MenuButton>
        <span class="cfmt-sets-hint" aria-hidden="true">{t(`cfmt-style-set-${store.set()}`)}</span>
      </div>
    ),
    /** Create style from selection, Update style, Reset to Normal. */
    actions: (): JSX.Element => (
      <div class="compose-rib__row">
        <Feature id="format.customStyles">
          <CreateStyle disabled={!adapter.ready() || store.max() === 0} onCreate={doCreate} message={message} />
          <button type="button" class="compose-rt__btn" disabled={!adapter.ready()} data-testid="style-update" data-tooltip={t('cfmt-style-update-help')} aria-label={t('cfmt-style-update')} onMouseDown={noFocus} onClick={doUpdate}><Icon name="sync" /></button>
        </Feature>
        <button type="button" class="compose-rt__btn" disabled={!adapter.ready()} data-testid="style-reset" data-tooltip={t('cfmt-style-reset')} aria-label={t('cfmt-style-reset')} onMouseDown={noFocus} onClick={() => adapter.apply(normal())}><Icon name="eraser" /></button>
      </div>
    ),
  };
}

function CreateStyle(props: { disabled: boolean; onCreate: (name: string, done: () => void) => void; message: () => string }): JSX.Element {
  const [open, setOpen] = createSignal(false);
  const [name, setName] = createSignal('');
  const submit = (): void => {
    if (name().trim() === '') return;
    props.onCreate(name(), () => { setName(''); setOpen(false); });
  };
  return (
    <RibbonPopover label={t('cfmt-style-create')} icon="plus" iconOnly keepSelection disabled={props.disabled} buttonTestId="style-create" buttonClass="compose-rt__btn" open={open()} onOpenChange={setOpen}>
      <div class="cfmt-sort" role="group" aria-label={t('cfmt-style-create')}>
        <label>{t('cfmt-style-name')}
          <input type="text" maxLength={40} value={name()} data-testid="style-name" onInput={(event) => setName(event.currentTarget.value)} onKeyDown={(event) => { if (event.key === 'Enter') { event.preventDefault(); submit(); } event.stopPropagation(); }} />
        </label>
        <div class="cfmt-sort__actions">
          <button type="button" class="compose-rt__btn" disabled={name().trim() === ''} data-testid="style-create-apply" onMouseDown={noFocus} onClick={submit}>{t('cfmt-style-create-apply')}</button>
        </div>
        <span class="cfmt-sort__message" aria-live="polite" data-testid="styles-message">{props.message()}</span>
      </div>
    </RibbonPopover>
  );
}

export type StylesGallery = ReturnType<typeof createStylesGallery>;
