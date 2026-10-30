// "Search related messages" in the reader action row: a magnifier opens a menu that starts (or, with a search active, refines) a
// search from this message's sender, subject or folder. The search runs in the
// Mail tab's list, so that tab is brought forward; the query and its chips land
// in the search box like any other search.

import { onMount, type JSX } from 'solid-js';
import { useApp } from '../../state/context.ts';
import { useRealtime } from '../../realtime/context.ts';
import { loadCatalog, t } from '../../i18n/index.ts';
import { ReaderAction } from '../ReaderAction.tsx';
import { createContextMenu, type ContextAction } from '../ContextMenu.tsx';
import { relatedSearchValue, relatedSenderDomain } from '../messageSearch.ts';
import type { Email } from '../../api/jmap-types.ts';

/** The rows for one message: sender, subject, folder (each only when known). */
export function relatedSearchActions(
  email: Email,
  mailboxIds: readonly string[],
  run: (field: 'from' | 'from-domain' | 'subject' | 'in', value: string) => void,
  domainSearch = false,
): ContextAction[] {
  const rows: ContextAction[] = [];
  const from = relatedSearchValue(email, 'from');
  if (from) rows.push({ id: 'search:from', label: t('search-refine-sender'), icon: 'users', run: () => run('from', from) });
  const domain=domainSearch?relatedSenderDomain(email):null;
  if(domain)rows.push({id:'search:from-domain',label:t('search-refine-domain'),icon:'domains',run:()=>run('from-domain',domain)});
  const subject = relatedSearchValue(email, 'subject');
  if (subject) rows.push({ id: 'search:subject', label: t('search-refine-subject'), icon: 'mail', run: () => run('subject', subject) });
  const folder = Object.keys(email.mailboxIds ?? {}).find((id) => email.mailboxIds[id] && mailboxIds.includes(id));
  if (folder) rows.push({ id: 'search:in', label: t('search-refine-folder'), icon: 'folder', run: () => run('in', folder) });
  return rows;
}

export function ReaderSearchMenu(props: { email: Email }): JSX.Element {
  const app = useApp();
  const realtime = useRealtime();
  onMount(() => void loadCatalog('search'));
  let button!: HTMLButtonElement;
  const menu = createContextMenu({
    label: () => t('search-refine-menu'),
    scope: () => `${app.accountId()}:${props.email.id}`,
    restoreFocus: () => button?.focus(),
    actions: () => relatedSearchActions(props.email, app.mailboxes().map((m) => m.id), (field, value) => {
      if (realtime.subTabs.tabs().some((tab) => tab.id === 'mail')) realtime.subTabs.activate('mail');
      void app.searchMessageField(field, value);
    },app.supportsFromDomainSearch()),
  });
  return (
    <>
      <ReaderAction icon="search" label={t('search-refine-menu')} help={t('search-refine-menu')} testId="reader-search-related" popup="menu" expanded={menu.isOpen()} onClick={event=>{button=event.currentTarget as HTMLButtonElement;menu.open(event);}} />
      {menu.menu()}
    </>
  );
}
