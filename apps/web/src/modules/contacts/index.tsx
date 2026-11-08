import { InlineLoading } from '../../components/InlineLoading.tsx';
import {PersonAvatar} from '../../components/PersonAvatar.tsx';
import { Icon } from '../../components/Icon.tsx';
// Contacts module (plan §2.5, §3 e7). Address books, the contact list
// (favorites + groups / distribution lists), a business-card detail + edit view,
// merge-duplicates, vCard/CSV import (with a preview + CSV field mapping) and
// export, group management, the favorite toggle, and the per-contact opaque key
// placeholder field. Mock-backed via the store's contacts slice until e10 swaps
// in the real engine; the recipient autocomplete hook lives in `autocomplete.ts`
// (e10 wires it into Compose — this module does not edit Compose).

import { For, Show, createMemo, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { createContextMenu, type ContextAction } from '../../components/ContextMenu.tsx';
import { contactComposeInitial } from '../../components/AddressContactCard.tsx';
import { requestCompose } from '../../state/pendingCompose.ts';
import { ContactShareMenu, createShareActions } from './ContactShare.tsx';
import { ContactEditor } from './ContactEditor.tsx';
import { GroupEditor } from './GroupEditor.tsx';
import { observePreferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { t, loadCatalog } from '../../i18n';
import { useApp } from '../../state/context.ts';
import type { AppState } from '../../state/store.ts';
import { contactDisplayName } from '../../state/slices/contacts.ts';
import type { ContactCard } from '../../api/pim-types.ts';
import type { Id } from '../../api/jmap-types.ts';
import { toVCardDocument } from './vcard.ts';
import { contactsToCsv } from './csv.ts';
import { ImportPage, MembershipPage, MembershipSummary, MergePage, primaryEmail } from './pages.tsx';
// V7 directory security (SPEC §13/§8.2, e14b): the per-contact cert/key rows, sourced
// from the GAL. Gated on a configured directory, so an unconfigured deployment's
// business card is unchanged.
import { ContactSecurity } from '../directory/index.ts';
import * as css from './contacts.css.ts';

/** Trigger a client-side file download of `text` as `filename`. */
function download(filename: string, text: string, mime: string): void {
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

/** Toggle a favorite; a refused write has already been rolled back, so only say so. */
async function toggleFavorite(app: AppState, id: Id): Promise<void> {
  if (!(await app.toggleFavorite(id))) app.showToast('error', t('contacts-favorite-failed'));
}

export function ContactsModule(): JSX.Element {
  const app = useApp();
  onMount(() => {
    void loadCatalog('contacts');
    void app.loadContacts();
    // Probe the directory once (silent) so the per-contact Security tab appears only
    // when a GAL is configured; unconfigured ⇒ the business card is unchanged.
    void app.directory.ensureEnabled();
  });

  const [panel, setPanel] = createSignal<'view' | 'edit' | 'create' | 'group' | 'membership' | 'import' | 'merge'>('view');
  const [membershipGroup, setMembershipGroup] = createSignal<Id | null>(null);
  const back = (): void => { setPanel('view'); };
  function manageMembership(groupId: Id | null): void { setMembershipGroup(groupId); setPanel('membership'); }
  const [editingGroup,setEditingGroup] = createSignal<string|null>(null);
  onCleanup(observePreferenceAccount(() => setPanel('view')));

  const isAll = createMemo(
    () => app.selectedAddressBookId() === null && app.selectedGroupId() === null && !app.favoritesOnly(),
  );

  // Rows are keyed by id, so a favorite toggle keeps the row (and its focus) instead of rebuilding it.
  const rowIds = createMemo(() => app.filteredContacts().map((card) => card.id));

  // ── selection + bulk favorites ──
  const [selected, setSelected] = createSignal<ReadonlySet<Id>>(new Set<Id>());
  const selectedVisible = createMemo(() => app.filteredContacts().filter((card) => selected().has(card.id)));
  function toggleSelected(id: Id, on: boolean): void {
    setSelected((current) => { const next = new Set(current); if (on) next.add(id); else next.delete(id); return next; });
  }
  async function bulkFavorite(value: boolean): Promise<void> {
    const ids = selectedVisible().map((card) => card.id);
    const pending = ids.filter((id) => app.contactById(id)?.isFavorite !== value);
    const done = await app.setFavorite(ids, value);
    if (done.length > 0) app.showToast('success', t(value ? 'contacts-bulk-favorites-added' : 'contacts-bulk-favorites-removed', { count: done.length }));
    if (done.length < pending.length) app.showToast('error', t('contacts-favorite-failed'));
    setSelected(new Set<Id>());
  }

  // ── row context menu: favorite, send email, share ──
  const [menuCard, setMenuCard] = createSignal<ContactCard | null>(null);
  const rowMenu = createContextMenu({
    label: () => t('contacts-row-menu', { name: menuCard() ? contactDisplayName(menuCard()!) : '' }),
    scope: () => JSON.stringify([app.accountId(), menuCard()?.id ?? '']),
    actions: () => {
      const card = menuCard();
      if (!card) return [];
      const live = app.contactById(card.id) ?? card;
      const actions = createShareActions(app, () => app.contactById(card.id) ?? card, () => document, requestCompose);
      const address = primaryEmail(live);
      const draft = address ? contactComposeInitial(address, live.name.full) : null;
      return [
        { id: 'favorite', label: t(live.isFavorite ? 'contacts-favorite-remove' : 'contacts-favorite-add'), icon: live.isFavorite ? 'starOutline' : 'star', run: () => { void toggleFavorite(app, live.id); } },
        { id: 'email', label: t('contacts-address-compose'), icon: 'mail', disabled: !draft, run: () => { if (draft) requestCompose(draft); } },
        { id: 'share', label: t('contacts-share'), icon: 'share', separatorBefore: true, run: () => undefined, children: [
          { id: 'share-email', label: t('contacts-share-email'), icon: 'mail', run: () => actions.email() },
          { id: 'share-copy', label: t('contacts-share-copy'), icon: 'copy', run: () => actions.copy() },
          { id: 'share-download', label: t('contacts-share-download'), icon: 'download', run: () => actions.download() },
          ...(actions.nativeAvailable() ? [{ id: 'share-native', label: t('contacts-share-native'), icon: 'share', run: () => actions.native() } satisfies ContextAction] : []),
        ] },
      ] satisfies ContextAction[];
    },
  });

  function openContact(id: Id): void {
    app.selectContact(id);
    setPanel('view');
  }

  return (
    <section aria-label={t('contacts-title')} data-module="contacts" class={panel() === 'view' ? css.layout : panel() === 'membership' || panel() === 'import' || panel() === 'merge' ? css.pageHost : css.editorPage}>
        <Show when={panel() === 'group'}><GroupEditor books={app.addressBooks()} editing={editingGroup()!==null} initialName={app.contactGroups().find(group=>group.id===editingGroup())?.name??''} initialBook={app.contactGroups().find(group=>group.id===editingGroup())?.addressBookId??app.selectedAddressBookId()??app.addressBooks()[0]?.id??''} onSave={async(name,book)=>{const id=editingGroup();return id?(await app.updateGroup(id,{name})?id:null):app.createGroup(name,book);}} onSaved={()=>setPanel('view')} onCancel={()=>setPanel('view')}/></Show>
        {/* t39 R6: membership, import and merge are pages of their own (pages.tsx). */}
        <Show when={panel() === 'membership'}><MembershipPage initialGroup={membershipGroup()} onBack={back} onNewGroup={() => { setEditingGroup(null); setPanel('group'); }} /></Show>
        <Show when={panel() === 'import'}><ImportPage onBack={back} /></Show>
        <Show when={panel() === 'merge'}><MergePage onBack={back} /></Show>
        <Show when={panel() === 'create'}>
          <ContactEditor
            mode="create"
            onCancel={() => setPanel('view')}
            onSaved={(id) => { if (typeof id === 'string') app.selectContact(id); setPanel('view'); }}
            onSave={async (draft) => {
              const id = await app.createContact(draft);
              return id;
            }}
          />
        </Show>
        <Show when={panel() === 'edit' && app.selectedContact() !== null}>
          <ContactEditor
            mode="edit"
            source={app.selectedContact()!}
            onCancel={() => setPanel('view')}
            onSaved={(id) => { if (typeof id === 'string') app.selectContact(id); setPanel('view'); }}
            onSave={async (draft) => {
              return await app.updateContact(app.selectedContact()!.id, draft);
            }}
          />
        </Show>

      <Show when={panel() === 'view'}>
      <Sidebar
        isAll={isAll()}
        onNewGroup={() => {setEditingGroup(null);setPanel('group');}}
        onEditGroup={id=>{setEditingGroup(id);setPanel('group');}}
        onImport={() => setPanel('import')}
        onMembership={() => manageMembership(app.selectedGroupId())}
      />

      <div class={css.listPane}>
        <div class={css.toolbar}>
          <input
            type="search"
            class={css.input}
            aria-label={t('contacts-search')}
            placeholder={t('contacts-search')}
            value={app.contactSearch()}
            onInput={(e) => app.setContactSearch(e.currentTarget.value)}
          />
          <button type="button" class={css.button} onClick={() => { app.selectContact(null); setPanel('create'); }}>
            {t('contacts-new')}
          </button>
          <button type="button" class={css.buttonGhost} onClick={() => setPanel('merge')}>
            {t('contacts-find-duplicates')}
          </button>
        </div>

        <Show when={selectedVisible().length > 0}>
          <div class={css.bulkBar} role="group" aria-label={t('contacts-bulk-actions')}>
            <span role="status">{t('contacts-selected-count', { count: selectedVisible().length })}</span>
            <button type="button" class={css.buttonGhost} onClick={() => void bulkFavorite(true)}><Icon name="star" /> {t('contacts-favorite-add')}</button>
            <button type="button" class={css.buttonGhost} onClick={() => void bulkFavorite(false)}><Icon name="starOutline" /> {t('contacts-favorite-remove')}</button>
            <button type="button" class={css.buttonGhost} onClick={() => setSelected(new Set<Id>())}>{t('contacts-select-clear')}</button>
          </div>
        </Show>

        <Show
          when={app.filteredContacts().length > 0}
          fallback={<Show when={app.contactsLoading()} fallback={<p class={css.empty}>{app.favoritesOnly() && app.contactSearch().trim() === '' ? t('contacts-favorites-empty') : t('contacts-none')}</p>}><InlineLoading message={t('contacts-loading')}/></Show>}
        >
          <ul class={css.contactList} aria-label={t('contacts-list')}>
            <For each={rowIds()}>
              {(id) => { const card = (): ContactCard => app.contactById(id)!; return (
                <li>
                  <div
                    class={css.contactRow}
                    role="button"
                    tabindex={0}
                    aria-current={app.selectedContactId() === card().id}
                    onClick={() => openContact(card().id)}
                    onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); openContact(card().id); } else { setMenuCard(card()); rowMenu.keydown(e); } }}
                    onContextMenu={(e) => { setMenuCard(card()); rowMenu.open(e); }}
                  >
                    <input
                      type="checkbox"
                      class={css.rowCheck}
                      aria-label={t('contacts-select-row', { name: contactDisplayName(card()) })}
                      checked={selected().has(card().id)}
                      onClick={(e) => e.stopPropagation()}
                      onKeyDown={(e) => e.stopPropagation()}
                      onChange={(e) => toggleSelected(card().id, e.currentTarget.checked)}
                    />
                    <button
                      type="button"
                      class={css.star}
                      aria-label={t('contacts-favorite', { name: contactDisplayName(card()) })}
                      aria-pressed={card().isFavorite}
                      title={t(card().isFavorite ? 'contacts-favorite-remove' : 'contacts-favorite-add')}
                      onClick={(e) => { e.stopPropagation(); void toggleFavorite(app, card().id); }}
                    >
                      <Icon name={card().isFavorite ? 'star' : 'starOutline'} />
                    </button>
                    <span class={css.rowBody}>
                      <span class={css.rowName}><PersonAvatar name={contactDisplayName(card())} address={primaryEmail(card())}/><bdi>{contactDisplayName(card())}</bdi></span>
                      <span class={css.rowMeta}><bdi>{primaryEmail(card()) || card().organizations[0] || ''}</bdi></span>
                    </span>
                  </div>
                </li>
              ); }}
            </For>
          </ul>
        </Show>

        <div class={css.toolbar}>
          <button
            type="button"
            class={css.buttonGhost}
            onClick={() => download('contacts.vcf', toVCardDocument(app.filteredContacts()), 'text/vcard')}
          >
            {t('contacts-export-vcard')}
          </button>
          <button
            type="button"
            class={css.buttonGhost}
            onClick={() => download('contacts.csv', contactsToCsv(app.filteredContacts()), 'text/csv')}
          >
            {t('contacts-export-csv')}
          </button>
        </div>
      </div>
      {rowMenu.menu()}

      <div class={css.detail}>
        <Show when={panel() === 'view' && app.selectedContact() !== null}>
          <BusinessCard card={app.selectedContact()!} onEdit={() => setPanel('edit')} onManageMembership={manageMembership} />
        </Show>
        <Show when={panel() === 'view' && app.selectedContact() === null}>
          <p class={css.empty}>{t('contacts-select-hint')}</p>
        </Show>
      </div>

      </Show>
    </section>
  );
}

// ── Sidebar (address books + groups + import) ────────────────────────────────

function Sidebar(props: {
  isAll: boolean;
  onNewGroup: () => void;
  onEditGroup: (id:string) => void;
  onImport: () => void;
  onMembership: () => void;
}): JSX.Element {
  const app = useApp();

  const memberCount = (groupId: Id): number => {
    const g = app.contactGroups().find((x) => x.id === groupId);
    return g?.memberIds.length ?? 0;
  };

  return (
    <aside class={css.sidebar} aria-label={t('contacts-books-groups')}>
      <h2 class={css.heading}>{t('contacts-address-books')}</h2>
      <button
        type="button"
        class={css.navButton}
        aria-current={props.isAll}
        onClick={() => { app.selectAddressBook(null); app.setSelectedGroup(null); app.setFavoritesOnly(false); }}
      >
        <span>{t('contacts-all')}</span>
        <span class={css.count}>{app.contacts().length}</span>
      </button>
      <button
        type="button"
        class={css.navButton}
        aria-current={app.favoritesOnly()}
        onClick={() => { app.setFavoritesOnly(!app.favoritesOnly()); app.setSelectedGroup(null); }}
      >
        <span><Icon name="star" /> {t('contacts-favorites')}</span>
        <span class={css.count}>{app.favoriteCount()}</span>
      </button>
      <For each={app.addressBooks()}>
        {(book) => (
          <button
            type="button"
            class={css.navButton}
            aria-current={app.selectedAddressBookId() === book.id}
            onClick={() => { app.selectAddressBook(book.id); app.setSelectedGroup(null); app.setFavoritesOnly(false); }}
          >
            <span><bdi>{book.name}</bdi></span>
          </button>
        )}
      </For>

      <h2 class={css.heading}>
        <span>{t('contacts-groups')}</span>
        <span class={css.headingActions}>
          {/* t39 R6: the membership page sits beside New group, so the sidebar gains no row. */}
          <button type="button" class={css.buttonGhost} aria-label={t('contacts-membership-title')} title={t('contacts-membership-title')} onClick={props.onMembership}><Icon name="users" /></button>
          <button type="button" class={css.buttonGhost} aria-label={t('contacts-new-group')} onClick={props.onNewGroup}><Icon name="plus" /></button>
        </span>
      </h2>
      <For each={app.contactGroups()}>
        {(group) => (
          <div class={css.groupNavigation}>
          <button
            type="button"
            class={css.navButton}
            aria-current={app.selectedGroupId() === group.id}
            onClick={() => { app.setSelectedGroup(app.selectedGroupId() === group.id ? null : group.id); app.setFavoritesOnly(false); }}
          >
            <span><bdi>{group.name}</bdi></span>
            <span class={css.count}>{memberCount(group.id)}</span>
          </button>
          <button type="button" class={css.buttonGhost} aria-label={t('contacts-edit-named-group',{name:group.name})} onClick={()=>props.onEditGroup(group.id)}><Icon name="edit"/></button>
          </div>
        )}
      </For>

      <h2 class={css.heading}>{t('contacts-data')}</h2>
      <button type="button" class={css.navButton} onClick={props.onImport}>{t('contacts-import')}</button>
    </aside>
  );
}

// ── Business card (read view) ────────────────────────────────────────────────

function BusinessCard(props: { card: ContactCard; onEdit: () => void; onManageMembership: (groupId: Id | null) => void }): JSX.Element {
  const app = useApp();
  const card = () => props.card;

  return (
    <article class={css.card} aria-label={t('contacts-card', { name: contactDisplayName(card()) })}>
      <div class={`${css.fieldRow} ${css.cardHeader}`}>
        <div>
          <h1 class={css.cardName}><PersonAvatar name={contactDisplayName(card())} address={primaryEmail(card())}/><bdi>{contactDisplayName(card())}</bdi></h1>
          <Show when={card().titles.length > 0 || card().organizations.length > 0}>
            <p class={css.cardSub}><bdi>{[card().titles[0], card().organizations[0]].filter(Boolean).join(' · ')}</bdi></p>
          </Show>
        </div>
        <div class={css.actions}>
          <button
            type="button"
            class={css.buttonGhost}
            aria-label={t('contacts-favorite', { name: contactDisplayName(card()) })}
            aria-pressed={card().isFavorite}
            title={t(card().isFavorite ? 'contacts-favorite-remove' : 'contacts-favorite-add')}
            onClick={() => void toggleFavorite(app, card().id)}
          >
            <Icon name={card().isFavorite ? 'star' : 'starOutline'} /> {card().isFavorite ? t('contacts-favorited') : t('contacts-favorite-action')}
          </button>
          <ContactShareMenu card={card} triggerClass={css.buttonGhost} label onCompose={requestCompose} />
          <button type="button" class={css.button} onClick={props.onEdit}>{t('common-edit')}</button>
          <button type="button" class={css.buttonGhost} onClick={() => void app.deleteContact(card().id)}>{t('common-delete')}</button>
        </div>
      </div>

      <Show when={card().emails.length > 0}>
        <div class={css.fieldGroup}>
          <span class={css.fieldLabel}>{t('contacts-email')}</span>
          <For each={card().emails}>
            {(e) => <div class={css.fieldRow}><a href={`mailto:${e.value}`}><bdi>{e.value}</bdi></a> <Show when={e.context}><span class={css.chip}><bdi>{e.context}</bdi></span></Show></div>}
          </For>
        </div>
      </Show>

      <Show when={card().phones.length > 0}>
        <div class={css.fieldGroup}>
          <span class={css.fieldLabel}>{t('contacts-phone')}</span>
          <For each={card().phones}>
            {(p) => <div class={css.fieldRow}><span><bdi>{p.value}</bdi></span> <Show when={p.context}><span class={css.chip}><bdi>{p.context}</bdi></span></Show></div>}
          </For>
        </div>
      </Show>

      <Show when={card().anniversaries.length > 0}>
        <div class={css.fieldGroup}>
          <span class={css.fieldLabel}>{t('contacts-dates')}</span>
          <For each={card().anniversaries}>{(a) => <div class={css.fieldRow}><span>{a.kind}</span><span>{a.date}</span></div>}</For>
        </div>
      </Show>

      <Show when={card().notes.length > 0}>
        <div class={css.fieldGroup}>
          <span class={css.fieldLabel}>{t('contacts-notes')}</span>
          <p class={css.notesText}><bdi>{card().notes}</bdi></p>
        </div>
      </Show>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>{t('contacts-security-key')}</span>
        <p class={css.cardSub}>
          {card().pgpKey !== null || card().smimeCert !== null
            ? t('contacts-key-on-file')
            : t('contacts-no-key')}
        </p>
      </div>

      {/* V7 directory-published security material (SPEC §13/§8.2): photo + S/MIME
          certificate rows for this contact's address. Mounted only when a directory
          is configured, so the card is unchanged for non-directory deployments. */}
      <Show when={app.directory.enabled() && primaryEmail(card()).length > 0}>
        <div class={css.fieldGroup}>
          <span class={css.fieldLabel}>{t('contacts-directory-security')}</span>
          <ContactSecurity email={primaryEmail(card())} service={app.directory.service} />
        </div>
      </Show>

      <MembershipSummary card={card()} onManage={props.onManageMembership} />
    </article>
  );
}
