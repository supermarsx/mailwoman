import {AsyncPending} from '../../components/AsyncState.tsx';
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
import { createStore } from 'solid-js/store';
import { ContactEditor } from './ContactEditor.tsx';
import { GroupEditor } from './GroupEditor.tsx';
import { observePreferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { t, loadCatalog } from '../../i18n';
import { useApp } from '../../state/context.ts';
import { contactDisplayName } from '../../state/slices/contacts.ts';
import type { ContactCard } from '../../api/pim-types.ts';
import type { Id } from '../../api/jmap-types.ts';
import { parseVCards, toVCardDocument, type ParsedContact } from './vcard.ts';
import { contactsToCsv, csvToContacts, guessMapping, parseCsv, type CsvField, type CsvMapping } from './csv.ts';
import { findDuplicateClusters, mergeCards } from './merge.ts';
import { wireDialogFocus } from './dialogFocus.ts';
import { readCalendarFile } from '../calendar/decode.ts';
// V7 directory security (SPEC §13/§8.2, e14b): the per-contact cert/key rows, sourced
// from the GAL. Gated on a configured directory, so an unconfigured deployment's
// business card is unchanged.
import { ContactSecurity } from '../directory/index.ts';
import * as css from './contacts.css.ts';

const CSV_FIELDS: CsvField[] = [
  'ignore', 'fullName', 'given', 'surname', 'prefix', 'suffix',
  'nickname', 'organization', 'title', 'email', 'phone', 'birthday', 'notes',
];

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

function primaryEmail(card: ContactCard): string {
  return [...card.emails].sort((a, b) => (b.pref || 0) - (a.pref || 0))[0]?.value ?? '';
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

  const [panel, setPanel] = createSignal<'view' | 'edit' | 'create' | 'group'>('view');
  const [editingGroup,setEditingGroup] = createSignal<string|null>(null);
  onCleanup(observePreferenceAccount(() => setPanel('view')));
  const [showImport, setShowImport] = createSignal(false);
  const [showDuplicates, setShowDuplicates] = createSignal(false);

  const isAll = createMemo(
    () => app.selectedAddressBookId() === null && app.selectedGroupId() === null && !app.favoritesOnly(),
  );

  function openContact(id: Id): void {
    app.selectContact(id);
    setPanel('view');
  }

  return (
    <section aria-label={t('contacts-title')} data-module="contacts" class={panel() === 'view' ? css.layout : css.editorPage}>
        <Show when={panel() === 'group'}><GroupEditor books={app.addressBooks()} editing={editingGroup()!==null} initialName={app.contactGroups().find(group=>group.id===editingGroup())?.name??''} initialBook={app.contactGroups().find(group=>group.id===editingGroup())?.addressBookId??app.selectedAddressBookId()??app.addressBooks()[0]?.id??''} onSave={async(name,book)=>{const id=editingGroup();return id?(await app.updateGroup(id,{name})?id:null):app.createGroup(name,book);}} onSaved={()=>setPanel('view')} onCancel={()=>setPanel('view')}/></Show>
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
        onImport={() => setShowImport(true)}
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
          <button type="button" class={css.buttonGhost} onClick={() => setShowDuplicates(true)}>
            {t('contacts-find-duplicates')}
          </button>
        </div>

        <Show
          when={app.filteredContacts().length > 0}
          fallback={<Show when={app.contactsLoading()} fallback={<p class={css.empty}>{t('contacts-none')}</p>}><AsyncPending variant="list" message={t('contacts-loading')}/></Show>}
        >
          <ul class={css.contactList} aria-label={t('contacts-list')}>
            <For each={app.filteredContacts()}>
              {(card) => (
                <li>
                  <div
                    class={css.contactRow}
                    role="button"
                    tabindex={0}
                    aria-current={app.selectedContactId() === card.id}
                    onClick={() => openContact(card.id)}
                    onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); openContact(card.id); } }}
                  >
                    <button
                      type="button"
                      class={css.star}
                      aria-label={t('contacts-favorite', { name: contactDisplayName(card) })}
                      aria-pressed={card.isFavorite}
                      onClick={(e) => { e.stopPropagation(); void app.toggleFavorite(card.id); }}
                    >
                      <Icon name={card.isFavorite ? 'star' : 'starOutline'} />
                    </button>
                    <span class={css.rowBody}>
                      <span class={css.rowName}><PersonAvatar name={contactDisplayName(card)} address={primaryEmail(card)}/><bdi>{contactDisplayName(card)}</bdi></span>
                      <span class={css.rowMeta}><bdi>{primaryEmail(card) || card.organizations[0] || ''}</bdi></span>
                    </span>
                  </div>
                </li>
              )}
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

      <div class={css.detail}>
        <Show when={panel() === 'view' && app.selectedContact() !== null}>
          <BusinessCard card={app.selectedContact()!} onEdit={() => setPanel('edit')} />
        </Show>
        <Show when={panel() === 'view' && app.selectedContact() === null}>
          <p class={css.empty}>{t('contacts-select-hint')}</p>
        </Show>
      </div>

      </Show>
      <Show when={showImport()}>
        <ImportDialog onClose={() => setShowImport(false)} />
      </Show>
      <Show when={showDuplicates()}>
        <DuplicatesDialog onClose={() => setShowDuplicates(false)} />
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
        <span class={css.count}>{app.contacts().filter((c) => c.isFavorite).length}</span>
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
        <button type="button" class={css.buttonGhost} aria-label={t('contacts-new-group')} onClick={props.onNewGroup}><Icon name="plus" /></button>
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

function BusinessCard(props: { card: ContactCard; onEdit: () => void }): JSX.Element {
  const app = useApp();
  const card = () => props.card;
  const groupsOf = createMemo(() =>
    app.contactGroups().filter((g) => g.memberIds.includes(card().id) || card().groupIds.includes(g.id)),
  );

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
            onClick={() => void app.toggleFavorite(card().id)}
          >
            <Icon name={card().isFavorite ? 'star' : 'starOutline'} /> {card().isFavorite ? t('contacts-favorited') : t('contacts-favorite-action')}
          </button>
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

      <Show when={groupsOf().length > 0}>
        <div class={css.fieldGroup}>
          <span class={css.fieldLabel}>{t('contacts-groups-label')}</span>
          <div class={css.fieldRow}><For each={groupsOf()}>{(g) => <span class={css.chip}><bdi>{g.name}</bdi></span>}</For></div>
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

      <GroupMembership card={card()} />
    </article>
  );
}

/** Group membership checkboxes on the business card (group management). */
function GroupMembership(props: { card: ContactCard }): JSX.Element {
  const app = useApp();
  return (
    <Show when={app.contactGroups().length > 0}>
      <fieldset class={`${css.fieldGroup} ${css.bareFieldset}`}>
        <legend class={css.fieldLabel}>{t('contacts-member-of')}</legend>
        <For each={app.contactGroups()}>
          {(group) => {
            const isMember = (): boolean =>
              group.memberIds.includes(props.card.id) || props.card.groupIds.includes(group.id);
            return (
              <label class={css.fieldRow}>
                <input
                  type="checkbox"
                  aria-label={t('contacts-group-membership', { name: group.name })}
                  checked={isMember()}
                  onChange={(e) => void app.setGroupMembership(props.card.id, group.id, e.currentTarget.checked)}
                />
                <span><bdi>{group.name}</bdi></span>
              </label>
            );
          }}
        </For>
      </fieldset>
    </Show>
  );
}

// ── Editor (create / edit) ───────────────────────────────────────────────────

// ── Import dialog (vCard / CSV with preview + field mapping) ──────────────────

function ImportDialog(props: { onClose: () => void }): JSX.Element {
  const app = useApp();
  const [text, setText] = createSignal('');
  const [format, setFormat] = createSignal<'vcard' | 'csv'>('vcard');
  const [parsedCsv, setParsedCsv] = createSignal<ReturnType<typeof parseCsv> | null>(null);
  const [mapping, setMapping] = createStore<{ cols: CsvMapping }>({ cols: [] });
  const [vcards, setVcards] = createSignal<ParsedContact[] | null>(null);
  const [imported, setImported] = createSignal<number | null>(null);

  function detectFormat(value: string): 'vcard' | 'csv' {
    return value.trim().toUpperCase().startsWith('BEGIN:VCARD') ? 'vcard' : 'csv';
  }

  function preview(): void {
    const fmt = detectFormat(text());
    setFormat(fmt);
    if (fmt === 'vcard') {
      setParsedCsv(null);
      setVcards(parseVCards(text()));
    } else {
      const pc = parseCsv(text());
      setParsedCsv(pc);
      setMapping('cols', guessMapping(pc.headers));
      setVcards(null);
    }
  }

  const csvPreview = createMemo<ParsedContact[]>(() => {
    const pc = parsedCsv();
    if (pc === null) return [];
    return csvToContacts(pc, mapping.cols);
  });

  const previewCards = (): ParsedContact[] => (format() === 'vcard' ? (vcards() ?? []) : csvPreview());

  async function commit(): Promise<void> {
    const n = await app.importContacts(previewCards());
    setImported(n);
  }

  async function onFile(e: Event & { currentTarget: HTMLInputElement }): Promise<void> {
    const file = e.currentTarget.files?.[0];
    if (file === undefined) return;
    // `file.text()` is a UTF-8 decode, so Firefox turns a UTF-16 vCard or CSV
    // into text that parses to nothing; `readCalendarFile` goes by the byte
    // order mark, as the calendar import does.
    const content = await readCalendarFile(file);
    setText(content);
    preview();
  }

  let dialogRef!: HTMLDivElement;
  onMount(() => {
    const dispose = wireDialogFocus(dialogRef, props.onClose);
    onCleanup(dispose);
  });

  return (
    <div class={css.dialogBackdrop} onClick={props.onClose}>
      <div ref={dialogRef} class={css.dialog} role="dialog" tabindex={-1} aria-modal="true" aria-label={t('contacts-import-title')} onClick={(e) => e.stopPropagation()}>
        <h1 class={css.cardName}>{t('contacts-import-title')}</h1>
        <Show when={imported() === null} fallback={
          <>
            <p role="status">{t('contacts-imported', { count: imported() ?? 0 })}</p>
            <div class={css.actions}><button type="button" class={css.button} onClick={props.onClose}>{t('common-done')}</button></div>
          </>
        }>
          <p class={css.cardSub}>{t('contacts-import-hint')}</p>
          <input type="file" aria-label={t('contacts-import-file')} accept=".vcf,.csv,text/vcard,text/csv" onChange={onFile} />
          <textarea
            class={css.input}
            aria-label={t('contacts-paste')}
            rows={6}
            value={text()}
            onInput={(e) => setText(e.currentTarget.value)}
          />
          <div class={css.actions}>
            <button type="button" class={css.buttonGhost} onClick={props.onClose}>{t('common-cancel')}</button>
            <button type="button" class={css.button} onClick={preview}>{t('contacts-preview')}</button>
          </div>

          <Show when={format() === 'csv' && parsedCsv() !== null}>
            <div class={css.fieldGroup}>
              <span class={css.fieldLabel}>{t('contacts-map-columns')}</span>
              <table class={css.table}>
                <thead>
                  <tr><th class={css.th}>{t('contacts-col')}</th><th class={css.th}>{t('contacts-maps-to')}</th><th class={css.th}>{t('contacts-sample')}</th></tr>
                </thead>
                <tbody>
                  <For each={parsedCsv()!.headers}>
                    {(header, i) => (
                      <tr>
                        <td class={css.td}><bdi>{header}</bdi></td>
                        <td class={css.td}>
                          <select
                            class={css.select}
                            aria-label={t('contacts-map-column', { header })}
                            value={mapping.cols[i()] ?? 'ignore'}
                            onChange={(e) => setMapping('cols', i(), e.currentTarget.value as CsvField)}
                          >
                            <For each={CSV_FIELDS}>{(f) => <option value={f}>{f}</option>}</For>
                          </select>
                        </td>
                        <td class={css.td}><bdi>{parsedCsv()!.rows[0]?.[i()] ?? ''}</bdi></td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </Show>

          <Show when={previewCards().length > 0}>
            <div class={css.fieldGroup}>
              <span class={css.fieldLabel}>{t('contacts-preview-count', { count: previewCards().length })}</span>
              <ul class={css.contactList} aria-label={t('contacts-import-preview')}>
                <For each={previewCards()}>
                  {(c) => <li class={css.rowMeta}><bdi>{c.name.full || t('contacts-no-name')}</bdi> — <bdi>{c.emails[0]?.value ?? t('contacts-no-email')}</bdi></li>}
                </For>
              </ul>
            </div>
            <div class={css.actions}>
              <button type="button" class={css.button} onClick={commit}>
                {t('contacts-import-n', { count: previewCards().length })}
              </button>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}

// ── Merge-duplicates dialog ──────────────────────────────────────────────────

function DuplicatesDialog(props: { onClose: () => void }): JSX.Element {
  const app = useApp();
  const clusters = createMemo(() => findDuplicateClusters(app.contacts()));
  const [review, setReview] = createSignal<{ keepId: Id; mergeIds: Id[] } | null>(null);

  const merged = createMemo<ContactCard | null>(() => {
    const r = review();
    if (r === null) return null;
    const primary = app.contactById(r.keepId);
    if (primary === undefined) return null;
    const others = r.mergeIds.map((id) => app.contactById(id)).filter((c): c is ContactCard => c !== undefined);
    return mergeCards(primary, others);
  });

  async function confirm(): Promise<void> {
    const r = review();
    if (r === null) return;
    await app.mergeContacts(r.keepId, r.mergeIds);
    setReview(null);
  }

  let dialogRef!: HTMLDivElement;
  onMount(() => {
    const dispose = wireDialogFocus(dialogRef, props.onClose);
    onCleanup(dispose);
  });

  return (
    <div class={css.dialogBackdrop} onClick={props.onClose}>
      <div ref={dialogRef} class={css.dialog} role="dialog" tabindex={-1} aria-modal="true" aria-label={t('contacts-merge-title')} onClick={(e) => e.stopPropagation()}>
        <h1 class={css.cardName}>{t('contacts-merge-title')}</h1>

        <Show when={review() === null} fallback={
          <>
            <p class={css.cardSub}>{t('contacts-merge-review-hint')}</p>
            <Show when={merged() !== null}>
              <article class={css.card} aria-label={t('contacts-merged-preview')}>
                <h2 class={css.cardName}><bdi>{contactDisplayName(merged()!)}</bdi></h2>
                <div class={css.fieldGroup}>
                  <span class={css.fieldLabel}>{t('contacts-emails')}</span>
                  <For each={merged()!.emails}>{(e) => <span><bdi>{e.value}</bdi></span>}</For>
                </div>
                <Show when={merged()!.phones.length > 0}>
                  <div class={css.fieldGroup}>
                    <span class={css.fieldLabel}>{t('contacts-phones')}</span>
                    <For each={merged()!.phones}>{(p) => <span><bdi>{p.value}</bdi></span>}</For>
                  </div>
                </Show>
              </article>
            </Show>
            <div class={css.actions}>
              <button type="button" class={css.buttonGhost} onClick={() => setReview(null)}>{t('common-back')}</button>
              <button type="button" class={css.button} onClick={confirm}>{t('contacts-merge-confirm')}</button>
            </div>
          </>
        }>
          <Show when={clusters().length > 0} fallback={<p class={css.empty}>{t('contacts-no-duplicates')}</p>}>
            <For each={clusters()}>
              {(cluster) => (
                <div class={css.card}>
                  <span class={css.fieldLabel}>{t('contacts-possible-duplicates', { count: cluster.length })}</span>
                  <ul class={css.contactList}>
                    <For each={cluster}>
                      {(c) => <li class={css.rowMeta}><bdi>{contactDisplayName(c)}</bdi> — <bdi>{primaryEmail(c) || t('contacts-no-email')}</bdi></li>}
                    </For>
                  </ul>
                  <div class={css.actions}>
                    <button
                      type="button"
                      class={css.button}
                      onClick={() => setReview({ keepId: cluster[0]!.id, mergeIds: cluster.slice(1).map((c) => c.id) })}
                    >
                      {t('contacts-review-merge')}
                    </button>
                  </div>
                </div>
              )}
            </For>
          </Show>
          <div class={css.actions}>
            <button type="button" class={css.buttonGhost} onClick={props.onClose}>{t('common-close')}</button>
          </div>
        </Show>
      </div>
    </div>
  );
}
