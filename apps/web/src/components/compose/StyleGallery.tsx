// The paragraph style gallery (Format tab): one button showing the current style
// that opens a list of styles, each drawn in its own look with its keyboard
// shortcut beside it. Normal, Heading 1 to 4, Quote and Code come first; the
// rest of the editor's presets follow. Arrow keys move, Enter applies, Esc closes.

import { createSignal, For, type JSX } from 'solid-js';
import type { EditorState } from 'prosemirror-state';
import { RibbonPopover } from './ribbonParts.tsx';
import { t } from '../../i18n/index.ts';
import { Icon } from '../Icon.tsx';

export interface StyleEntry { id: string; shortcut?: string; group: 'main' | 'more' }

export const STYLE_ENTRIES: readonly StyleEntry[] = [
  { id: 'normal', shortcut: 'Mod-Alt-0', group: 'main' },
  { id: 'heading1', shortcut: 'Mod-Alt-1', group: 'main' },
  { id: 'heading2', shortcut: 'Mod-Alt-2', group: 'main' },
  { id: 'heading3', shortcut: 'Mod-Alt-3', group: 'main' },
  { id: 'heading4', shortcut: 'Mod-Alt-4', group: 'main' },
  { id: 'quote', shortcut: 'Mod-Alt-q', group: 'main' },
  { id: 'code', shortcut: 'Mod-Alt-c', group: 'main' },
  { id: 'title', group: 'more' },
  { id: 'subtitle', group: 'more' },
  { id: 'no-spacing', group: 'more' },
  { id: 'heading5', group: 'more' },
  { id: 'heading6', group: 'more' },
];

/** `Mod-Alt-1` as the user sees it: Ctrl+Alt+1, or ⌘⌥1 on Apple platforms. */
export function shortcutLabel(shortcut: string, apple = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform)): string {
  const parts = shortcut.split('-');
  const key = parts.pop()!.toUpperCase();
  const names = parts.map((part) => (part === 'Mod' ? (apple ? '⌘' : 'Ctrl') : part === 'Alt' ? (apple ? '⌥' : 'Alt') : part === 'Shift' ? (apple ? '⇧' : 'Shift') : part));
  return apple ? `${names.join('')}${key}` : [...names, key].join('+');
}

/** Which style the selection is in, as a `STYLE_ENTRIES` id. */
export function currentStyle(state: EditorState | null): string {
  if (!state) return 'normal';
  const { $from } = state.selection;
  if ($from.parent.type.name === 'code_block') return 'code';
  const preset = $from.parent.attrs['preset'];
  if (typeof preset === 'string' && preset !== '') return preset;
  if ($from.parent.type.name === 'heading') return `heading${String($from.parent.attrs['level'])}`;
  for (let depth = $from.depth; depth > 0; depth -= 1) if ($from.node(depth).type.name === 'blockquote') return 'quote';
  return 'normal';
}

export function StyleGallery(props: { state: () => EditorState | null; apply: (style: string) => void }): JSX.Element {
  const [open, setOpen] = createSignal(false);
  let list: HTMLDivElement | undefined;
  const current = (): string => currentStyle(props.state());
  const focusSelected = (): void => {
    queueMicrotask(() => (list?.querySelector<HTMLElement>('[aria-selected="true"]') ?? list?.querySelector<HTMLElement>('[role="option"]'))?.focus());
  };
  const move = (event: KeyboardEvent, delta: number): void => {
    const options = Array.from(list?.querySelectorAll<HTMLElement>('[role="option"]') ?? []);
    const at = options.indexOf(event.target as HTMLElement);
    options[Math.max(0, Math.min(options.length - 1, at + delta))]?.focus();
    event.preventDefault();
  };
  const choose = (id: string): void => {
    setOpen(false);
    props.apply(id);
  };
  return (
    <RibbonPopover
      label={t(`mail-editor-style-${current()}`)}
      popoverLabel={t('mail-editor-style')}
      ariaLabel={`${t('mail-editor-style')}: ${t(`mail-editor-style-${current()}`)}`}
      icon="heading"
      keepSelection
      buttonClass="compose-rt__btn compose-rt__style-button"
      buttonTestId="rt-style-gallery"
      open={open()}
      onOpenChange={(next) => { setOpen(next); if (next) focusSelected(); }}
    >
      <div
        ref={list}
        class="compose-rt__styles"
        role="listbox"
        aria-label={t('mail-editor-style')}
        data-testid="rt-style-list"
        onKeyDown={(event) => {
          if (event.key === 'ArrowDown') move(event, 1);
          else if (event.key === 'ArrowUp') move(event, -1);
          else if (event.key === 'Home') { (list?.querySelector('[role="option"]') as HTMLElement | null)?.focus(); event.preventDefault(); }
        }}
      >
        <For each={STYLE_ENTRIES}>{(entry) => (
          <button
            type="button"
            role="option"
            class="compose-rt__style-option"
            data-style={entry.id}
            data-more={entry.group === 'more' ? '' : undefined}
            aria-selected={current() === entry.id ? 'true' : 'false'}
            data-testid={`rt-style-${entry.id}`}
            onMouseDown={(event) => event.preventDefault()}
            onClick={() => choose(entry.id)}
          >
            <span class="compose-rt__style-preview" data-style={entry.id}>{t(`mail-editor-style-${entry.id}`)}</span>
            {entry.shortcut !== undefined ? <kbd class="compose-rt__style-keys">{shortcutLabel(entry.shortcut)}</kbd> : null}
            {current() === entry.id ? <span class="compose-rt__style-check"><Icon name="check" /></span> : null}
          </button>
        )}</For>
      </div>
    </RibbonPopover>
  );
}
