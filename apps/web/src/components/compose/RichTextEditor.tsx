import { Icon, type IconName } from '../Icon.tsx';
// ProseMirror rich-text editor for the composer (W1). Statically imports the
// ProseMirror packages; Compose pulls THIS module via `lazy()` so the editor +
// its libraries land in their own chunk (kept off the <250 KB login→inbox entry).
//
// The editor is the source of truth while rich mode is on. On every change it
// reports BOTH the serialized HTML (for the send payload) and a plain-text
// projection (so the existing crypto / DLP / dictation paths, which read the
// plain `body()` signal, keep working). It also reconciles external plain-text
// mutations (Assist rewrite, dictation) back into the document.

import { createEffect, createSignal, onCleanup, onMount, For, Show, type JSX } from 'solid-js';
import { EditorState, type Command } from 'prosemirror-state';
import { EditorView, type NodeView } from 'prosemirror-view';
import type { MarkType, Node as PMNode } from 'prosemirror-model';
import { baseKeymap, toggleMark, wrapIn } from 'prosemirror-commands';
import { keymap } from 'prosemirror-keymap';
import { history, undo, redo } from 'prosemirror-history';
import { wrapInList, splitListItem } from 'prosemirror-schema-list';
import type { NodeType } from 'prosemirror-model';
import { richSchema, docFromHtml, docFromText, htmlFromDoc, textFromDoc, COMPOSE_FONTS, COMPOSE_FONT_SIZES } from './richtext.ts';
import { imageFiles } from './inlineImages.ts';
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
const nImage: NodeType = need(richSchema.nodes.image, 'image');

/** Is `type` active in the current selection (so the toolbar button reads pressed)? */
function markActive(state: EditorState, type: MarkType): boolean {
  const { from, $from, to, empty } = state.selection;
  if (empty) return type.isInSet(state.storedMarks ?? $from.marks()) !== undefined;
  return state.doc.rangeHasMark(from, to, type);
}

interface ToolbarButton {
  readonly id: string;
  readonly labelKey: string;
  readonly glyph: IconName;
  readonly command: Command;
  /** Marks whose active-state lights the button (optional). */
  readonly activeMark?: MarkType;
}

const BUTTONS: readonly ToolbarButton[] = [
  { id: 'bold', labelKey: 'mail-compose-rt-bold', glyph: 'bold', command: toggleMark(mStrong), activeMark: mStrong },
  { id: 'italic', labelKey: 'mail-compose-rt-italic', glyph: 'italic', command: toggleMark(mEm), activeMark: mEm },
  { id: 'underline', labelKey: 'mail-compose-rt-underline', glyph: 'underline', command: toggleMark(mUnderline), activeMark: mUnderline },
  { id: 'strike', labelKey: 'mail-compose-rt-strike', glyph: 'strike', command: toggleMark(mStrike), activeMark: mStrike },
  { id: 'bullet', labelKey: 'mail-compose-rt-bullet', glyph: 'bulletList', command: wrapInList(nBullet) },
  { id: 'ordered', labelKey: 'mail-compose-rt-ordered', glyph: 'numberList', command: wrapInList(nOrdered) },
  { id: 'quote', labelKey: 'mail-compose-rt-quote', glyph: 'quote', command: wrapIn(nQuote) },
];

/** Imperative handle the composer uses for signature insert (W12) and draft
 *  resume (W9) without remounting the editor / losing rich formatting. */
export interface RichTextApi {
  /** Append parsed HTML as new blocks at the end of the document. */
  appendHtml: (html: string) => void;
  /** Replace the whole document with parsed HTML. */
  setHtml: (html: string) => void;
  focus: () => void;
  /** Re-read every image's preview (the composer made new ones). */
  refreshImages: () => void;
}

/** Images in the body (t29-e12). The composer uploads them and owns the list;
 *  the editor inserts, shows and serialises them. */
export interface InlineImageSupport {
  /** Content-IDs this draft uploaded: the only `cid:` sources a parse keeps. */
  owned: () => ReadonlySet<string>;
  /** The local preview of an image, or `null` when there is none (yet). */
  preview: (cid: string) => string | null;
  /** Upload `files`; resolves to the images that can be placed, in order. */
  upload: (files: File[]) => Promise<{ cid: string; alt: string }[]>;
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
  /** Inline images; absent = the editor offers none and keeps none. */
  inline?: InlineImageSupport;
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
  const [ribbonTab,setRibbonTab]=createSignal<'format'|'insert'>('format');
  function fontValue(name:string):string {const state=activeState();return String(state?.storedMarks?.find(mark=>mark.type.name===name)?.attrs['value']??state?.selection.$from.marks().find(mark=>mark.type.name===name)?.attrs['value']??'');}
  function setTextStyle(name:string,value:string):void {if(!view)return;const mark=richSchema.marks[name];if(!mark)return;const {from,to,empty}=view.state.selection;let tr=view.state.tr;if(empty)tr=value?tr.addStoredMark(mark.create({value})):tr.removeStoredMark(mark);else{tr=tr.removeMark(from,to,mark);if(value)tr=tr.addMark(from,to,mark.create({value}));}view.dispatch(tr);view.focus();}

  const owned = (): ReadonlySet<string> | undefined => props.inline?.owned();

  /** The editor shows an image from its local preview, not its `cid:`. */
  function imageView(node: PMNode): NodeView {
    const dom = document.createElement('img');
    dom.className = 'compose-rt__img';
    dom.alt = node.attrs['alt'] as string;
    dom.src = props.inline?.preview(node.attrs['cid'] as string) ?? '';
    dom.dataset['cid'] = node.attrs['cid'] as string;
    return { dom };
  }

  /** Upload image files and place them at `pos` (default: the selection). */
  async function placeImages(files: File[], pos?: number): Promise<void> {
    const support = props.inline;
    if (support === undefined || files.length === 0) return;
    const placed = await support.upload(files);
    if (view === undefined || placed.length === 0) return;
    let tr = view.state.tr;
    for (const p of placed) {
      const node = nImage.create({ cid: p.cid, alt: p.alt });
      tr = pos !== undefined ? tr.insert(Math.min(pos, tr.doc.content.size), node) : tr.replaceSelectionWith(node);
      if (pos !== undefined) pos += node.nodeSize;
    }
    view.dispatch(tr);
    view.focus();
  }

  function emit(state: EditorState): void {
    const html = htmlFromDoc(state.doc);
    const text = textFromDoc(state.doc);
    lastText = text;
    setActiveState(state);
    props.onChange(html, text);
  }

  onMount(() => {
    const state = EditorState.create({
      doc: docFromHtml(props.initialHtml, owned()),
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
      nodeViews: { image: (node) => imageView(node) },
      // A pasted or dropped image file is uploaded and placed; pasted HTML is
      // parsed under the schema, which keeps no image it does not own.
      handlePaste(_view, event) {
        const files = props.inline !== undefined ? imageFiles(event.clipboardData) : [];
        if (files.length === 0) return false;
        event.preventDefault();
        void placeImages(files);
        return true;
      },
      handleDrop(v, event) {
        const files = props.inline !== undefined ? imageFiles(event.dataTransfer) : [];
        if (files.length === 0) return false;
        event.preventDefault();
        const at = v.posAtCoords({ left: event.clientX, top: event.clientY });
        void placeImages(files, at?.pos);
        return true;
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
        const next = EditorState.create({ doc: docFromHtml(html, owned()), plugins: view.state.plugins });
        view.updateState(next);
        emit(next);
      },
      focus(): void {
        view?.focus();
      },
      refreshImages(): void {
        for (const img of Array.from(host.querySelectorAll<HTMLImageElement>('img.compose-rt__img'))) {
          const url = props.inline?.preview(img.dataset['cid'] ?? '') ?? null;
          if (url !== null && img.src !== url) img.src = url;
        }
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
      <div class="compose-rt__tabs" role="tablist" aria-label={t('mail-compose-ribbon')}><For each={['format','insert'] as const}>{tab=><button type="button" role="tab" aria-selected={ribbonTab()===tab} onClick={()=>setRibbonTab(tab)}>{t(`mail-compose-ribbon-${tab}`)}</button>}</For></div>
      <div class="compose-rt__toolbar" role="toolbar" aria-label={t('mail-compose-rt-toolbar')}>
        <button type="button" class="compose-rt__btn" aria-label={t('mail-compose-undo')} title={t('mail-compose-undo')} onMouseDown={event=>{event.preventDefault();run(undo);}}><Icon name="back"/></button>
        <button type="button" class="compose-rt__btn" aria-label={t('mail-compose-redo')} title={t('mail-compose-redo')} onMouseDown={event=>{event.preventDefault();run(redo);}}><Icon name="forward"/></button>
        <Show when={ribbonTab()==='format'}>
        <label class="compose-rt__field"><span>{t('mail-compose-font')}</span><select aria-label={t('mail-compose-font')} value={fontValue('fontFamily')} onChange={event=>setTextStyle('fontFamily',event.currentTarget.value)}><option value="">{t('mail-compose-font-inherit')}</option><For each={COMPOSE_FONTS}>{font=><option value={font}>{font}</option>}</For></select></label>
        <label class="compose-rt__field"><span>{t('mail-compose-font-size')}</span><select aria-label={t('mail-compose-font-size')} value={fontValue('fontSize')} onChange={event=>setTextStyle('fontSize',event.currentTarget.value)}><option value="">{t('mail-compose-font-inherit')}</option><For each={COMPOSE_FONT_SIZES}>{size=><option value={`${size}pt`}>{size} pt</option>}</For></select></label>
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
                <Icon name={btn.glyph} />
              </button>
            );
          }}
        </For>
        </Show><Show when={ribbonTab()==='insert'}>
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
          <Icon name="link" />
        </button>
        <Show when={props.inline !== undefined}>
          <label
            class={`compose-rt__btn ${a11y.focusable}`}
            title={t('mail-compose-rt-image')}
            data-testid="rt-image"
          >
            <span aria-hidden="true"><Icon name="image" /></span>
            <input
              type="file"
              accept="image/png,image/jpeg,image/gif,image/webp"
              multiple
              class={a11y.srOnly}
              aria-label={t('mail-compose-rt-image')}
              onChange={(e) => {
                const input = e.currentTarget;
                const files = Array.from(input.files ?? []);
                void placeImages(files).finally(() => {
                  // Reset so choosing the same file again fires another change.
                  input.value = '';
                });
              }}
            />
          </label>
        </Show>
        </Show>
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
