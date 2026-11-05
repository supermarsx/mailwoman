// The Picture tab's groups (rich-text editor), shown while an image is selected:
// Size, Arrange, Text, Style and Edit. Each is a three-row ribbon group with an
// icon and a title below. The controls only call back; the commands and the
// canvas work live in imageCommands.ts.

import { createSignal, For, Show, type JSX } from 'solid-js';
import type { Command, EditorState } from 'prosemirror-state';
import { RibbonGroup, RibbonItem } from '../bars/index.ts';
import { Icon, type IconName } from '../Icon.tsx';
import { RibbonPopover } from './ribbonParts.tsx';
import { t } from '../../i18n/index.ts';
import * as a11y from '../mailA11y.css.ts';
import { IMAGE_BORDERS, IMAGE_RADII, lockedSide, scaledSize, type ImageAlign } from './imageAttrs.ts';
import { removeImage, selectedImage, setImageAlt, setImageLook, type ImageOp } from './imageCommands.ts';

export interface PictureProps {
  state: () => EditorState | null;
  execute: (command: Command) => void;
  /** The picture's own pixel size once it has loaded, or null. */
  naturalSize: () => { width: number; height: number } | null;
  transform: (op: ImageOp) => void;
  replace: (file: File) => void;
  /** The host can store a new image (a draft's uploads, or embedded images). */
  canStore: boolean;
}

const ALIGNS: readonly [ImageAlign | null, IconName, string][] = [
  [null, 'paragraph', 'inline'],
  ['left', 'alignLeft', 'left'],
  ['center', 'alignCenter', 'center'],
  ['right', 'alignRight', 'right'],
];
const PRESETS = [0.25, 0.5, 1] as const;

export function PictureGroups(props: PictureProps): JSX.Element {
  const [locked, setLocked] = createSignal(true);
  const [cropOpen, setCropOpen] = createSignal(false);
  const [crop, setCrop] = createSignal({ left: 0, top: 0, right: 0, bottom: 0 });
  const attrs = (): Record<string, unknown> => {
    const state = props.state();
    return (state ? selectedImage(state)?.node.attrs : undefined) ?? {};
  };
  const has = (): boolean => selectedImage(props.state() ?? ({} as EditorState)) !== null;
  const width = (): number | null => (attrs()['width'] as number | null) ?? props.naturalSize()?.width ?? null;
  const height = (): number | null => (attrs()['height'] as number | null) ?? (attrs()['width'] === null || attrs()['width'] === undefined ? props.naturalSize()?.height ?? null : null);
  const ratio = (): number => {
    const natural = props.naturalSize();
    if (natural) return natural.width / natural.height;
    const w = attrs()['width'] as number | null, h = attrs()['height'] as number | null;
    return w !== null && h !== null && h > 0 ? w / h : 1;
  };
  const setSize = (changed: 'width' | 'height', raw: string): void => {
    const value = Number(raw);
    if (!Number.isInteger(value) || value < 1) return;
    const other = lockedSide(changed, value, ratio());
    props.execute(setImageLook(locked() ? (changed === 'width' ? { width: value, height: other } : { width: other, height: value }) : { [changed]: value }));
  };
  const button = (testId: string, label: string, icon: IconName, onClick: () => void, pressed?: () => boolean): JSX.Element => (
    <button
      type="button"
      class={`compose-rt__btn ${a11y.focusable}`}
      classList={{ 'compose-rt__btn--active': pressed?.() === true }}
      aria-pressed={pressed === undefined ? undefined : pressed()}
      disabled={!has()}
      data-testid={testId}
      aria-label={label}
      data-tooltip={label}
      onMouseDown={(event) => event.preventDefault()}
      onClick={onClick}
    >
      <Icon name={icon} />
    </button>
  );
  const row = (...controls: JSX.Element[]): JSX.Element => <div class="compose-rib__row">{controls}</div>;
  const field = (id: string, label: string, value: string, onCommit: (value: string) => void, extra: JSX.InputHTMLAttributes<HTMLInputElement> = {}): JSX.Element => (
    <label class="compose-rib__field">
      <span class="compose-rib__text">{label}</span>
      <input data-testid={id} aria-label={label} disabled={!has()} value={value} onChange={(event) => onCommit(event.currentTarget.value)} {...extra} />
    </label>
  );
  return (
    <>
      <RibbonGroup label={t('mail-picture-group-size')} priority={60} icon="image" testId="rt-group-picture-size">
        <RibbonItem>{row(
          field('rt-picture-width', t('mail-picture-width'), String(width() ?? ''), (v) => setSize('width', v), { type: 'number', min: 1, max: 4000 }),
          field('rt-picture-height', t('mail-picture-height'), String(height() ?? ''), (v) => setSize('height', v), { type: 'number', min: 1, max: 4000 }),
          <button type="button" class={`compose-rt__btn ${a11y.focusable}`} aria-pressed={locked()} disabled={!has()} data-testid="rt-picture-lock" aria-label={t('mail-picture-lock')} data-tooltip={t('mail-picture-lock')} onMouseDown={(event) => event.preventDefault()} onClick={() => setLocked(!locked())}><Icon name={locked() ? 'lock' : 'unlock'} /></button>,
        )}</RibbonItem>
        <RibbonItem>{row(<For each={PRESETS}>{(factor) => (
          <button
            type="button"
            class={`compose-rt__btn ${a11y.focusable}`}
            disabled={!has() || props.naturalSize() === null}
            data-testid={`rt-picture-scale-${factor * 100}`}
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => { const natural = props.naturalSize(); if (natural) props.execute(setImageLook(scaledSize(natural, factor))); }}
          >{factor * 100}%</button>
        )}</For>)}</RibbonItem>
        <RibbonItem>
          <button type="button" class={`compose-rt__btn ${a11y.focusable}`} disabled={!has()} data-testid="rt-picture-reset-size" onMouseDown={(event) => event.preventDefault()} onClick={() => props.execute(setImageLook({ width: null, height: null }))}><Icon name="restore" /><span class="compose-rib__text">{t('mail-picture-reset-size')}</span></button>
        </RibbonItem>
      </RibbonGroup>

      <RibbonGroup label={t('mail-picture-group-arrange')} priority={55} icon="alignCenter" testId="rt-group-picture-arrange">
        <RibbonItem>{row(<For each={ALIGNS}>{([align, icon, name]) => button(`rt-picture-align-${name}`, t(`mail-picture-align-${name}`), icon, () => props.execute(setImageLook({ align })), () => (attrs()['align'] ?? null) === align)}</For>)}</RibbonItem>
      </RibbonGroup>

      <RibbonGroup label={t('mail-picture-group-text')} priority={50} icon="edit" testId="rt-group-picture-text">
        <RibbonItem>{field('rt-picture-alt', t('mail-picture-alt'), String(attrs()['alt'] ?? ''), (v) => props.execute(setImageAlt(v)), { maxLength: 200 })}</RibbonItem>
        <RibbonItem>{field('rt-picture-title', t('mail-picture-title'), String(attrs()['title'] ?? ''), (v) => props.execute(setImageLook({ title: v })), { maxLength: 200 })}</RibbonItem>
      </RibbonGroup>

      <RibbonGroup label={t('mail-picture-group-style')} priority={45} icon="paint" testId="rt-group-picture-style">
        <RibbonItem>
          <label class="compose-rib__field"><span class="compose-rib__text">{t('mail-picture-border')}</span>
            <select data-testid="rt-picture-border" aria-label={t('mail-picture-border')} disabled={!has()} value={String(attrs()['border'] ?? 0)} onChange={(event) => props.execute(setImageLook({ border: Number(event.currentTarget.value) }))}>
              <For each={IMAGE_BORDERS}>{(px) => <option value={px}>{px === 0 ? t('mail-picture-none') : `${px} px`}</option>}</For>
            </select>
          </label>
        </RibbonItem>
        <RibbonItem>
          <label class="compose-rib__field"><span class="compose-rib__text">{t('mail-picture-radius')}</span>
            <select data-testid="rt-picture-radius" aria-label={t('mail-picture-radius')} disabled={!has()} value={String(attrs()['radius'] ?? 0)} onChange={(event) => props.execute(setImageLook({ radius: Number(event.currentTarget.value) }))}>
              <For each={IMAGE_RADII}>{(px) => <option value={px}>{px === 0 ? t('mail-picture-none') : px >= 9999 ? t('mail-picture-round') : `${px} px`}</option>}</For>
            </select>
          </label>
        </RibbonItem>
      </RibbonGroup>

      <RibbonGroup label={t('mail-picture-group-edit')} priority={40} icon="crop" testId="rt-group-picture-edit">
        <RibbonItem>{row(
          button('rt-picture-rotate-ccw', t('mail-picture-rotate-ccw'), 'rotateCcw', () => props.transform({ kind: 'rotate', dir: 'ccw' })),
          button('rt-picture-rotate-cw', t('mail-picture-rotate-cw'), 'rotateCw', () => props.transform({ kind: 'rotate', dir: 'cw' })),
          button('rt-picture-flip-h', t('mail-picture-flip-h'), 'flipH', () => props.transform({ kind: 'flip', axis: 'h' })),
          button('rt-picture-flip-v', t('mail-picture-flip-v'), 'flipV', () => props.transform({ kind: 'flip', axis: 'v' })),
        )}</RibbonItem>
        <RibbonItem>{row(
          <RibbonPopover label={t('mail-picture-crop')} icon="crop" keepSelection buttonClass={`compose-rt__btn ${a11y.focusable}`} buttonTestId="rt-picture-crop" disabled={!has()} open={cropOpen()} onOpenChange={setCropOpen}>
            <div class="compose-rt__options compose-rib__form" data-testid="rt-picture-crop-form">
              <For each={['left', 'top', 'right', 'bottom'] as const}>{(side) => (
                <label>{t(`mail-picture-crop-${side}`)}
                  <input type="number" min="0" max="90" step="5" value={Math.round(crop()[side] * 100)} aria-label={t(`mail-picture-crop-${side}`)} onInput={(event) => setCrop({ ...crop(), [side]: Math.min(90, Math.max(0, Number(event.currentTarget.value) || 0)) / 100 })} />
                </label>
              )}</For>
              <button type="button" class={`btn btn--ghost ${a11y.focusable}`} data-testid="rt-picture-crop-apply" onClick={() => { props.transform({ kind: 'crop', ...crop() }); setCrop({ left: 0, top: 0, right: 0, bottom: 0 }); setCropOpen(false); }}>{t('mail-picture-crop-apply')}</button>
            </div>
          </RibbonPopover>,
          <Show when={props.canStore}>
            <label class={`compose-rt__btn ${a11y.focusable}`} aria-disabled={!has()} data-testid="rt-picture-replace">
              <Icon name="refresh" /><span class="compose-rib__text">{t('mail-picture-replace')}</span>
              <input type="file" accept="image/png,image/jpeg,image/gif,image/webp" class={a11y.srOnly} disabled={!has()} aria-label={t('mail-picture-replace')} onChange={(event) => { const file = event.currentTarget.files?.[0]; event.currentTarget.value = ''; if (file) props.replace(file); }} />
            </label>
          </Show>,
        )}</RibbonItem>
        <RibbonItem>
          <button type="button" class={`compose-rt__btn ${a11y.focusable}`} disabled={!has()} data-testid="rt-picture-remove" onMouseDown={(event) => event.preventDefault()} onClick={() => props.execute(removeImage)}><Icon name="delete" /><span class="compose-rib__text">{t('mail-picture-remove')}</span></button>
        </RibbonItem>
      </RibbonGroup>
    </>
  );
}
