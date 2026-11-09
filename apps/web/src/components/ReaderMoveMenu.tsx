// "Move to folder" in the reader action row: a folder button opens a menu of the
// folders this message can go to, the same list the message list's bulk Move uses.

import { onMount, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { loadCatalog, t } from '../i18n/index.ts';
import { ReaderAction } from './ReaderAction.tsx';
import { createContextMenu, type ContextAction } from './ContextMenu.tsx';
import { spamFolderHidden } from '../state/mailFeatures.ts';
import {ACTION_ICONS} from '../actionIcons.ts';
import type { Email } from '../api/jmap-types.ts';

/** The folders `email` can be moved to: any real folder it is not already in. */
export function moveActions(email: Email, app: ReturnType<typeof useApp>): ContextAction[] {
  return app.mailboxes()
    .filter((box) => email.mailboxIds?.[box.id] !== true && box.mailwomanSearchQuery === undefined && box.myRights?.mayAddItems!==false && !spamFolderHidden(box))
    .map((box) => ({ id: `move:${box.id}`, label: box.name, icon: 'folder', run: () => void app.moveMessage(email.id, box.id) }));
}

export function ReaderMoveMenu(props: { email: Email }): JSX.Element {
  const app = useApp();
  onMount(() => void loadCatalog('mail'));
  let button!: HTMLButtonElement;
  const menu = createContextMenu({
    label: () => t('mail-reader-move-to'),
    searchLabel:()=>t('mail-move-search'),
    scope: () => `${app.accountId()}:${props.email.id}`,
    restoreFocus: () => button?.focus(),
    actions: () => moveActions(props.email, app),
  });
  return (
    <>
      <ReaderAction icon={ACTION_ICONS.moveTo} label={t('mail-reader-move-to')} help={t('mail-action-help-move')} testId="reader-move" popup="menu" expanded={menu.isOpen()} onClick={(event) => { button = event.currentTarget as HTMLButtonElement; menu.open(event); }} />
      {menu.menu()}
    </>
  );
}
