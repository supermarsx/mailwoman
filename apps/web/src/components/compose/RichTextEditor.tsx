// ProseMirror rich-text editor for the composer (W1). Statically imports the
// ProseMirror packages; Compose pulls THIS module via `lazy()` so the editor +
// its libraries land in their own chunk (kept off the <250 KB login→inbox entry).
//
// The editor is the source of truth while rich mode is on. On every change it
// reports BOTH the serialized HTML (for the send payload) and a plain-text
// projection (so the existing crypto / DLP / dictation paths, which read the
// plain `body()` signal, keep working). It also reconciles external plain-text
// mutations (Assist rewrite, dictation) back into the document.

import { createEffect, createSignal, onCleanup, onMount, For, type JSX } from 'solid-js';
import { EditorState, type Command } from 'prosemirror-state';
import { EditorView } from 'prosemirror-view';
import type { MarkType } from 'prosemirror-model';
import { baseKeymap, toggleMark, wrapIn } from 'prosemirror-commands';
import { keymap } from 'prosemirror-keymap';
import { history, undo, redo } from 'prosemirror-history';
import { wrapInList, splitListItem } from 'prosemirror-schema-list';
import type { NodeType } from 'prosemirror-model';
import { richSchema, docFromHtml, docFromText, htmlFromDoc, textFromDoc } from './richtext.ts';
import { t } from '../../i18n/index.ts';
import * as a11y from '../mailA11y.css.ts';
import './richtext.css';

/** Assert a schema mark/node exists (the composer schema always defines these). */
function need<T>(value: T | undefined, name: string): T {
  if (value === undefined) throw new Error(`richtext: schema is missing "${name}"`);
  return value;
}

const mStrong: MarkType = need(richSchema.marks.strong, 'strong');
const mEm: MarkType = need(richSchema.marks.em, 'em');
const mUnderline: MarkType = need(richSchema.marks.underline, 'underline');
const mStrike: MarkType = need(richSchema.marks.strikethrough, 'strikethrough');
const mLink: MarkType = need(richSchema.marks.link, 'link');
const nBullet: NodeType = need(richSchema.nodes.bullet_list, 'bullet_list');
const nOrdered: NodeType = need(richSchema.nodes.ordered_list, 'ordered_list');
const nQuote: NodeType = need(richSchema.nodes.blockquote, 'blockquote');
const nListItem: NodeType = need(richSchema.nodes.list_item, 'list_item');

/** Is `type` active in the current selection (so the toolbar button reads pressed)? */
function markActive(state: EditorState, type: MarkType): boolean {
  const { from, $from, to, empty } = state.selection;
  if (empty) return type.isInSet(state.storedMarks ?? $from.marks()) !== undefined;
  return state.doc.rangeHasMark(from, to, type);
}

interface ToolbarButton {
  readonly id: string;
  readonly labelKey: string;
  readonly glyph: string;
  readonly command: Command;
  /** Marks whose active-state lights the button (optional). */
  readonly activeMark?: MarkType;
}

const BUTTONS: readonly ToolbarButton[] = [
  { id: 'bold', labelKey: 'mail-compose-rt-bold', glyph: 'B', command: toggleMark(mStrong), activeMark: mStrong },
  { id: 'italic', labelKey: 'mail-compose-rt-italic', glyph: 'I', command: toggleMark(mEm), activeMark: mEm },
  { id: 'underline', labelKey: 'mail-compose-rt-underline', glyph: 'U', command: toggleMark(mUnderline), activeMark: mUnderline },
  { id: 'strike', labelKey: 'mail-compose-rt-strike', glyph: 'S', command: toggleMark(mStrike), activeMark: mStrike },
  { id: 'bullet', labelKey: 'mail-compose-rt-bullet', glyph: '•', command: wrapInList(nBullet) },
  { id: 'ordered', labelKey: 'mail-compose-rt-ordered', glyph: '1.', command: wrapInList(nOrdered) },
  { id: 'quote', labelKey: 'mail-compose-rt-quote', glyph: '❝', command: wrapIn(nQuote) },
];

/** Imperative handle the composer uses for signature insert (W12) and draft
 *  resume (W9) without remounting the editor / losing rich formatting. */
export interface RichTextApi {
  /** Append parsed HTML as new blocks at the end of the document. */
  appendHtml: (html: string) => void;
  /** Replace the whole document with parsed HTML. */
  setHtml: (html: string) => void;
  focus: () => void;
}

export interface RichTextEditorProps {
  /** Initial HTML to seed the document (from the composer's `bodyHtml`). */
  initialHtml: string;
  /** External plain-text source (dictation / Assist) reconciled into the doc. */
  externalText?: () => string;
  /** Reports the serialized HTML + plain-text projection after every change. */
  onChange: (html: string, text: string) => void;
  /** Accessible name for the editable region (matches the Body field label). */
  ariaLabel: string;
  /** Receives the imperative handle once the view is live. */
  onReady?: (api: RichTextApi) => void;
}

export function RichTextEditor(props: RichTextEditorProps): JSX.Element {
  let host!: HTMLDivElement;
  let view: EditorView | undefined;
  // The last plain text WE emitted — used to tell our own edits from external
  // (dictation / Assist) mutations of the shared `body()` signal.
  let lastText = '';
  const [activeState, setActiveState] = createSignal<EditorState | null>(null);
  const [linkOpen, setLinkOpen] = createSignal(false);
  const [linkUrl, setLinkUrl] = createSignal('');

  function emit(state: EditorState): void {
    const html = htmlFromDoc(state.doc);
    const text = textFromDoc(state.doc);
    lastText = text;
    setActiveState(state);
    props.onChange(html, text);
  }

  onMount(() => {
    const state = EditorState.create({
      doc: docFromHtml(props.initialHtml),
      plugins: [
        history(),
        keymap({
          'Mod-z': undo,
          'Mod-y': redo,
          'Mod-Shift-z': redo,
          'Mod-b': toggleMark(mStrong),
          'Mod-i': toggleMark(mEm),
          'Mod-u': toggleMark(mUnderline),
          Enter: splitListItem(nListItem),
        }),
        keymap(baseKeymap),
      ],
    });
    view = new EditorView(host, {
      state,
      dispatchTransaction(tr) {
        if (view === undefined) return;
        const next = view.state.apply(tr);
        view.updateState(next);
        if (tr.docChanged) emit(next);
        else setActiveState(next);
      },
      attributes: {
        'aria-label': props.ariaLabel,
        'aria-multiline': 'true',
        role: 'textbox',
        'data-testid': 'compose-richtext',
      },
    });
    emit(view.state);

    props.onReady?.({
      appendHtml(html: string): void {
        if (view === undefined) return;
        const frag = docFromHtml(html).content;
        const next = view.state.apply(view.state.tr.insert(view.state.doc.content.size, frag));
        view.updateState(next);
        emit(next);
        view.focus();
      },
      setHtml(html: string): void {
        if (view === undefined) return;
        const next = EditorState.create({ doc: docFromHtml(html), plugins: view.state.plugins });
        view.updateState(next);
        emit(next);
      },
      focus(): void {
        view?.focus();
      },
    });
  });

  onCleanup(() => view?.destroy());

  // Reconcile external plain-text edits (dictation / Assist rewrite) into the
  // document. Tracks `externalText()`; only replaces the doc when it diverges
  // from what we last emitted, so the editor's own edits never loop back here.
  createEffect(() => {
    const ext = props.externalText;
    if (ext === undefined) return;
    const incoming = ext();
    if (view === undefined || incoming === lastText) return;
    const next = EditorState.create({ doc: docFromText(incoming), plugins: view.state.plugins });
    view.updateState(next);
    emit(next);
  });

  /** Run a command against the live view and return focus to the editor. */
  function run(command: Command): void {
    if (view === undefined) return;
    command(view.state, view.dispatch, view);
    view.focus();
  }

  function applyLink(): void {
    if (view === undefined) return;
    const url = linkUrl().trim();
    setLinkOpen(false);
    setLinkUrl('');
    if (url === '') {
      run(toggleMark(mLink, { href: '' }));
      return;
    }
    // Only http(s)/mailto — never javascript: or data: URLs into the body.
    if (!/^(https?:|mailto:)/i.test(url)) return;
    run(toggleMark(mLink, { href: url }));
  }

  return (
    <div class="compose-rt" data-testid="compose-rt">
      <div class="compose-rt__toolbar" role="toolbar" aria-label={t('mail-compose-rt-toolbar')}>
        <For each={BUTTONS}>
          {(btn) => {
            const state = () => activeState();
            const active = () => {
              const s = state();
              return s !== null && btn.activeMark !== undefined ? markActive(s, btn.activeMark) : false;
            };
            return (
              <button
                type="button"
                class={`compose-rt__btn ${a11y.focusable}`}
                classList={{ 'compose-rt__btn--active': active() }}
                aria-pressed={active()}
                aria-label={t(btn.labelKey)}
                title={t(btn.labelKey)}
                data-testid={`rt-${btn.id}`}
                onMouseDown={(e) => {
                  // mousedown (not click) so the editor keeps its selection.
                  e.preventDefault();
                  run(btn.command);
                }}
              >
                {btn.glyph}
              </button>
            );
          }}
        </For>
        <button
          type="button"
          class={`compose-rt__btn ${a11y.focusable}`}
          aria-label={t('mail-compose-rt-link')}
          title={t('mail-compose-rt-link')}
          data-testid="rt-link"
          onMouseDown={(e) => {
            e.preventDefault();
            setLinkOpen((v) => !v);
          }}
        >
          🔗
        </button>
      </div>

      {linkOpen() && (
        <div class="compose-rt__link" data-testid="rt-link-row">
          <input
            type="url"
            class={a11y.focusable}
            placeholder={t('mail-compose-rt-link-placeholder')}
            aria-label={t('mail-compose-rt-link')}
            value={linkUrl()}
            onInput={(e) => setLinkUrl(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                applyLink();
              }
            }}
          />
          <button
            type="button"
            class={`btn btn--ghost ${a11y.focusable}`}
            data-testid="rt-link-apply"
            onClick={() => applyLink()}
          >
            {t('mail-compose-rt-link-apply')}
          </button>
        </div>
      )}

      <div class="compose-rt__surface" ref={host} />
    </div>
  );
}

export default RichTextEditor;
