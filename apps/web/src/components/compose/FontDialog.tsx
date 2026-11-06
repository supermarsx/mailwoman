// The Font dialog (FORMAT, Ctrl+D): family, size, colour, bold and italic, effects,
// underline style and character spacing, with a live preview. It changes only what
// the user touched, and 'Set as default' keeps family, size and colour as the font a
// new message starts with (`compose.layout.defaultFont`).

import { createSignal, For, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n/index.ts';
import { InsertDialog } from './insert/InsertDialog.tsx';
import { COMPOSER_FONTS } from './fontCatalog.ts';
import { FONT_RAMP, pointText } from './formatModel.ts';
import { effectStyle, type EffectKind } from './effects.ts';
import { writeSetting } from '../../api/settings.ts';
import { encodeFont } from './layout/LayoutGroups.tsx';

/** What the dialog shows and writes. A field the user did not touch stays out of `changes`. */
export interface FontSpec {
  family: string;
  size: number;
  color: string;
  bold: boolean;
  italic: boolean;
  /** On/off effects (strikethrough and the rest). */
  effects: Partial<Record<EffectKind, boolean>>;
  /** '' = none, else single, double, dotted, dashed, wavy or thick. */
  underline: string;
  /** normal, condensed or expanded */
  spacing: string;
}

export interface FontAdapter {
  read: () => FontSpec;
  /** Apply the fields in `changes` to the selection. */
  apply: (changes: Partial<FontSpec>) => void;
}

export const DIALOG_EFFECTS: readonly { kind: EffectKind; key: string }[] = [
  { kind: 'doubleStrike', key: 'cfmt-effect-doubleStrike' },
  { kind: 'superscript', key: 'cfmt-effect-superscript' },
  { kind: 'subscript', key: 'cfmt-effect-subscript' },
  { kind: 'smallCaps', key: 'cfmt-effect-smallCaps' },
  { kind: 'allCaps', key: 'cfmt-effect-allCaps' },
  { kind: 'shadow', key: 'cfmt-effect-shadow' },
];

/** The CSS the preview shows for a spec. */
export function previewCss(spec: FontSpec): JSX.CSSProperties {
  const css: Record<string, string> = { 'font-size': `${Math.round(spec.size * (4 / 3))}px` };
  if (spec.family) css['font-family'] = spec.family;
  if (spec.color) css['color'] = spec.color;
  if (spec.bold) css['font-weight'] = 'bold';
  if (spec.italic) css['font-style'] = 'italic';
  const decoration: string[] = [];
  if (spec.underline) decoration.push('underline');
  for (const [kind, on] of Object.entries(spec.effects)) {
    if (!on) continue;
    if (kind === 'superscript') { css['vertical-align'] = 'super'; css['font-size'] = `${Math.round(spec.size * (4 / 3) * 0.75)}px`; continue; }
    if (kind === 'subscript') { css['vertical-align'] = 'sub'; css['font-size'] = `${Math.round(spec.size * (4 / 3) * 0.75)}px`; continue; }
    for (const [property, value] of effectStyle(kind as EffectKind, '')) {
      if (property === 'text-decoration') decoration.push(value);
      else css[property] = value;
    }
  }
  if (spec.underline && spec.underline !== 'single') for (const [property, value] of effectStyle('underlineStyle', spec.underline)) if (property !== 'text-decoration') css[property] = value;
  if (decoration.length) css['text-decoration'] = decoration.join(' ');
  if (spec.spacing === 'condensed' || spec.spacing === 'expanded') for (const [property, value] of effectStyle('spacing', spec.spacing)) css[property] = value;
  return css as JSX.CSSProperties;
}

export function FontDialog(props: { adapter: FontAdapter; onClose: () => void; returnTo?: HTMLElement | null; saveDefault?: (value: string) => Promise<unknown> }): JSX.Element {
  void loadCatalog('compose-insert');
  const initial = props.adapter.read();
  const [spec, setSpec] = createSignal<FontSpec>({ ...initial, effects: { ...initial.effects } });
  const [touched, setTouched] = createSignal<Partial<FontSpec>>({});
  const [message, setMessage] = createSignal('');
  const change = <K extends keyof FontSpec>(key: K, value: FontSpec[K]): void => {
    setSpec({ ...spec(), [key]: value });
    setTouched({ ...touched(), [key]: value });
  };
  const toggleEffect = (kind: EffectKind, on: boolean): void => {
    const effects = { ...spec().effects, [kind]: on };
    if (on && kind === 'superscript') effects.subscript = false;
    if (on && kind === 'subscript') effects.superscript = false;
    setSpec({ ...spec(), effects });
    setTouched({ ...touched(), effects: { ...(touched().effects ?? {}), ...effects } });
  };
  const apply = (): void => {
    props.adapter.apply(touched());
    props.onClose();
  };
  const setDefault = async (): Promise<void> => {
    const value = spec();
    const stack = value.family || undefined;
    const px = Math.round(value.size * (4 / 3));
    const encoded = encodeFont({ ...(stack ? { family: stack } : {}), size: px, ...(/^#[0-9a-f]{6}$/i.test(value.color) ? { color: value.color } : {}) });
    try {
      await (props.saveDefault ?? ((v: string) => writeSetting('compose.layout.defaultFont', v)))(encoded);
      setMessage(t('cfmt-font-default-set'));
    } catch {
      setMessage(t('cfmt-font-default-failed'));
    }
  };
  return (
    <InsertDialog title={t('cfmt-font-dialog')} testId="font-dialog" onClose={props.onClose} onSubmit={apply} submitLabel={t('cfmt-font-apply')} {...(props.returnTo ? { returnTo: props.returnTo } : {})}>
      <div class="cfmt-fontdlg">
        <label>{t('mail-compose-font')}
          <select data-testid="font-family" value={spec().family} onChange={(event) => change('family', event.currentTarget.value)}>
            <option value="">{t('mail-compose-font-inherit')}</option>
            <Show when={spec().family && !COMPOSER_FONTS.some((font) => font.stack === spec().family)}><option value={spec().family}>{spec().family}</option></Show>
            <For each={COMPOSER_FONTS}>{(font) => <option value={font.stack}>{font.label}</option>}</For>
          </select>
        </label>
        <label>{t('mail-compose-font-size')}
          <input type="number" min="1" max="96" step="0.5" data-testid="font-size" value={spec().size} list="cfmt-sizes" onInput={(event) => { const n = Number(event.currentTarget.value); if (n >= 1 && n <= 96) change('size', n); }} />
          <datalist id="cfmt-sizes"><For each={FONT_RAMP}>{(size) => <option value={pointText(size).replace('pt', '')} />}</For></datalist>
        </label>
        <label>{t('cfmt-font-color')}
          <input type="color" data-testid="font-color" value={/^#[0-9a-f]{6}$/i.test(spec().color) ? spec().color : '#000000'} onInput={(event) => change('color', event.currentTarget.value)} />
        </label>
        <label class="cfmt-check"><input type="checkbox" data-testid="font-bold" checked={spec().bold} onChange={(event) => change('bold', event.currentTarget.checked)} />{t('mail-compose-rt-bold')}</label>
        <label class="cfmt-check"><input type="checkbox" data-testid="font-italic" checked={spec().italic} onChange={(event) => change('italic', event.currentTarget.checked)} />{t('mail-compose-rt-italic')}</label>
        <fieldset class="cfmt-fontdlg__effects">
          <legend>{t('cfmt-effects')}</legend>
          <For each={DIALOG_EFFECTS}>{(entry) => (
            <label class="cfmt-check"><input type="checkbox" data-testid={`font-effect-${entry.kind}`} checked={!!spec().effects[entry.kind]} onChange={(event) => toggleEffect(entry.kind, event.currentTarget.checked)} />{t(entry.key)}</label>
          )}</For>
        </fieldset>
        <label>{t('cfmt-font-underline')}
          <select data-testid="font-underline" value={spec().underline} onChange={(event) => change('underline', event.currentTarget.value)}>
            <option value="">{t('cfmt-font-none')}</option>
            <option value="single">{t('cfmt-font-single')}</option>
            <For each={['double', 'dotted', 'dashed', 'wavy', 'thick']}>{(style) => <option value={style}>{t(`cfmt-underline-${style}`)}</option>}</For>
          </select>
        </label>
        <label>{t('cfmt-font-spacing')}
          <select data-testid="font-spacing" value={spec().spacing} onChange={(event) => change('spacing', event.currentTarget.value)}>
            <option value="normal">{t('cfmt-font-spacing-normal')}</option>
            <option value="condensed">{t('cfmt-spacing-condensed')}</option>
            <option value="expanded">{t('cfmt-spacing-expanded')}</option>
          </select>
        </label>
        <div class="cfmt-fontdlg__preview" data-testid="font-preview" style={previewCss(spec())} aria-label={t('cfmt-font-preview')}>{t('cfmt-font-sample')}</div>
        <div class="cfmt-fontdlg__default">
          <button type="button" class="btn btn--ghost" data-testid="font-set-default" onClick={() => void setDefault()}>{t('cfmt-font-set-default')}</button>
          <span aria-live="polite" data-testid="font-default-message">{message()}</span>
        </div>
      </div>
    </InsertDialog>
  );
}
