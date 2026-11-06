// Insert link (Ctrl+K): caption, address, optional title, new tab, a safe-link
// check. The same dialog edits an existing link (the host passes `existing`).

import { createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n/index.ts';
import { InsertDialog } from './InsertDialog.tsx';
import type { LinkSpecEdit } from './linkEdit.ts';
import { checkLink, linkHtml, linkText, normalizeHref, urlFromClipboard } from './links.ts';

export interface LinkInitial {
  caption: string;
  href: string;
  title: string;
  newTab: boolean;
  existing: boolean;
  /** Called when the user chooses "Remove link" on an existing link. */
  onRemove?: () => void;
  /** Called instead of inserting, when the host edits the link in place. */
  onApply?: (spec: LinkSpecEdit) => void;
}

export function LinkDialog(props: { initial: LinkInitial; richMode: boolean; returnTo: HTMLElement | null; onClose: () => void; onInsert: (html: string, text: string) => void }): JSX.Element {
  const clip = (): string => '';
  const [caption, setCaption] = createSignal(props.initial.caption);
  const [href, setHref] = createSignal(props.initial.href || clip());
  const [title, setTitle] = createSignal(props.initial.title);
  const [newTab, setNewTab] = createSignal(props.initial.newTab);
  const normalized = createMemo(() => normalizeHref(href()));
  const warnings = createMemo(() => checkLink(normalized()));
  const blocked = (): boolean => warnings().includes('script');
  // A URL on the clipboard prefills the address and, with no selection, the caption.
  void (typeof navigator !== 'undefined' && navigator.clipboard?.readText
    ? navigator.clipboard.readText().then((text) => {
        const url = urlFromClipboard(text);
        if (url && href() === '') { setHref(url); }
      }).catch(() => undefined)
    : undefined);
  const spec = (): { caption: string; href: string; title: string; newTab: boolean } => ({ caption: caption(), href: normalized(), title: title(), newTab: newTab() });
  const apply = (): void => {
    const s = spec();
    if (s.href === '' || blocked()) return;
    const html = linkHtml({ caption: s.caption, href: s.href, ...(s.title ? { title: s.title } : {}), newTab: s.newTab });
    if (props.initial.onApply) { props.initial.onApply({ caption: s.caption.trim() === '' ? s.href : s.caption, href: s.href, title: s.title, newTab: s.newTab }); props.onClose(); return; }
    props.onInsert(html, linkText({ caption: s.caption, href: s.href }));
  };
  return (
    <InsertDialog title={t(props.initial.existing ? 'ins-link-edit-title' : 'ins-link-title')} testId="insert-link-dialog" returnTo={props.returnTo} onClose={props.onClose} onSubmit={apply} submitLabel={t(props.initial.existing ? 'ins-link-save' : 'ins-insert')} submitDisabled={normalized() === '' || blocked()}>
      <label>{t('ins-link-caption')}<input data-testid="insert-link-caption" value={caption()} onInput={(e) => setCaption(e.currentTarget.value)} /></label>
      <label>{t('ins-link-address')}<input data-testid="insert-link-href" type="text" inputmode="url" placeholder="https://, mailto:, tel:, #anchor" value={href()} onInput={(e) => setHref(e.currentTarget.value)} /></label>
      <Show when={props.richMode}>
        <label>{t('ins-link-tooltip')}<input data-testid="insert-link-title" value={title()} onInput={(e) => setTitle(e.currentTarget.value)} /></label>
        <label class="mw-insert-check"><input type="checkbox" data-testid="insert-link-newtab" checked={newTab()} onChange={(e) => setNewTab(e.currentTarget.checked)} />{t('ins-link-newtab')}</label>
      </Show>
      <Show when={!props.richMode}><p class="mw-insert-note">{t('ins-link-plain-note')}</p></Show>
      <div role="status" aria-live="polite">
        <For each={warnings()}>{(w) => <p class="mw-insert-warn" data-testid={`insert-link-warn-${w}`}>{t(`ins-link-warn-${w}`)}</p>}</For>
      </div>
      <Show when={props.initial.existing && props.initial.onRemove}>
        <button type="button" class="btn btn--ghost" data-testid="insert-link-remove" onClick={() => { props.initial.onRemove?.(); props.onClose(); }}>{t('ins-link-remove')}</button>
      </Show>
    </InsertDialog>
  );
}
