// Insert business card: "My card" (from the identity, editable here) or a contact.
// Inserts a mail-safe card, the vCard 4.0 as an attachment, or both.

import { createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import type { ContactCard } from '../../../api/pim-types.ts';
import { t } from '../../../i18n/index.ts';
import { InsertDialog } from './InsertDialog.tsx';
import { businessCardHtml, businessCardText, businessCardVcf, fieldsFromContact, fieldsFromIdentity, type CardFields, type CardMode } from './businessCard.ts';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('compose-insert');

export type CardSource = { kind: 'self' } | { kind: 'contact' };

const FIELDS: readonly (keyof Omit<CardFields, 'photoSrc'>)[] = ['name', 'title', 'org', 'phone', 'email', 'address'];

export function BusinessCardDialog(props: {
  source: CardSource;
  identity: { name: string; email: string; organization?: string | null } | null;
  contacts: ContactCard[];
  returnTo: HTMLElement | null;
  onClose: () => void;
  onInsert: (html: string | null, text: string, vcf: { name: string; text: string; type: string } | null) => void;
}): JSX.Element {
  const [fields, setFields] = createSignal<CardFields>(props.source.kind === 'self' ? fieldsFromIdentity(props.identity) : { ...fieldsFromIdentity(null) });
  const [mode, setMode] = createSignal<CardMode>('both');
  const [query, setQuery] = createSignal('');
  const matches = createMemo(() => {
    const q = query().trim().toLowerCase();
    return props.contacts.filter((c) => q === '' || c.name.full.toLowerCase().includes(q) || c.emails.some((e) => e.value.toLowerCase().includes(q))).slice(0, 50);
  });
  const set = (key: keyof CardFields, value: string): void => { setFields({ ...fields(), [key]: value }); };
  const ready = (): boolean => fields().name.trim() !== '' || fields().email.trim() !== '';
  const insert = (): void => {
    const f = fields();
    props.onInsert(mode() === 'vcard' ? null : businessCardHtml(f), mode() === 'vcard' ? '' : businessCardText(f), mode() === 'card' ? null : businessCardVcf(f));
  };
  const label = (key: string): string => t(`ins-card-field-${key}`);
  return (
    <InsertDialog title={t(props.source.kind === 'self' ? 'ins-card-mine' : 'ins-card-contact')} testId="insert-card-dialog" returnTo={props.returnTo} onClose={props.onClose} onSubmit={insert} submitDisabled={!ready()}>
      <Show when={props.source.kind === 'contact'}>
        <label>{t('ins-card-search')}<input type="search" data-testid="insert-card-search" value={query()} onInput={(e) => setQuery(e.currentTarget.value)} /></label>
        <ul role="listbox" aria-label={t('ins-card-contact')} class="mw-insert-list">
          <For each={matches()} fallback={<li class="mw-insert-note">{t('ins-card-no-contacts')}</li>}>{(c) => (
            <li><button type="button" role="option" class="btn btn--ghost mw-insert-option" data-testid="insert-card-pick" aria-selected={fields().email === (c.emails[0]?.value ?? '') && fields().name === c.name.full} onClick={() => setFields(fieldsFromContact(c))}>{c.name.full || c.emails[0]?.value}</button></li>
          )}</For>
        </ul>
      </Show>
      <div class="mw-insert-row"><For each={FIELDS}>{(key) => (
        <label>{label(key)}<input data-testid={`insert-card-${key}`} value={fields()[key]} onInput={(e) => set(key, e.currentTarget.value)} /></label>
      )}</For></div>
      <label>{t('ins-card-mode')}
        <select data-testid="insert-card-mode" value={mode()} onChange={(e) => setMode(e.currentTarget.value as CardMode)}>
          <option value="both">{t('ins-card-mode-both')}</option>
          <option value="card">{t('ins-card-mode-card')}</option>
          <option value="vcard">{t('ins-card-mode-vcard')}</option>
        </select>
      </label>
      <Show when={mode() !== 'vcard'}><div class="mw-insert-preview" aria-label={t('ins-preview')} innerHTML={businessCardHtml(fields())} /></Show>
    </InsertDialog>
  );
}
