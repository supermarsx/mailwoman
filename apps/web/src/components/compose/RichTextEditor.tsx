import {Portal} from 'solid-js/web';
import {createAdvancedEditorTools} from './AdvancedEditorTools.tsx';
import {Feature,createFormatControls,createPainter,type FormatAdapter} from './FormatControls.tsx';
import {useComposeFeature} from './composeFeatures.ts';
import {applyListStyle,applyPaint,canContinueNumbering,canOutdent,capturePaint,changeIndent,clearFormatting,currentListType,blockAttrs,enclosingList,liftItem,setAlignment,setDirection,setLineSpacing,setListStart,setSpacing,sinkItem,stepFontSizeCommand,type PaintedFormat} from './richFormatCommands.ts';
import {chainCommands} from 'prosemirror-commands';
import './compose-lists.css';
import {RibbonGroup,RibbonItem} from '../bars/index.ts';
import {EDITOR_TABS,SLOT_TABS,EditorRibbon,createEditorRibbon,type EditorSlots,type EditorTab} from './EditorRibbon.tsx';
import {PictureGroups} from './PictureGroups.tsx';
import {TableLayoutGroups} from './TableLayoutGroups.tsx';
import {imageStyle,cleanImageLook} from './imageAttrs.ts';
import {renderWithCanvas,selectedImage,setImageSource,transformImage,type ImageIO,type ImageOp} from './imageCommands.ts';
import {inTable} from './tableLayout.ts';
import {InRibbonSlot,RibbonPopover} from './ribbonParts.tsx';
import {paragraphStyle,tableLimitPlugin} from './richCommands.ts';
import {tableEditing,goToNextCell} from 'prosemirror-tables';
import {COMPOSER_FONTS} from './fontCatalog.ts';
import {SURFACE_CHANGED,cssColorHex,surfaceTone,toggledBackground,viewColorPlugin} from './messageBackground.ts';
import {AppContext} from '../../state/context.ts';
import type {ComposeBackground} from '../../theme/appearance.ts';
import './composerFonts.css';
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

import { createEffect, createMemo, createSignal, on, onCleanup, onMount, useContext, For, Show, type JSX } from 'solid-js';
import { EditorState, type Command } from 'prosemirror-state';
import { EditorView, type NodeView } from 'prosemirror-view';
import { Slice, type MarkType, type Node as PMNode } from 'prosemirror-model';
import { baseKeymap, toggleMark, wrapIn } from 'prosemirror-commands';
import { keymap } from 'prosemirror-keymap';
import { history, undo, redo, undoDepth, redoDepth, closeHistory } from 'prosemirror-history';
import { wrapInList, splitListItem } from 'prosemirror-schema-list';
import type { NodeType } from 'prosemirror-model';
import { richSchema, docFromHtml, docFromText, htmlFromDoc, textFromDoc, COMPOSE_FONT_SIZES, EMBEDDED_IMAGE } from './richtext.ts';
import { imageFiles, INLINE_IMAGE_TYPES } from './inlineImages.ts';
import { t } from '../../i18n/index.ts';
import * as a11y from '../mailA11y.css.ts';
import './richtext.css';
import './compose-extras.css';

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
const nRule: NodeType = need(richSchema.nodes.horizontal_rule, 'horizontal_rule');

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
/** Bold, italic, underline, strike: the Font group's second row. */
const MARK_BUTTONS = BUTTONS.filter((b) => b.activeMark !== undefined);
/** Lists and quote: the Paragraph group's first row. */
const BLOCK_BUTTONS = BUTTONS.filter((b) => b.activeMark === undefined && b.id === 'quote');

/** Imperative handle the composer uses for signature insert (W12) and draft
 *  resume (W9) without remounting the editor / losing rich formatting. */
export interface RichTextApi {
  /** Settle pending HTML-source sanitization before saving or submitting. */
  whenReady?:()=>Promise<void>;
  /** Append parsed HTML as new blocks at the end of the document. */
  appendHtml: (html: string) => void;
  /** Insert parsed HTML at the caret (replacing the selection); `inline` merges into the current paragraph. */
  insertHtml?: (html: string, inline: boolean) => void;
  /** Replace the whole document with parsed HTML. */
  setHtml: (html: string) => void;
  /** Explicit user replacement that remains undoable. */
  replaceHtml?: (html:string)=>void;
  focus: () => void;
  /** Re-read every image's preview (the composer made new ones). */
  refreshImages: () => void;
  undo?:()=>void;
  redo?:()=>void;
  historyDepth?:()=>{undo:number;redo:number};
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

/** Images kept in the HTML itself as raster `data:` URLs (t49, the signature
 *  editor: a signature has no uploads). `fits` says whether images adding
 *  `bytes` of `data:` URL still fit the host's size limit. */
export interface EmbeddedImageSupport {
  fits: (bytes: number) => boolean;
  /** An image was refused: not a raster type, or too large. */
  refused: (reason: 'type' | 'size', name: string) => void;
}

/** The editor's ribbon groups, by tab: Format = clipboard, font, paragraph,
 *  editing, view; Insert = insert, tableInsert; Table = table, tableShading. */
export type EditorGroup = 'clipboard'|'font'|'paragraph'|'editing'|'view'|'insert'|'tableInsert'|'table'|'tableShading';

/** What the caret or selection is in: drives the Picture and Table layout tabs. */
export interface EditorContext { image: boolean; table: boolean }

export interface RichTextEditorProps {
  /** Initial HTML to seed the document (from the composer's `bodyHtml`). */
  initialHtml: string;
  /** The host's ribbon shows the undo/redo history (the editor's own ribbon then omits it). */
  externalHistory?: boolean;
  /** Where the editor's own ribbon goes when the host gives no `ribbonSlots`. */
  toolbarMount?: HTMLElement;
  /** The host's `EditorRibbon` slots, one per tab (t38, t49): the editor's groups join each
   *  tab's panel. Without them the editor shows an `EditorRibbon` of its own. */
  ribbonSlots?: () => EditorSlots;
  /** Optional groups a host leaves out; message and signature editors use the full set. */
  omitGroups?: readonly EditorGroup[];
  /** External plain-text source (dictation / Assist) reconciled into the doc. */
  externalText?: () => string;
  /** Reports the serialized HTML + plain-text projection after every change. */
  onChange: (html: string, text: string) => void;
  /** Accessible name for the editable region (matches the Body field label). */
  ariaLabel: string;
  /** Receives the imperative handle once the view is live. */
  onReady?: (api: RichTextApi) => void;
  /** Reports what the selection is on, for contextual ribbon tabs: an image, or a table cell. */
  onContext?: (context: EditorContext) => void;
  /** Inline images; absent = the editor offers none and keeps none. */
  inline?: InlineImageSupport;
  /** Embedded images instead of inline ones (the signature editor). */
  embedded?: EmbeddedImageSupport;
}

export function RichTextEditor(props: RichTextEditorProps): JSX.Element {
  let host!: HTMLDivElement;
  let view: EditorView | undefined;
  const [editorApi,setEditorApi]=createSignal<RichTextApi|null>(null);
  // The last plain text WE emitted — used to tell our own edits from external
  // (dictation / Assist) mutations of the shared `body()` signal.
  let lastText = '';
  const [activeState, setActiveState] = createSignal<EditorState | null>(null);
  const [,setBodyHtml]=createSignal(props.initialHtml??'');
  const [editorError,setEditorError]=createSignal('');
  const [imageLoads,setImageLoads]=createSignal(0);
  const [zoom,setZoom]=createSignal(100),[markers,setMarkers]=createSignal(false);
  const [linkOpen, setLinkOpen] = createSignal(false);
  const [linkUrl, setLinkUrl] = createSignal('');
  const has=(group:EditorGroup):boolean=>!(props.omitGroups??[]).includes(group);
  // The message background while writing (t39 L12): the user's appearance
  // preference, or a local value outside the app (tests). View only.
  const app=useContext(AppContext);
  const [localBackground,setLocalBackground]=createSignal<ComposeBackground>('theme');
  const background=():ComposeBackground=>app?app.appearancePrefs().composeBackground:localBackground();
  const appTone=():'dark'|'light'=>app?.appearance()==='dark'?'dark':'light';
  const darkSurface=():boolean=>surfaceTone(background(),appTone())==='dark';
  const toggleBackground=():void=>{const next=toggledBackground(background(),appTone());if(app)app.setAppearancePrefs({composeBackground:next});else setLocalBackground(next);};
  // The surface's painted background, read once per change (not per keystroke).
  let surfaceColor:string|null=null,surfaceText:string|null=null;
  const embedded=():boolean=>props.embedded!==undefined;
  function fontValue(name:string):string {
    const state=activeState();if(!state)return '';
    if(state.selection.empty)return String((state.storedMarks??state.selection.$from.marks()).find(mark=>mark.type.name===name)?.attrs['value']??'');
    let value:string|undefined;let mixed=false;
    state.doc.nodesBetween(state.selection.from,state.selection.to,node=>{if(!node.isText)return;const found=String(node.marks.find(mark=>mark.type.name===name)?.attrs['value']??'');if(value!==undefined&&value!==found)mixed=true;value=found;});
    return mixed?'':value??'';
  }
  function setTextStyle(name:string,value:string):void {if(!view)return;const mark=richSchema.marks[name];if(!mark)return;const {from,to,empty}=view.state.selection;let tr=view.state.tr;if(empty)tr=value?tr.addStoredMark(mark.create({value})):tr.removeStoredMark(mark);else{tr=tr.removeMark(from,to,mark);if(value)tr=tr.addMark(from,to,mark.create({value}));}view.dispatch(tr);view.focus();}

  const owned = (): ReadonlySet<string> | undefined => props.inline?.owned();
  // A switched-off Format feature leaves its shortcut inert, not just its button hidden.
  const stepOn=useComposeFeature('format.fontSizeStep'),indentOn=useComposeFeature('format.indent'),clearOn=useComposeFeature('format.clearFormatting');
  const gate=(on:()=>boolean,command:Command):Command=>(state,dispatch,v)=>on()?command(state,dispatch,v):false;
  const inList:Command=(state,dispatch,v)=>enclosingList(state)?sinkItem(state,dispatch,v):false;
  const outOfList:Command=(state,dispatch,v)=>enclosingList(state)?liftItem(state,dispatch,v):false;

  /** The editor shows an image from its local preview, not its `cid:`. */
  function imageView(node: PMNode): NodeView {
    const dom = document.createElement('img');
    dom.className = 'compose-rt__img';
    dom.alt = node.attrs['alt'] as string;
    const src = node.attrs['src'] as string | null;
    dom.src = src ?? props.inline?.preview(node.attrs['cid'] as string) ?? '';
    dom.dataset['cid'] = node.attrs['cid'] as string;
    const look = cleanImageLook(node.attrs);
    // Through the CSSOM: a style attribute is refused by the production CSP.
    for (const [property, value] of imageStyle(look)) dom.style.setProperty(property, value);
    if (look.title !== '') dom.title = look.title;
    dom.addEventListener('load', () => setImageLoads((n) => n + 1));
    return { dom };
  }

  /** Upload image files and place them at `pos` (default: the selection). */
  async function placeImages(files: File[], pos?: number): Promise<void> {
    if (files.length === 0) return;
    const placed: { cid?: string; src?: string; alt: string }[] = props.embedded !== undefined
      ? await embedImages(props.embedded, files)
      : props.inline !== undefined ? await props.inline.upload(files) : [];
    if (view === undefined || placed.length === 0) return;
    let tr = view.state.tr;
    for (const p of placed) {
      const node = nImage.create(p.src !== undefined ? { src: p.src, alt: p.alt } : { cid: p.cid, alt: p.alt });
      tr = pos !== undefined ? tr.insert(Math.min(pos, tr.doc.content.size), node) : tr.replaceSelectionWith(node);
      if (pos !== undefined) pos += node.nodeSize;
    }
    view.dispatch(tr);
    view.focus();
  }

  /** Read each raster file as a `data:` URL, while the host still has room for it. */
  async function embedImages(support: EmbeddedImageSupport, files: File[]): Promise<{ src: string; alt: string }[]> {
    const out: { src: string; alt: string }[] = [];
    let used = 0;
    for (const file of files) {
      if (!INLINE_IMAGE_TYPES.has(file.type)) { support.refused('type', file.name); continue; }
      const src = await new Promise<string>((resolve) => {
        const reader = new FileReader();
        reader.onload = () => resolve(typeof reader.result === 'string' ? reader.result : '');
        reader.onerror = () => resolve('');
        reader.readAsDataURL(file);
      });
      if (!EMBEDDED_IMAGE.test(src)) { support.refused('type', file.name); continue; }
      if (!support.fits(used + src.length)) { support.refused('size', file.name); continue; }
      used += src.length;
      out.push({ src, alt: file.name });
    }
    return out;
  }

  function emit(state: EditorState): void {
    const html = htmlFromDoc(state.doc);
    const text = textFromDoc(state.doc);
    lastText = text;
    setBodyHtml(html);
    setActiveState(state);
    props.onChange(html, text);
  }

  const updateDocumentRoot = () => view?.updateRoot();
  // A new background (the button, or the app theme under `theme`): read the
  // painted colour and let the view-colour plugin recompute.
  const repaintSurface=():void=>{
    if(view===undefined)return;
    const painted=getComputedStyle(host);
    surfaceColor=cssColorHex(painted.backgroundColor);
    surfaceText=cssColorHex(painted.color);
    view.dispatch(view.state.tr.setMeta(SURFACE_CHANGED,true));
  };
  createEffect(on(()=>[background(),app?.theme(),app?.appearance()],repaintSurface,{defer:true}));
  onMount(() => {
    host.addEventListener('mailwoman:documentchange', updateDocumentRoot);
    const state = EditorState.create({
      doc: docFromHtml(props.initialHtml, owned(), embedded()),
      plugins: [
        history(),
        tableLimitPlugin(()=>setEditorError(t('mail-editor-table-limits'))),
        keymap({
          'Mod-z': undo,
          'Mod-y': redo,
          'Mod-Shift-z': redo,
          'Mod-b': toggleMark(mStrong),
          'Mod-i': toggleMark(mEm),
          'Mod-u': toggleMark(mUnderline),
          'Mod-Alt-0': paragraphStyle('normal'),
          'Mod-Alt-1': paragraphStyle('heading1'),
          'Mod-Alt-2': paragraphStyle('heading2'),
          'Mod-Alt-3': paragraphStyle('heading3'),
          'Mod-Alt-4': paragraphStyle('heading4'),
          'Mod-Alt-q': paragraphStyle('quote'),
          'Mod-Alt-c': paragraphStyle('code'),
          Enter: splitListItem(nListItem),
          'Mod->': gate(stepOn,stepFontSizeCommand(1)),
          'Mod-<': gate(stepOn,stepFontSizeCommand(-1)),
          'Mod-]': gate(stepOn,stepFontSizeCommand(1,true)),
          'Mod-[': gate(stepOn,stepFontSizeCommand(-1,true)),
          'Mod-Space': gate(clearOn,clearFormatting('all')),
          'Mod-\\': gate(clearOn,clearFormatting('all')),
          Tab: chainCommands(gate(indentOn,inList),goToNextCell(1)),
          'Shift-Tab': chainCommands(gate(indentOn,outOfList),goToNextCell(-1)),
        }),
        keymap(baseKeymap),
        tableEditing(),
        viewColorPlugin(()=>surfaceColor,()=>surfaceText),
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
        const files = props.inline !== undefined || embedded() ? imageFiles(event.clipboardData) : [];
        if (files.length === 0) return false;
        event.preventDefault();
        void placeImages(files);
        return true;
      },
      handleDrop(v, event) {
        const files = props.inline !== undefined || embedded() ? imageFiles(event.dataTransfer) : [];
        if (files.length === 0) return false;
        event.preventDefault();
        const at = v.posAtCoords({ left: event.clientX, top: event.clientY });
        void placeImages(files, at?.pos);
        return true;
      },
    });
    emit(view.state);
    repaintSurface();
    // Sticky format painter: paint each new selection until Esc.
    const paintOnRelease=():void=>{if(painter.sticky()&&view&&!view.state.selection.empty)painter.apply();};
    view.dom.addEventListener('pointerup',paintOnRelease);
    view.dom.addEventListener('keyup',event=>{if((event as KeyboardEvent).shiftKey)paintOnRelease();});
    view.dom.addEventListener('keydown',event=>{if((event as KeyboardEvent).key==='Escape'&&painter.sticky()){painter.cancel();event.stopPropagation();}},true);

    const api:RichTextApi = {
      undo:()=>run(undo),redo:()=>run(redo),historyDepth:()=>{const state=activeState();return {undo:state?undoDepth(state):0,redo:state?redoDepth(state):0};},
      appendHtml(html: string): void {
        if (view === undefined) return;
        const frag = docFromHtml(html, owned(), embedded()).content;
        const next = view.state.apply(view.state.tr.insert(view.state.doc.content.size, frag));
        view.updateState(next);
        emit(next);
        view.focus();
      },
      insertHtml(html: string, inline: boolean): void {
        if (view === undefined) return;
        const frag = docFromHtml(html, owned(), embedded()).content;
        const slice = inline && frag.childCount === 1 && frag.firstChild?.isTextblock ? new Slice(frag, 1, 1) : new Slice(frag, 0, 0);
        const next = view.state.apply(view.state.tr.replaceSelection(slice).scrollIntoView());
        view.updateState(next);
        emit(next);
        view.focus();
      },
      replaceHtml(html:string):void {
        if(view===undefined)return;
        const next=view.state.apply(view.state.tr.replaceWith(0,view.state.doc.content.size,docFromHtml(html,owned(),embedded()).content));
        view.updateState(next);emit(next);view.focus();
      },
      setHtml(html: string): void {
        if (view === undefined) return;
        const next = EditorState.create({ doc: docFromHtml(html, owned(), embedded()), plugins: view.state.plugins });
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
    };
    setEditorApi(api);props.onReady?.(api);
  });

  onCleanup(() => { host.removeEventListener('mailwoman:documentchange', updateDocumentRoot); view?.destroy(); view=undefined; });

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
    command(view.state, tr => view?.dispatch(tr.docChanged && command !== undo && command !== redo ? closeHistory(tr) : tr), view);
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

  const tools=createAdvancedEditorTools({state:activeState,view:()=>view,run,owned,report:setEditorError,markers,onMarkers:()=>setMarkers(value=>!value)});
  const painter=createPainter<PaintedFormat>({capture:()=>{const s=view?.state;if(!s)return null;const p=capturePaint(s);return {value:p,summary:p.summary};},apply:value=>{if(!view)return false;run(applyPaint(value));return true;}});
  const attrsOf=():Record<string,unknown>=>{const s=activeState();return s?blockAttrs(s):{};};
  const fmtAdapter:FormatAdapter={
    ready:()=>activeState()!==null,
    fontStep:(direction,fine)=>execute(stepFontSizeCommand(direction,fine)),
    listType:()=>{const s=activeState();return s?currentListType(s):null;},
    applyList:(kind,style)=>execute(applyListStyle(kind,style)),
    listStart:value=>execute(setListStart(value)),
    canContinue:()=>{const s=activeState();return s!==null&&canContinueNumbering(s);},
    indent:direction=>execute(changeIndent(direction)),
    canOutdent:()=>{const s=activeState();return s!==null&&canOutdent(s);},
    clear:variant=>execute(clearFormatting(variant)),
    lineSpacing:value=>execute(setLineSpacing(value)),
    currentLineSpacing:()=>(attrsOf()['lineHeight'] as string|null)??null,
    spacing:(side,points)=>execute(setSpacing(side,points)),
    align:value=>execute(setAlignment(value)),
    currentAlign:()=>(attrsOf()['align'] as string|null)??null,
    direction:value=>execute(setDirection(value)),
    currentDirection:()=>(attrsOf()['dir'] as string|null)??null,
    paste:mode=>tools.pasteWith(mode),
    painter,
  };
  const fmt=createFormatControls(fmtAdapter);
  const markButton=(btn:ToolbarButton):JSX.Element=>{
    const active=()=>{const s=activeState();return s!==null&&btn.activeMark!==undefined?markActive(s,btn.activeMark):false;};
    return <button type="button" class={`compose-rt__btn ${a11y.focusable}`} classList={{'compose-rt__btn--active':active()}} aria-pressed={active()} aria-label={t(btn.labelKey)} title={t(btn.labelKey)} data-testid={`rt-${btn.id}`} onMouseDown={e=>e.preventDefault()} onClick={()=>run(btn.command)}><Icon name={btn.glyph}/></button>;
  };
  const row=(...controls:JSX.Element[]):JSX.Element=><div class="compose-rib__row">{controls}</div>;

  // Contextual tabs: the Picture tab while an image is selected, the Table layout tab while the caret is in a table.
  const context=createMemo<EditorContext>(()=>{const state=activeState();return {image:state!==null&&selectedImage(state)!==null,table:state!==null&&inTable(state)};},{image:false,table:false},{equals:(a,b)=>a.image===b.image&&a.table===b.table});
  createEffect(()=>props.onContext?.(context()));
  onCleanup(()=>props.onContext?.({image:false,table:false}));
  /** Run a command from a ribbon control, saying so when it does not apply here. */
  const execute=(command:Command):void=>{if(!view||!command(view.state)){setEditorError(t('mail-editor-selection-required'));return;}setEditorError('');run(command);};
  const naturalSize=createMemo(()=>{
    activeState();imageLoads();
    const state=view?.state;const found=state?selectedImage(state):null;
    const dom=found&&view?view.nodeDOM(found.pos):null;
    return dom instanceof HTMLImageElement&&dom.naturalWidth>0?{width:dom.naturalWidth,height:dom.naturalHeight}:null;
  });
  const imageIo:ImageIO={
    sourceUrl:node=>(node.attrs['src'] as string|null)??props.inline?.preview(node.attrs['cid'] as string)??null,
    render:(url,op)=>renderWithCanvas(url,op),
    store:async file=>{
      if(props.embedded!==undefined){const stored=await embedImages(props.embedded,[file]);return stored[0]?{src:stored[0].src}:null;}
      if(props.inline!==undefined){const stored=await props.inline.upload([file]);return stored[0]?{cid:stored[0].cid}:null;}
      return null;
    },
  };
  async function transformPicture(op:ImageOp):Promise<void> {
    if(!view)return;
    const result=await transformImage(view,op,imageIo);
    setEditorError(result==='done'?'':t(`mail-picture-failed-${result}`));
  }
  async function replacePicture(file:File):Promise<void> {
    if(!view)return;
    const stored=await imageIo.store(file);
    if(stored===null){setEditorError(t('mail-picture-failed-failed'));return;}
    execute(setImageSource(stored,true));
  }
  const pictureGroups=():JSX.Element=><PictureGroups state={activeState} execute={execute} naturalSize={naturalSize} transform={op=>void transformPicture(op)} replace={file=>void replacePicture(file)} canStore={props.inline!==undefined||props.embedded!==undefined}/>;

  // The Format tab: Clipboard, Font, Paragraph and Editing, each exactly three rows (t38 §2.2).
  const formatGroups=():JSX.Element=><>
    <Show when={has('clipboard')}><RibbonGroup label={t('mail-compose-group-clipboard')} priority={100} icon="paste" testId="rt-group-clipboard">
      <RibbonItem size="large">{fmt.pasteLarge()}</RibbonItem>
      <RibbonItem>{tools.cut()}</RibbonItem><RibbonItem>{tools.copy()}</RibbonItem>
    </RibbonGroup></Show>
    <Show when={has('clipboard')}><Feature id="format.painter"><RibbonGroup label={t('cfmt-group-painter')} priority={95} icon="brushCopy" testId="rt-group-painter">
      <RibbonItem>{fmt.copyFormat()}</RibbonItem><RibbonItem>{fmt.applyFormat()}</RibbonItem><RibbonItem>{fmt.formatChip()}</RibbonItem>
    </RibbonGroup></Feature></Show>
    <Show when={has('font')}><RibbonGroup label={t('mail-compose-font')} priority={90} icon="bold" testId="rt-group-font">
      <RibbonItem>{row(
        <select class="compose-rib__font" aria-label={t('mail-compose-font')} data-tooltip={t('mail-compose-font')} value={fontValue('fontFamily')} onChange={event=>setTextStyle('fontFamily',event.currentTarget.value)}><option value="">{t('mail-compose-font-inherit')}</option><For each={COMPOSER_FONTS}>{font=><option value={font.stack}>{font.label}</option>}</For></select>,
        <select class="compose-rib__size" aria-label={t('mail-compose-font-size')} data-tooltip={t('mail-compose-font-size')} value={fontValue('fontSize')} onChange={event=>setTextStyle('fontSize',event.currentTarget.value)}><option value="">{t('mail-compose-font-inherit')}</option><Show when={fontValue('fontSize')&&!COMPOSE_FONT_SIZES.some(size=>`${size}pt`===fontValue('fontSize'))}><option value={fontValue('fontSize')}>{fontValue('fontSize')}</option></Show><For each={COMPOSE_FONT_SIZES}>{size=><option value={`${size}pt`}>{size} pt</option>}</For></select>,
        fmt.fontSteps(),
      )}</RibbonItem>
      <RibbonItem>{row(<For each={MARK_BUTTONS}>{markButton}</For>,tools.changeCase(),fmt.clearFormatting())}</RibbonItem>
      <RibbonItem>{row(tools.textColor(),tools.clearColor(),tools.highlight(),tools.clearHighlight())}</RibbonItem>
    </RibbonGroup></Show>
    <Show when={has('paragraph')}><RibbonGroup label={t('mail-compose-group-paragraph')} priority={80} icon="paragraph" testId="rt-group-paragraph">
      <RibbonItem>{row(fmt.bulletList(),fmt.numberList(),fmt.indentButtons(),tools.sort())}</RibbonItem>
      <RibbonItem>{row(fmt.alignButtons(),fmt.lineSpacing(),fmt.direction())}</RibbonItem>
      <RibbonItem>{row(<For each={BLOCK_BUTTONS}>{markButton}</For>,tools.markers(),tools.paragraphOptions())}</RibbonItem>
    </RibbonGroup></Show>
    <Show when={has('paragraph')}><RibbonGroup label={t('cfmt-group-styles')} priority={70} icon="heading" testId="rt-group-styles">
      <RibbonItem>{tools.style()}</RibbonItem>
    </RibbonGroup></Show>
    <Show when={has('editing')}><RibbonGroup label={t('mail-compose-group-editing')} priority={45} icon="search" testId="rt-group-editing">
      <RibbonItem>{tools.findReplace()}</RibbonItem><RibbonItem>{tools.selectAll()}</RibbonItem><RibbonItem>{tools.clearSelection()}</RibbonItem>
    </RibbonGroup></Show>
    {/* View (t39 L12): how the body looks while writing; never part of what is sent. */}
    <Show when={has('view')}><RibbonGroup label={t('mail-compose-group-view')} priority={20} icon="eye" testId="rt-group-view">
      <RibbonItem><button type="button" class={`compose-rt__btn ${a11y.focusable}`} aria-pressed={darkSurface()} data-testid="rt-dark-background" data-tooltip={t('mail-compose-dark-background-help')} onMouseDown={e=>e.preventDefault()} onClick={toggleBackground}><Icon name="eye"/><span class="compose-rib__text">{t('mail-compose-dark-background')}</span></button></RibbonItem>
    </RibbonGroup></Show>
  </>;

  /** Put a horizontal rule at the selection. */
  const insertRule:Command=(state,dispatch)=>{dispatch?.(state.tr.replaceSelectionWith(nRule.create()).scrollIntoView());return true;};

  // The Insert tab: a link, an image (inline or embedded), a horizontal rule, and a new table.
  const insertGroups=():JSX.Element=><>
    <Show when={has('insert')}><RibbonGroup label={t('mail-compose-ribbon-insert')} priority={50} icon="link" testId="rt-group-insert">
      <RibbonItem>
        <RibbonPopover label={t('mail-compose-rt-link')} icon="link" keepSelection buttonClass={`compose-rt__btn ${a11y.focusable}`} buttonTestId="rt-link" open={linkOpen()} onOpenChange={setLinkOpen}>
          <div class="compose-rt__link" data-testid="rt-link-row">
            <input type="url" class={a11y.focusable} placeholder={t('mail-compose-rt-link-placeholder')} aria-label={t('mail-compose-rt-link')} value={linkUrl()} onInput={(e)=>setLinkUrl(e.currentTarget.value)} onKeyDown={(e)=>{if(e.key==='Enter'){e.preventDefault();applyLink();}}}/>
            <button type="button" class={`btn btn--ghost ${a11y.focusable}`} data-testid="rt-link-apply" onClick={()=>applyLink()}>{t('mail-compose-rt-link-apply')}</button>
          </div>
        </RibbonPopover>
      </RibbonItem>
      <Show when={props.inline !== undefined || embedded()}>
        <RibbonItem>
          <label class={`compose-rt__btn ${a11y.focusable}`} title={t('mail-compose-rt-image')} data-testid="rt-image">
            <span aria-hidden="true"><Icon name="image"/></span><span class="compose-rib__text" aria-hidden="true">{t('mail-compose-rt-image')}</span>
            <input type="file" accept="image/png,image/jpeg,image/gif,image/webp" multiple class={a11y.srOnly} aria-label={t('mail-compose-rt-image')} onChange={(e)=>{const input=e.currentTarget;const files=Array.from(input.files??[]);void placeImages(files).finally(()=>{input.value='';});}}/>
          </label>
        </RibbonItem>
      </Show>
      <RibbonItem>
        <button type="button" class={`compose-rt__btn ${a11y.focusable}`} data-testid="rt-rule" data-tooltip={t('mail-compose-rt-rule')} onMouseDown={e=>e.preventDefault()} onClick={()=>run(insertRule)}><Icon name="minus"/><span class="compose-rib__text">{t('mail-compose-rt-rule')}</span></button>
      </RibbonItem>
    </RibbonGroup></Show>
    <Show when={has('tableInsert')}><RibbonGroup label={t('mail-compose-ribbon-table')} priority={30} icon="table" testId="rt-group-table-insert">
      <RibbonItem>{tools.tableRows()}</RibbonItem><RibbonItem>{tools.tableColumns()}</RibbonItem><RibbonItem>{tools.insertTable()}</RibbonItem>
    </RibbonGroup></Show>
  </>;

  // The Table layout tab: rows, columns, cells, then the cell shading and the table itself.
  const tableGroups=():JSX.Element=><Show when={has('table')}><TableLayoutGroups state={activeState} execute={execute} shading={tools.tableShading} clearShading={tools.clearTableShading}/></Show>;

  const groupsFor=(tab:EditorTab):JSX.Element=>tab==='format'?formatGroups():tab==='insert'?insertGroups():tab==='picture'?pictureGroups():tableGroups();

  // Without a host ribbon, the editor shows the same `EditorRibbon` of its own (editor tabs only).
  const own=props.ribbonSlots===undefined?createEditorRibbon():undefined;
  const slots=():EditorSlots=>props.ribbonSlots?.()??own?.slots()??{};
  const ownRibbon=(state:NonNullable<typeof own>):JSX.Element=><EditorRibbon state={state} tabs={SLOT_TABS.map(id=>({id}))} visible={()=>context().image?SLOT_TABS:EDITOR_TABS} history={editorApi} showHistory={()=>!props.externalHistory}/>;
  return (
    <div class="compose-rt" data-testid="compose-rt" data-markers={markers()}>
      <Show when={own}>{state=><Show when={props.toolbarMount} fallback={ownRibbon(state())}>{mount=><Portal mount={mount()}>{ownRibbon(state())}</Portal>}</Show>}</Show>
      <For each={SLOT_TABS}>{tab=><Show when={slots()[tab]} keyed>{target=><InRibbonSlot target={target}>{groupsFor(tab)}</InRibbonSlot>}</Show>}</For>
      <div class="compose-rt__surface" data-zoom={zoom()} data-surface={background()} data-tone={darkSurface()?'dark':'light'} ref={host} />
      <Show when={editorError()}><p class="compose-rt__error" role="alert">{editorError()}</p></Show>
      <div class="compose-rt__footer" data-detached-local><label>{t('mail-editor-zoom')}<select aria-label={t('mail-editor-zoom')} value={zoom()} onChange={event=>setZoom(Number(event.currentTarget.value))}><For each={[75,100,125,150,200]}>{value=><option value={value}>{value}%</option>}</For></select></label></div>
    </div>
  );
}

export default RichTextEditor;
