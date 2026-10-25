import { Icon, type IconName } from '../../components/Icon.tsx';
// Notes rich-text editor (plan §3 e6). A themed `contentEditable` surface — NO
// third-party (GPL) editor dependency — with a small formatting toolbar over the
// browser's `execCommand`. Every value the editor emits is run through
// `sanitizeNoteHtml` first, so the body the slice sends can never carry a script
// or an unsafe attribute regardless of what was pasted in (plan §3 e6 acceptance:
// "editor output sanitized — no script survives").

import { For, onMount, createEffect, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { sanitizeNoteHtml } from './sanitize.ts';

/** One toolbar command: an `execCommand` name + its label/glyph + a11y title key. */
interface ToolbarCommand {
  cmd: string;
  value?: string;
  label: IconName;
  titleKey: string;
}

const COMMANDS: readonly ToolbarCommand[] = [
  { cmd: 'bold', label: 'bold', titleKey: 'notes-fmt-bold' },
  { cmd: 'italic', label: 'italic', titleKey: 'notes-fmt-italic' },
  { cmd: 'underline', label: 'underline', titleKey: 'notes-fmt-underline' },
  { cmd: 'strikeThrough', label: 'strike', titleKey: 'notes-fmt-strike' },
  { cmd: 'formatBlock', value: 'h2', label: 'heading', titleKey: 'notes-fmt-heading' },
  { cmd: 'insertUnorderedList', label: 'bulletList', titleKey: 'notes-fmt-ul' },
  { cmd: 'insertOrderedList', label: 'numberList', titleKey: 'notes-fmt-ol' },
  { cmd: 'formatBlock', value: 'blockquote', label: 'quote', titleKey: 'notes-fmt-quote' },
  { cmd: 'removeFormat', label: 'clearFormat', titleKey: 'notes-fmt-clear' },
];

export interface NoteEditorProps {
  /** Switching this id reloads the surface's content (per-note, not per-keystroke). */
  noteId: string;
  /** The note's current body HTML (already client-held plaintext). */
  html: string;
  /** Called with the SANITIZED body HTML on every edit. */
  onInput: (html: string) => void;
  /** Optional label for the editing region. */
  ariaLabel?: string;
}

export function NoteEditor(props: NoteEditorProps): JSX.Element {
  let ref!: HTMLDivElement;

  /** Read the live DOM, sanitize, and surface it to the parent. */
  function emit(): void {
    props.onInput(sanitizeNoteHtml(ref.innerHTML));
  }

  // Whether the user has typed or formatted since this note was loaded. Until
  // then the surface follows the note's body (a change made elsewhere shows);
  // after it, the surface keeps what the user wrote.
  let typed = false;

  function typedInput(): void {
    typed = true;
    emit();
  }

  /** Run a formatting command against the current selection, then emit. */
  function run(c: ToolbarCommand): void {
    typed = true;
    ref.focus();
    // execCommand is deprecated but is the only dependency-free rich-text path
    // and is supported by every browser we target; jsdom is a no-op (tests
    // exercise the sanitize-on-input path directly).
    document.execCommand(c.cmd, false, c.value);
    emit();
  }

  function insertLink(): void {
    const url = globalThis.prompt?.(t('notes-link-prompt')) ?? '';
    if (url.length === 0) return;
    typed = true;
    ref.focus();
    document.execCommand('createLink', false, url);
    emit();
  }

  onMount(() => {
    ref.innerHTML = sanitizeNoteHtml(props.html);
  });

  // Reload content when the selected note changes, or when its body changes
  // before the user has typed in it (never under the cursor).
  let loadedId = props.noteId;
  let loadedHtml = props.html;
  createEffect(() => {
    const html = props.html;
    if (props.noteId !== loadedId) {
      loadedId = props.noteId;
      typed = false;
    } else if (typed || html === loadedHtml) {
      return;
    }
    loadedHtml = html;
    ref.innerHTML = sanitizeNoteHtml(html);
  });

  return (
    <div class="note-editor">
      <div class="note-editor__toolbar" role="toolbar" aria-label={t('notes-formatting')}>
        <For each={COMMANDS}>
          {(c) => (
            <button
              type="button"
              class="note-editor__btn"
              title={t(c.titleKey)}
              aria-label={t(c.titleKey)}
              onMouseDown={(e) => e.preventDefault() /* keep selection */}
              onClick={() => run(c)}
            >
              <Icon name={c.label} />
            </button>
          )}
        </For>
        <button
          type="button"
          class="note-editor__btn"
          title={t('notes-fmt-link')}
          aria-label={t('notes-fmt-link')}
          onMouseDown={(e) => e.preventDefault()}
          onClick={insertLink}
        >
          <Icon name="link" />
        </button>
      </div>
      <div
        ref={ref}
        class="note-editor__surface"
        contentEditable
        role="textbox"
        aria-multiline="true"
        aria-label={props.ariaLabel ?? t('notes-body')}
        data-testid="note-body"
        onInput={typedInput}
        onBlur={emit}
      />
    </div>
  );
}
