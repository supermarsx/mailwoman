// Linked-files bar (t110; the download-through-server action of the privacy
// work): links in the message body that point to files (by extension, see
// `readerReferences` in api/remote-content.ts) can be downloaded through this
// server, so the file host never sees the reader's address. On the shared reader
// notice frame, third in the bar stack. "Show files" opens a menu listing each
// file (name and host); other body links can be downloaded the same way from the
// same menu. Sizes are not shown: the server learns a file's size only when it
// downloads it, and nothing is fetched before the user asks.

import { createEffect, createMemo, createSignal, on, type JSX } from 'solid-js';
import { signContent, signedSource } from '../api/remote-content.ts';
import { t } from '../i18n/index.ts';
import { toast } from './toast/toast.ts';
import { createContextMenu, type ContextAction } from './ContextMenu.tsx';
import { NoticeButton, ReaderNoticeBar, noticeCss } from './ReaderNoticeBar.tsx';

const isolate = (s: string): string => `\u2068${s}\u2069`;
/** Non-file links offered in the menu, at most. */
const OTHER_LINKS = 16;

export interface LinkedFile { url: string; name: string; host: string }

/** The display name and host of a file link: the last path segment, decoded. */
export function linkedFile(url: string): LinkedFile {
  try {
    const parsed = new URL(url);
    const segment = parsed.pathname.split('/').filter(Boolean).pop() ?? '';
    let name = segment;
    try { name = decodeURIComponent(segment); } catch { /* keep the raw segment */ }
    return { url, name: name || parsed.hostname, host: parsed.hostname };
  } catch {
    return { url, name: url, host: '' };
  }
}

/** Host and path of a link, without its query or fragment. */
function shortLink(url: string): string {
  try { const u = new URL(url); return `${u.hostname}${u.pathname === '/' ? '' : u.pathname}`; } catch { return url; }
}

export function LinkedFilesBar(props: { emailId: string; scope: string; urls: readonly string[]; links: readonly string[] }): JSX.Element {
  const [pending, setPending] = createSignal(false);
  createEffect(on(() => props.scope, () => setPending(false)));
  const files = createMemo(() => props.urls.map(linkedFile));
  const others = createMemo(() => props.links.filter((url) => !props.urls.includes(url)).slice(0, OTHER_LINKS).map(linkedFile));

  async function download(url: string): Promise<void> {
    const scope = props.scope;
    if (pending() || !props.links.includes(url)) return;
    setPending(true);
    try {
      const response = await signContent(props.emailId, { files: [url] });
      const path = response.files.find((file) => file.url === url)?.src;
      const src = path ? signedSource(path, 'f') : null;
      if (!src) throw new Error('File was refused');
      if (props.scope !== scope) return;
      const link = document.createElement('a'); link.href = src; link.download = ''; link.rel = 'noopener noreferrer'; link.referrerPolicy = 'no-referrer';
      document.body.append(link); link.click(); link.remove();
    } catch {
      if (props.scope === scope) toast.error(t('privacy-file-failed'));
    } finally {
      if (props.scope === scope) setPending(false);
    }
  }

  const headline = (): string => (files().length > 0 ? t('privacy-linked-files-count', { count: files().length }) : t('privacy-linked-links'));
  const hosts = createMemo(() => [...new Set(files().map((f) => f.host).filter(Boolean))]);
  const detail = (): string | null => (hosts().length > 0 ? hosts().slice(0, 3).map(isolate).join(', ') + (hosts().length > 3 ? ', …' : '') : null);
  const tooltip = (): string => {
    const lines = [[headline(), detail()].filter(Boolean).join(' · ')];
    for (const f of files().slice(0, 8)) lines.push(`${isolate(f.name)} · ${isolate(f.host)}`);
    if (files().length > 8) lines.push('…');
    lines.push(t(files().length > 0 ? 'privacy-linked-files-help' : 'privacy-linked-links-help'));
    return lines.join('\n');
  };

  const actions = createMemo<ContextAction[]>(() => {
    const items: ContextAction[] = files().map((f, i) => ({ id: `f${i}`, label: t('privacy-linked-file-item', { name: isolate(f.name), host: isolate(f.host) }), icon: 'download', disabled: pending(), run: () => download(f.url) }));
    others().forEach((f, i) => items.push({ id: `o${i}`, label: t('privacy-linked-other-item', { link: isolate(shortLink(f.url)) }), icon: 'link', disabled: pending(), separatorBefore: i === 0 && files().length > 0, run: () => download(f.url) }));
    return items;
  });
  const menu = createContextMenu({ label: () => t('privacy-linked-files'), scope: () => props.scope, actions, onError: () => toast.error(t('privacy-file-failed')) });

  return (
    <ReaderNoticeBar
      label={t('privacy-linked-files')}
      testid="linked-files-bar"
      state={files().length > 0 ? 'files' : 'links'}
      icon={files().length > 0 ? 'file' : 'link'}
      tooltip={tooltip()}
      actions={<NoticeButton icon="download" label={t(files().length > 0 ? 'privacy-linked-files-show' : 'privacy-linked-links-show')} haspopup="menu" expanded={menu.isOpen()} busy={pending()} disabled={actions().length === 0} testid="linked-files-show" onClick={(e) => menu.open(e)} onKeyDown={menu.keydown} />}
      after={menu.menu()}
    >
      <span class={noticeCss.headline}>{headline()}</span>
      {detail() !== null ? <span class={noticeCss.detail}>{' · '}{detail()}</span> : null}
    </ReaderNoticeBar>
  );
}
