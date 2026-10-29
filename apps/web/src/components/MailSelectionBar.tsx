// The message list's selection bar (t58; the actions were the inline bulk bar of
// t28-e16). Every action applies to the whole selection in ONE request (see
// `mail.ts` `updateEmails`) and raises the shared undo toast, as before; the bar
// itself is the floating, non-modal SelectionBar.

import { createSignal, onCleanup, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';
import { downloadMessages, forwardMessages } from '../api/mailBulk.ts';
import { preferenceAccount, preferenceAccountIsCurrent } from '../api/accountPreferenceLifecycle.ts';
import { SelectionBar, type SelectionAction } from './SelectionBar.tsx';
import type { ContextAction } from './ContextMenu.tsx';
import type { ComposeInitial } from './compose/reply.ts';
import type { Id } from '../api/jmap-types.ts';

export function MailSelectionBar(props: {
  onCompose?: ((initial: ComposeInitial) => void) | undefined;
  onClear: () => void;
  returnFocus: () => void;
}): JSX.Element {
  const app = useApp();
  const [busy, setBusy] = createSignal(false);
  const controller = new AbortController();
  onCleanup(() => controller.abort());
  const targets = (): Id[] => app.actionTargets();
  const chosen = () => {
    const sel = app.selectedIds();
    return app.messages().filter((m) => sel.has(m.id));
  };
  const failed = (): void => app.showToast('error', t('mail-bulk-transfer-failed'));

  function forward(): void {
    try {
      const ids = new Set(targets());
      props.onCompose?.(forwardMessages(app.messages().filter((message) => ids.has(message.id)), t('mail-bulk-forward-subject', { count: ids.size })));
    } catch { failed(); }
  }
  async function download(): Promise<void> {
    if (busy()) return;
    const owner = preferenceAccount();
    if (!owner) return;
    setBusy(true);
    try {
      const blob = await downloadMessages(targets(), () => preferenceAccountIsCurrent(owner), controller.signal);
      if (!preferenceAccountIsCurrent(owner)) return;
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement('a');
      anchor.href = url;
      anchor.download = 'messages.mbox';
      document.body.append(anchor);
      anchor.click();
      anchor.remove();
      setTimeout(() => URL.revokeObjectURL(url), 10000);
    } catch {
      if (!controller.signal.aborted && preferenceAccountIsCurrent(owner)) failed();
    } finally { setBusy(false); }
  }

  const moveItems = (): ContextAction[] =>
    app.mailboxes()
      .filter((box) => box.id !== app.selectedMailboxId() && box.mailwomanSearchQuery === undefined)
      .map((box) => ({ id: `move:${box.id}`, label: box.name, icon: 'folder', run: () => app.moveMessage(targets(), box.id) }));
  const labelItems = (): ContextAction[] =>
    app.tags().map((tag) => {
      const on = chosen().length > 0 && chosen().every((m) => m.keywords?.[tag.id] === true);
      return { id: `tag:${tag.id}`, label: tag.name, icon: 'tag', checked: on, run: () => app.tagMessages(targets(), tag.id, !on, tag.name) };
    });

  const actions = (): SelectionAction[] => {
    const list = chosen();
    const anyUnread = list.some((m) => m.keywords?.['$seen'] !== true);
    const allFlagged = list.length > 0 && list.every((m) => m.keywords?.['$flagged'] === true);
    const out: SelectionAction[] = [
      { id: 'archive', label: t('mail-archive'), icon: 'archive', testId: 'bulk-archive', run: () => app.archiveMessage(targets()) },
      { id: 'delete', label: t('mail-delete'), icon: 'delete', testId: 'bulk-delete', run: () => app.trashMessage(targets()) },
      { id: 'move', label: t('selection-bar-move'), icon: 'folder', testId: 'bulk-move', menu: moveItems },
      anyUnread
        ? { id: 'read', label: t('mail-mark-read'), icon: 'mailRead', testId: 'bulk-read', run: () => app.markRead(targets(), true) }
        : { id: 'read', label: t('mail-mark-unread'), icon: 'mail', testId: 'bulk-read', run: () => app.markRead(targets(), false) },
      { id: 'flag', label: allFlagged ? t('mail-unflag-message') : t('mail-flag-message'), icon: 'flag', testId: 'bulk-flag', run: () => app.flagMessage(targets(), !allFlagged) },
      { id: 'spam', label: t('mail-mark-spam'), icon: 'spam', testId: 'bulk-spam', run: () => app.markSpam(targets()) },
    ];
    if (app.tags().length > 0) out.push({ id: 'labels', label: t('selection-bar-labels'), icon: 'tag', testId: 'bulk-labels', menu: labelItems });
    if (props.onCompose) out.push({ id: 'forward', label: t('mail-bulk-forward'), icon: 'forward', testId: 'bulk-forward', run: forward });
    out.push({ id: 'download', label: t('mail-bulk-download'), icon: 'download', testId: 'bulk-download', disabled: busy(), run: download });
    return out;
  };

  return (
    <SelectionBar
      count={app.selectedIds().size}
      total={app.listMessages().length}
      selectAll={() => app.selectAllInView()}
      clear={props.onClear}
      actions={actions()}
      returnFocus={props.returnFocus}
      scope={`${app.accountId()}:${app.selectedMailboxId()}`}
    />
  );
}
