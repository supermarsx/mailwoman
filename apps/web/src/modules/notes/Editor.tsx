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
  label: string;
  titleKey: string;
}

const COMMANDS: readonly ToolbarCommand[] = [
  { cmd: 'bold', label: 'B', titleKey: 'notes-fmt-bold' },
  { cmd: 'italic', label: 'I', titleKey: 'notes-fmt-italic' },
  { cmd: 'underline', label: 'U', titleKey: 'notes-fmt-underline' },
  { cmd: 'strikeThrough', label: 'S', titleKey: 'notes-fmt-strike' },
  { cmd: 'formatBlock', value: 'h2', label: 'H', titleKey: 'notes-fmt-heading' },
  { cmd: 'insertUnorderedList', label: '•', titleKey: 'notes-fmt-ul' },
  { cmd: 'insertOrderedList', label: '1.', titleKey: 'notes-fmt-ol' },
  { cmd: 'formatBlock', value: 'blockquote', label: '❝', titleKey: 'notes-fmt-quote' },
  { cmd: 'removeFormat', label: '⌫', titleKey: 'notes-fmt-clear' },
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

  /** Run a formatting command against the current selection, then emit. */
  function run(c: ToolbarCommand): void {
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
    ref.focus();
    document.execCommand('createLink', false, url);
    emit();
  }

  onMount(() => {
    ref.innerHTML = sanitizeNoteHtml(props.html);
  });

  // Reload content only when the selected note changes (guards cursor jumps).
  let loadedId = props.noteId;
  createEffect(() => {
    if (props.noteId !== loadedId) {
      loadedId = props.noteId;
      ref.innerHTML = sanitizeNoteHtml(props.html);
    }
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
              {c.label}
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
          🔗
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
        onInput={emit}
        onBlur={emit}
      />
    </div>
  );
}
