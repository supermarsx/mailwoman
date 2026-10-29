// Contacts management pages (t39 R6). Group membership, import and merge each
// get a page of their own in the Contacts module, laid out list-detail: the
// list pane picks a group / a source / a duplicate set, the detail pane edits
// it. They replace the business card's membership checkboxes and the import
// and merge modals. On a narrow screen the panes stack.
import { createEffect, createMemo, createSignal, createUniqueId, For, onMount, Show, type JSX } from 'solid-js';
import { createStore } from 'solid-js/store';
import { t } from '../../i18n';
import { useApp } from '../../state/context.ts';
import { contactDisplayName } from '../../state/slices/contacts.ts';
import type { ContactCard } from '../../api/pim-types.ts';
import type { Id } from '../../api/jmap-types.ts';
import { MultiSelect } from '../../components/MultiSelect.tsx';
import { Icon } from '../../components/Icon.tsx';
import { parseVCards, type ParsedContact } from './vcard.ts';
import { csvToContacts, guessMapping, parseCsv, type CsvField, type CsvMapping } from './csv.ts';
import { findDuplicateClusters, mergeCards } from './merge.ts';
import { readCalendarFile } from '../calendar/decode.ts';
import * as css from './contacts.css.ts';

export type ContactsPage = 'membership' | 'import' | 'merge';

const CSV_FIELDS: CsvField[] = [
  'ignore', 'fullName', 'given', 'surname', 'prefix', 'suffix',
  'nickname', 'organization', 'title', 'email', 'phone', 'birthday', 'notes',
];

export function primaryEmail(card: ContactCard): string {
  return [...card.emails].sort((a, b) => (b.pref || 0) - (a.pref || 0))[0]?.value ?? '';
}

/** The page frame: a Back button, the page heading, then list and detail. */
function Page(props: { title: string; help: string; listLabel: string; onBack: () => void; list: JSX.Element; detail: JSX.Element; testId: string }): JSX.Element {
  const id = createUniqueId();
  let heading!: HTMLHeadingElement;
  // Moving to a page puts focus on its heading, as a route change does.
  onMount(() => heading.focus());
  return <section class={css.page} aria-labelledby={`${id}-title`} data-testid={props.testId}>
    <header class={css.pageHeader}>
      <button type="button" class={css.buttonGhost} onClick={props.onBack}><Icon name="back" /> {t('contacts-back-to-list')}</button>
      <h1 ref={heading} id={`${id}-title`} class={css.cardName} tabindex={-1}>{props.title}</h1>
      <p class={css.cardSub}>{props.help}</p>
    </header>
    <div class={css.pageBody}>
      <nav class={css.pageList} aria-label={props.listLabel}>{props.list}</nav>
      <div class={css.pageDetail}>{props.detail}</div>
    </div>
  </section>;
}

// ── Group membership ─────────────────────────────────────────────────────────

export function MembershipPage(props: { onBack: () => void; initialGroup?: Id | null; onNewGroup: () => void }): JSX.Element {
  const app = useApp();
  const [groupId, setGroupId] = createSignal<Id | null>(props.initialGroup ?? null);
  createEffect(() => { const groups = app.contactGroups(); if (!groups.some(g => g.id === groupId())) setGroupId(groups[0]?.id ?? null); });
  const group = createMemo(() => app.contactGroups().find(g => g.id === groupId()));
  const members = createMemo(() => { const g = group(); if (!g) return []; return app.contacts().filter(c => g.memberIds.includes(c.id) || c.groupIds.includes(g.id)).map(c => c.id); });
  const options = createMemo(() => app.contacts().map(c => ({ value: c.id, label: contactDisplayName(c) || primaryEmail(c) || t('contacts-no-name'), detail: primaryEmail(c) })));
  const [saving, setSaving] = createSignal(false);
  const [failed, setFailed] = createSignal(false);
  async function change(next: string[]): Promise<void> {
    const g = group(); if (!g) return;
    const before = new Set(members()); const after = new Set(next);
    setSaving(true); setFailed(false);
    try {
      for (const id of next) if (!before.has(id)) await app.setGroupMembership(id, g.id, true);
      for (const id of before) if (!after.has(id)) await app.setGroupMembership(id, g.id, false);
      const stored = new Set(app.contactGroups().find(x => x.id === g.id)?.memberIds ?? []);
      if (next.some(id => !stored.has(id)) || [...before].some(id => !after.has(id) && stored.has(id))) setFailed(true);
    } catch { setFailed(true); } finally { setSaving(false); }
  }
  const labelId = createUniqueId();
  return <Page testId="contacts-membership-page" title={t('contacts-membership-title')} help={t('contacts-membership-help')} listLabel={t('contacts-groups')} onBack={props.onBack}
    list={<>
      <Show when={app.contactGroups().length > 0} fallback={<p class={css.empty}>{t('contacts-membership-no-groups')}</p>}>
        <ul class={css.pageItems}>
          <For each={app.contactGroups()}>{g => <li><button type="button" class={css.navButton} aria-current={groupId() === g.id} onClick={() => setGroupId(g.id)}><span><bdi>{g.name}</bdi></span><span class={css.count}>{g.memberIds.length}</span></button></li>}</For>
        </ul>
      </Show>
      <button type="button" class={css.buttonGhost} onClick={props.onNewGroup}><Icon name="plus" /> {t('contacts-new-group')}</button>
    </>}
    detail={<Show when={group()} fallback={<p class={css.empty}>{t('contacts-membership-pick')}</p>}>{g => <article class={css.card} aria-label={t('contacts-membership-of', { name: g().name })}>
      <h2 class={css.pageSubheading}><bdi>{g().name}</bdi></h2>
      <div class={css.fieldGroup}>
        <span id={labelId} class={css.fieldLabel}>{t('contacts-membership-members')}</span>
        <MultiSelect labelledBy={labelId} label={t('contacts-membership-members')} options={options()} selected={members()} disabled={saving()} placeholder={t('contacts-membership-add')} onChange={values => void change(values)} testId="contacts-membership-picker" />
        <p class={css.cardSub}>{t('contacts-membership-count', { count: members().length })}</p>
      </div>
      <Show when={failed()}><p role="alert">{t('contacts-membership-failed')}</p></Show>
    </article>}</Show>}
  />;
}

/** The business card's group membership: chips plus a link to the page. */
export function MembershipSummary(props: { card: ContactCard; onManage: (groupId: Id | null) => void }): JSX.Element {
  const app = useApp();
  const groups = createMemo(() => app.contactGroups().filter(g => g.memberIds.includes(props.card.id) || props.card.groupIds.includes(g.id)));
  const labelId = createUniqueId();
  const [saving, setSaving] = createSignal(false);
  async function change(next: string[]): Promise<void> {
    const before = new Set(groups().map(g => g.id)); const after = new Set(next);
    setSaving(true);
    try {
      for (const id of next) if (!before.has(id)) await app.setGroupMembership(props.card.id, id, true);
      for (const id of before) if (!after.has(id)) await app.setGroupMembership(props.card.id, id, false);
    } finally { setSaving(false); }
  }
  return <Show when={app.contactGroups().length > 0}>
    <div class={css.fieldGroup}>
      <span id={labelId} class={css.fieldLabel}>{t('contacts-member-of')}</span>
      <MultiSelect labelledBy={labelId} label={t('contacts-member-of')} options={app.contactGroups().map(g => ({ value: g.id, label: g.name }))} selected={groups().map(g => g.id)} disabled={saving()} placeholder={t('contacts-membership-add-group')} onChange={values => void change(values)} testId="contacts-card-groups" />
      <div class={css.actions}><button type="button" class={css.buttonGhost} onClick={() => props.onManage(groups()[0]?.id ?? null)}>{t('contacts-membership-manage')}</button></div>
    </div>
  </Show>;
}

// ── Import ───────────────────────────────────────────────────────────────────

export function ImportPage(props: { onBack: () => void }): JSX.Element {
  const app = useApp();
  const [text, setText] = createSignal('');
  const [format, setFormat] = createSignal<'vcard' | 'csv'>('vcard');
  const [parsedCsv, setParsedCsv] = createSignal<ReturnType<typeof parseCsv> | null>(null);
  const [mapping, setMapping] = createStore<{ cols: CsvMapping }>({ cols: [] });
  const [vcards, setVcards] = createSignal<ParsedContact[] | null>(null);
  const [excluded, setExcluded] = createSignal<ReadonlySet<number>>(new Set());
  const [imported, setImported] = createSignal<number | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [book, setBook] = createSignal<Id>(app.selectedAddressBookId() ?? app.addressBooks()[0]?.id ?? '');
  createEffect(() => { if (!book() && app.addressBooks()[0]) setBook(app.addressBooks()[0]!.id); });

  const detectFormat = (value: string): 'vcard' | 'csv' => (value.trim().toUpperCase().startsWith('BEGIN:VCARD') ? 'vcard' : 'csv');
  function preview(): void {
    const fmt = detectFormat(text()); setFormat(fmt); setExcluded(new Set<number>()); setImported(null);
    if (fmt === 'vcard') { setParsedCsv(null); setVcards(parseVCards(text())); }
    else { const pc = parseCsv(text()); setParsedCsv(pc); setMapping('cols', guessMapping(pc.headers)); setVcards(null); }
  }
  const csvPreview = createMemo<ParsedContact[]>(() => { const pc = parsedCsv(); return pc === null ? [] : csvToContacts(pc, mapping.cols); });
  const previewCards = (): ParsedContact[] => (format() === 'vcard' ? (vcards() ?? []) : csvPreview());
  const chosen = (): ParsedContact[] => previewCards().filter((_, i) => !excluded().has(i));
  function include(index: number, on: boolean): void { const next = new Set(excluded()); if (on) next.delete(index); else next.add(index); setExcluded(next); }
  async function commit(): Promise<void> {
    setBusy(true);
    try { setImported(await app.importContacts(chosen(), book() || undefined)); } finally { setBusy(false); }
  }
  async function onFile(e: Event & { currentTarget: HTMLInputElement }): Promise<void> {
    const file = e.currentTarget.files?.[0]; if (file === undefined) return;
    // Decoded by byte order mark, as the calendar import does (UTF-16 vCards).
    setText(await readCalendarFile(file)); preview();
  }
  return <Page testId="contacts-import-page" title={t('contacts-import-title')} help={t('contacts-import-hint')} listLabel={t('contacts-import-source')} onBack={props.onBack}
    list={<div class={css.fieldGroup}>
      <h2 class={css.pageSubheading}>{t('contacts-import-source')}</h2>
      <label class={css.fieldLabel} for="contacts-import-file">{t('contacts-import-file')}</label>
      <input id="contacts-import-file" type="file" accept=".vcf,.csv,text/vcard,text/csv" onChange={onFile} />
      <label class={css.fieldLabel} for="contacts-import-text">{t('contacts-paste')}</label>
      <textarea id="contacts-import-text" class={css.input} rows={8} value={text()} onInput={e => setText(e.currentTarget.value)} />
      <Show when={app.addressBooks().length > 1}>
        <label class={css.fieldLabel} for="contacts-import-book">{t('contacts-import-book')}</label>
        <select id="contacts-import-book" class={css.input} value={book()} onChange={e => setBook(e.currentTarget.value)}><For each={app.addressBooks()}>{b => <option value={b.id}>{b.name}</option>}</For></select>
      </Show>
      <div class={css.actionsStart}><button type="button" class={css.button} disabled={!text().trim()} onClick={preview}>{t('contacts-preview')}</button></div>
      <Show when={format() === 'csv' && parsedCsv() !== null}>
        <h3 class={css.fieldLabel}>{t('contacts-map-columns')}</h3>
        <div class={css.tableScroll}><table class={css.table}>
          <thead><tr><th class={css.th}>{t('contacts-col')}</th><th class={css.th}>{t('contacts-maps-to')}</th><th class={css.th}>{t('contacts-sample')}</th></tr></thead>
          <tbody><For each={parsedCsv()!.headers}>{(header, i) => <tr>
            <td class={css.td}><bdi>{header}</bdi></td>
            <td class={css.td}><select class={css.select} aria-label={t('contacts-map-column', { header })} value={mapping.cols[i()] ?? 'ignore'} onChange={e => setMapping('cols', i(), e.currentTarget.value as CsvField)}><For each={CSV_FIELDS}>{f => <option value={f}>{f}</option>}</For></select></td>
            <td class={css.td}><bdi>{parsedCsv()!.rows[0]?.[i()] ?? ''}</bdi></td>
          </tr>}</For></tbody>
        </table></div>
      </Show>
    </div>}
    detail={<div class={css.card}>
      <h2 class={css.pageSubheading}>{t('contacts-import-review')}</h2>
      <Show when={imported() === null} fallback={<><p role="status">{t('contacts-imported', { count: imported() ?? 0 })}</p><div class={css.actionsStart}><button type="button" class={css.button} onClick={props.onBack}>{t('common-done')}</button></div></>}>
        <Show when={previewCards().length > 0} fallback={<p class={css.empty}>{t('contacts-import-empty')}</p>}>
          <p class={css.fieldLabel}>{t('contacts-preview-count', { count: previewCards().length })}</p>
          <ul class={css.pageItems} aria-label={t('contacts-import-preview')}>
            <For each={previewCards()}>{(c, i) => <li><label class={css.fieldRow}><input type="checkbox" checked={!excluded().has(i())} onChange={e => include(i(), e.currentTarget.checked)} /><span class={css.rowBody}><span class={css.rowName}><bdi>{c.name.full || t('contacts-no-name')}</bdi></span><span class={css.rowMeta}><bdi>{c.emails[0]?.value ?? t('contacts-no-email')}</bdi></span></span></label></li>}</For>
          </ul>
          <div class={css.actionsStart}><button type="button" class={css.button} disabled={busy() || chosen().length === 0} onClick={() => void commit()}>{t('contacts-import-n', { count: chosen().length })}</button></div>
        </Show>
      </Show>
    </div>}
  />;
}

// ── Merge duplicates ─────────────────────────────────────────────────────────

export function MergePage(props: { onBack: () => void }): JSX.Element {
  const app = useApp();
  const clusters = createMemo(() => findDuplicateClusters(app.contacts()));
  const [index, setIndex] = createSignal(0);
  const cluster = createMemo(() => clusters()[Math.min(index(), clusters().length - 1)] ?? null);
  const [keepId, setKeepId] = createSignal<Id | null>(null);
  createEffect(() => { const c = cluster(); if (c && !c.some(card => card.id === keepId())) setKeepId(c[0]!.id); });
  const merged = createMemo<ContactCard | null>(() => {
    const c = cluster(); const keep = keepId(); if (!c || !keep) return null;
    const primary = c.find(card => card.id === keep); if (!primary) return null;
    return mergeCards(primary, c.filter(card => card.id !== keep));
  });
  const [busy, setBusy] = createSignal(false);
  const [done, setDone] = createSignal<string | null>(null);
  async function confirm(): Promise<void> {
    const c = cluster(); const keep = keepId(); const result = merged(); if (!c || !keep || !result) return;
    setBusy(true);
    try { if (await app.mergeContacts(keep, c.filter(card => card.id !== keep).map(card => card.id))) setDone(contactDisplayName(result)); } finally { setBusy(false); }
  }
  const name = (card: ContactCard): string => contactDisplayName(card) || primaryEmail(card) || t('contacts-no-name');
  return <Page testId="contacts-merge-page" title={t('contacts-merge-title')} help={t('contacts-merge-review-hint')} listLabel={t('contacts-merge-sets')} onBack={props.onBack}
    list={<Show when={clusters().length > 0} fallback={<p class={css.empty}>{t('contacts-no-duplicates')}</p>}>
      <ul class={css.pageItems}>
        <For each={clusters()}>{(c, i) => <li><button type="button" class={css.navButton} aria-current={cluster() === c} onClick={() => { setIndex(i()); setDone(null); }}><span class={css.rowBody}><span class={css.rowName}><bdi>{name(c[0]!)}</bdi></span><span class={css.rowMeta}>{t('contacts-possible-duplicates', { count: c.length })}</span></span></button></li>}</For>
      </ul>
    </Show>}
    detail={<>
      <Show when={done()}>{merged => <p role="status">{t('contacts-merge-done', { name: merged() })}</p>}</Show>
      <Show when={cluster()}>{c => <article class={css.card} aria-label={t('contacts-merge-set', { name: name(c()[0]!) })}>
        <fieldset class={`${css.fieldGroup} ${css.bareFieldset}`}>
          <legend class={css.fieldLabel}>{t('contacts-merge-keep')}</legend>
          <For each={c()}>{card => <label class={css.fieldRow}><input type="radio" name="contacts-merge-keep" checked={keepId() === card.id} onChange={() => setKeepId(card.id)} /><span class={css.rowBody}><span class={css.rowName}><bdi>{name(card)}</bdi></span><span class={css.rowMeta}><bdi>{[primaryEmail(card), card.organizations[0]].filter(Boolean).join(' · ') || t('contacts-no-email')}</bdi></span></span></label>}</For>
        </fieldset>
        <Show when={merged()}>{m => <section class={css.fieldGroup} aria-label={t('contacts-merged-preview')}>
          <h2 class={css.pageSubheading}>{t('contacts-merged-preview')}: <bdi>{contactDisplayName(m())}</bdi></h2>
          <span class={css.fieldLabel}>{t('contacts-emails')}</span>
          <For each={m().emails}>{e => <span><bdi>{e.value}</bdi></span>}</For>
          <Show when={m().phones.length > 0}><span class={css.fieldLabel}>{t('contacts-phones')}</span><For each={m().phones}>{p => <span><bdi>{p.value}</bdi></span>}</For></Show>
        </section>}</Show>
        <div class={css.actionsStart}><button type="button" class={css.button} disabled={busy() || !merged()} onClick={() => void confirm()}>{t('contacts-merge-confirm')}</button></div>
      </article>}</Show>
    </>}
  />;
}
