import type {JSX} from 'solid-js';
import {isolate, t} from '../i18n';
import {Icon, type IconName} from './Icon.tsx';
import './readerBadges.css';

export type FolderBadgeKind = 'inbox' | 'junk' | 'trash' | 'other';

const ICONS: Record<string, IconName> = {inbox: 'inbox', junk: 'spam', trash: 'delete', sent: 'sent', archive: 'archive'};

/** The badge's look for a mailbox role (`null` for a user folder). */
export function folderBadgeKind(role: string | null | undefined): FolderBadgeKind {
  return role === 'inbox' ? 'inbox' : role === 'junk' ? 'junk' : role === 'trash' ? 'trash' : 'other';
}

/**
 * Where the open message lives, in the reader's badge family. A button that goes to the folder when
 * `onGo` is given, otherwise a plain status badge.
 */
export function FolderBadge(props: {names: string[]; role?: string | null | undefined; onGo?: (() => void) | undefined}): JSX.Element {
  const label = (): string => props.names.map(isolate).join(', ');
  const title = (): string => t(props.onGo ? 'mail-reader-folder-go' : 'mail-reader-folder-title', {folder: props.names.join(', ')});
  const body = (): JSX.Element => <><Icon name={ICONS[props.role ?? ''] ?? 'folder'}/><span class="mw-badge__label">{label()}</span></>;
  return props.onGo
    ? <button type="button" class="mw-badge mw-badge--folder" data-testid="reader-folder" data-role={folderBadgeKind(props.role)} data-tooltip={title()} aria-label={title()} onClick={() => props.onGo?.()}>{body()}</button>
    : <span class="mw-badge mw-badge--folder" data-testid="reader-folder" data-role={folderBadgeKind(props.role)} title={title()}>{body()}</span>;
}
