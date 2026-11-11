// Drop files, folders, images and messages anywhere on the composer. A full-window
// overlay says what the drop will do (counts, limit, Shift for inline images);
// blocked types and oversized files are named with their reason. Touch devices
// and reduced-motion users are not shown anything animated; the overlay has none.

import { createSignal, For, onCleanup, onMount, Show, type Accessor, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import type { AttachmentPolicy } from '../../../api/attachmentPolicy.ts';
import { t } from '../../../i18n/index.ts';
import { Spinner } from '../../spinner/Spinner.tsx';
import { peekMessageDrag, takeMessageDrag } from '../../messageDrag.ts';
import { readAttachmentDrag, type AttachmentDragToken } from '../../attachmentDrag.ts';
import { dragCarriesContent, flattenEntries, planFiles, summarizeDrag, type DropPlan, type DropSummary, type FsEntry } from './dropPolicy.ts';
import { dropCard, dropOverlay } from './insertDialog.css.ts';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('compose-insert');

export interface DropHost {
  /** The element whose window is the drop target (the composer). */
  root: Accessor<HTMLElement | undefined>;
  attachAllowed: Accessor<boolean>;
  inlineAllowed: Accessor<boolean>;
  messagesAllowed: Accessor<boolean>;
  maxBytes: Accessor<number>;
  policy: Accessor<AttachmentPolicy | null>;
  accountId: Accessor<string | null>;
  attachFiles(files: File[]): Promise<void>;
  /** `at` is an editor position computed from the drop point, when there is one. */
  placeImages(files: File[], at?: number | null): Promise<void>;
  /** The editor position under a screen point (rich editor), or null. */
  dropPoint?(x: number, y: number): number | null;
  /** A reader attachment dragged in, fetched through the policy-checked download. */
  fetchAttachment(token: AttachmentDragToken, signal: AbortSignal): Promise<File>;
  /** A message as `.eml`, by id. */
  fetchMessage(id: string, signal: AbortSignal): Promise<File>;
  report(message: string): void;
}

const isTouch = (): boolean => typeof matchMedia === 'function' && matchMedia('(pointer: coarse) and (hover: none)').matches;
const mb = (n: number): string => (n / (1024 * 1024)).toFixed(n >= 10 * 1024 * 1024 ? 0 : 1);

export function ComposeDropZone(props: { host: DropHost }): JSX.Element {
  const host = props.host;
  const [active, setActive] = createSignal(false);
  const [summary, setSummary] = createSignal<DropSummary>({ files: 0, images: 0, folders: 0, messages: 0, bytes: 0 });
  const [shift, setShift] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [problems, setProblems] = createSignal<{ name: string; reason: string }[]>([]);
  let depth = 0;
  let controller: AbortController | null = null;

  const accepts = (event: DragEvent): boolean => {
    if (isTouch() || !dragCarriesContent(event.dataTransfer)) return false;
    const s = summarizeDrag(event.dataTransfer);
    const messagesOk = host.messagesAllowed() && (s.messages > 0 ? peekMessageDrag(event, host.accountId()) !== null : false);
    return (s.files > 0 && host.attachAllowed()) || messagesOk || (s.files > 0 && host.inlineAllowed());
  };
  const end = (): void => { depth = 0; setActive(false); };
  const reason = (r: string): string => t(`ins-drop-reason-${r}`);

  async function collect(event: DragEvent): Promise<File[]> {
    const dt = event.dataTransfer;
    if (!dt) return [];
    const entries: FsEntry[] = [];
    let folder = false;
    for (const item of Array.from(dt.items ?? [])) {
      const entry = (item as DataTransferItem & { webkitGetAsEntry?: () => FsEntry | null }).webkitGetAsEntry?.();
      if (entry) { entries.push(entry); if (entry.isDirectory) folder = true; }
    }
    if (!folder) return Array.from(dt.files);
    const flat = await flattenEntries(entries);
    if (flat.truncated) host.report(t('ins-drop-folder-truncated'));
    return flat.files;
  }

  async function onDrop(event: DragEvent): Promise<void> {
    const wantsInline = event.shiftKey || !host.attachAllowed();
    // Read everything that vanishes with the event now: the drop point and the tokens.
    const point = host.dropPoint?.(event.clientX, event.clientY) ?? null;
    const token = host.attachAllowed() ? readAttachmentDrag(event.dataTransfer) : null;
    const messageIds = host.messagesAllowed() ? takeMessageDrag(event, host.accountId()) : null;
    setBusy(true);
    setActive(true);
    controller = new AbortController();
    const bad: { name: string; reason: string }[] = [];
    try {
      const files = await collect(event);
      if (controller.signal.aborted) return;
      if (files.length > 0) {
        const plan: DropPlan = planFiles(files, { maxBytes: host.maxBytes(), policy: host.policy(), inlineImages: host.inlineAllowed() && wantsInline, attachAllowed: host.attachAllowed() });
        for (const r of plan.refused) bad.push({ name: r.name, reason: reason(r.reason) });
        if (plan.warned.length > 0) host.report(t('ins-drop-warned', { names: plan.warned.join(', ') }));
        setSummary({ files: plan.attach.length + plan.inline.length, images: plan.inline.length, folders: 0, messages: messageIds?.length ?? 0, bytes: 0 });
        if (plan.inline.length > 0) await host.placeImages(plan.inline, point);
        if (plan.attach.length > 0) await host.attachFiles(plan.attach);
      }
      if (token !== null) {
        if (token.accountId !== host.accountId()) bad.push({ name: token.name, reason: reason('account') });
        else {
          try {
            const file = await host.fetchAttachment(token, controller.signal);
            const plan = planFiles([file], { maxBytes: host.maxBytes(), policy: host.policy(), inlineImages: false, attachAllowed: host.attachAllowed() });
            for (const x of plan.refused) bad.push({ name: x.name, reason: reason(x.reason) });
            if (plan.attach.length > 0) await host.attachFiles(plan.attach);
          } catch { if (!controller.signal.aborted) bad.push({ name: token.name, reason: reason('message') }); }
        }
      }
      for (const id of messageIds ?? []) {
        if (controller.signal.aborted) break;
        try {
          const eml = await host.fetchMessage(id, controller.signal);
          if (eml.size > host.maxBytes()) bad.push({ name: eml.name, reason: reason('size') });
          else await host.attachFiles([eml]);
        } catch { if (!controller.signal.aborted) bad.push({ name: id, reason: reason('message') }); }
      }
    } catch {
      if (!controller.signal.aborted) host.report(t('ins-drop-folder-truncated'));
    } finally {
      setBusy(false);
      controller = null;
      setProblems(bad);
      if (bad.length === 0) end();
      else setTimeout(() => { if (!busy()) end(); }, 6000);
    }
  }

  onMount(() => {
    const doc = (host.root() ?? document.body).ownerDocument;
    const enter = (event: DragEvent): void => {
      if (!accepts(event)) return;
      event.preventDefault();
      depth++;
      setSummary(summarizeDrag(event.dataTransfer));
      setShift(event.shiftKey);
      setProblems([]);
      setActive(true);
    };
    const over = (event: DragEvent): void => {
      if (!active() && !accepts(event)) return;
      event.preventDefault();
      if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
      setShift(event.shiftKey);
    };
    const leave = (event: DragEvent): void => {
      if (!active() || busy()) return;
      depth = Math.max(0, depth - 1);
      if (depth === 0 || event.relatedTarget === null) end();
    };
    const drop = (event: DragEvent): void => {
      const accepted=accepts(event);
      if (!accepted && !dragCarriesContent(event.dataTransfer)) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      if (accepted && !busy()) void onDrop(event);
      else if (!busy()) end();
    };
    const root = host.root() ?? doc.body;
    // Capture on the document so a drop works over the editor, the ribbon and the header alike.
    const inside = (event: DragEvent, fn: (e: DragEvent) => void): void => { if (root.contains(event.target as Node) || event.target === doc.documentElement) fn(event); };
    const hEnter = (e: DragEvent): void => inside(e, enter);
    const hOver = (e: DragEvent): void => inside(e, over);
    const hLeave = (e: DragEvent): void => inside(e, leave);
    const hDrop = (e: DragEvent): void => inside(e, drop);
    const editorDocuments = new Map<HTMLIFrameElement, Document>();
    const editorListeners = [['dragenter', enter], ['dragover', over], ['dragleave', leave], ['drop', drop], ['dragend', end]] as const;
    function bindEditor(frame: HTMLIFrameElement): void {
      const inner = frame.contentDocument;
      const previous = editorDocuments.get(frame);
      if (previous === inner) return;
      if (previous) for (const [type, handler] of editorListeners) previous.removeEventListener(type, handler as EventListener, true);
      editorDocuments.delete(frame);
      if (inner?.documentElement?.getAttribute('data-mw-native-document') !== 'true') return;
      editorDocuments.set(frame, inner);
      for (const [type, handler] of editorListeners) inner.addEventListener(type, handler as EventListener, true);
    }
    const frameLoaded = (event: Event): void => {
      if (event.target instanceof HTMLIFrameElement) bindEditor(event.target);
    };
    root.addEventListener('load', frameLoaded, true);
    root.querySelectorAll('iframe').forEach(bindEditor);
    doc.addEventListener('dragenter', hEnter, true);
    doc.addEventListener('dragover', hOver, true);
    doc.addEventListener('dragleave', hLeave, true);
    doc.addEventListener('drop', hDrop, true);
    doc.addEventListener('dragend', end, true);
    onCleanup(() => {
      root.removeEventListener('load', frameLoaded, true);
      for (const inner of editorDocuments.values()) {
        for (const [type, handler] of editorListeners) inner.removeEventListener(type, handler as EventListener, true);
      }
      doc.removeEventListener('dragenter', hEnter, true);
      doc.removeEventListener('dragover', hOver, true);
      doc.removeEventListener('dragleave', hLeave, true);
      doc.removeEventListener('drop', hDrop, true);
      doc.removeEventListener('dragend', end, true);
      controller?.abort();
    });
  });
  const parts = (): string => {
    const s = summary();
    const out: string[] = [];
    if (s.files > 0) out.push(t('ins-drop-files', { count: s.files }));
    if (s.images > 0 && host.inlineAllowed()) out.push(t('ins-drop-images', { count: s.images }));
    if (s.messages > 0) out.push(t('ins-drop-messages'));
    return out.join(' · ');
  };
  return (
    <Show when={active()}>
      <Portal mount={(host.root() ?? document.body).ownerDocument.body}>
        <div class={dropOverlay} data-testid="compose-drop-overlay" role="status" aria-live="polite">
          <div class={dropCard}>
            <Show when={busy()} fallback={<>
              <p><strong>{host.inlineAllowed() ? t('ins-drop-title') : t('ins-drop-title-attach')}</strong></p>
              <p data-testid="compose-drop-counts">{parts()}</p>
              <p class="mw-drop-note">{t('ins-drop-limit', { max: mb(host.maxBytes()) })}{shift() && host.inlineAllowed() ? ` · ${t('ins-drop-inline-on')}` : ''}</p>
              <For each={problems()}>{(p) => <p class="mw-drop-bad" data-testid="compose-drop-problem">{p.name}: {p.reason}</p>}</For>
            </>}>
              <Spinner label={t('ins-drop-working')} />
              <p>{t('ins-drop-working')}</p>
              <button type="button" class="btn btn--ghost" onClick={() => controller?.abort()}>{t('ins-cancel')}</button>
            </Show>
          </div>
        </div>
      </Portal>
    </Show>
  );
}
