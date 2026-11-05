import { createEffect, createMemo, createResource, createSignal, createUniqueId, For, onCleanup, onMount, Show, type Accessor, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import type { ContactCard } from '../api/pim-types.ts';
import type { EmailAddress } from '../api/jmap-types.ts';
import { observePreferenceAccount } from '../api/accountPreferenceLifecycle.ts';
import { useApp } from '../state/context.ts';
import { featureOn } from '../state/features.ts';
import { loadCatalog, t } from '../i18n';
import { routeHash } from '../shell/router.ts';
import { normalizeContactAddress } from '../modules/contacts/contactAddress.ts';
import { ContactShareMenu } from '../modules/contacts/ContactShare.tsx';
import { cardForAddress, copyText } from '../modules/contacts/share.ts';
import { formatAddressList, type ComposeInitial } from './compose/reply.ts';
import { PersonAvatar } from './PersonAvatar.tsx';
import { Icon } from './Icon.tsx';
import { createFocusTrap } from './a11y';
import './address-contact-card.css';

export interface ContactCardPoint { x: number; y: number; document: Document; trigger?: HTMLElement }
interface Target extends ContactCardPoint { address: string; name: string; scope: string }
export function contactComposeInitial(address: string, name = ''): ComposeInitial|null {
  const valid = normalizeContactAddress(address);
  if (!valid) return null;
  return { mode: 'new', to: formatAddressList([{ email: valid, name: name || null }]), cc: '', subject: '', bodyHtml: '', bodyText: '', inReplyTo: [], references: [], attachments: [], quotesDecrypted: false };
}

/** One card owner per reader; account/message changes revoke its snapshot. */
export function createAddressContactCard(options: { scope: Accessor<string>; active: Accessor<boolean>; onCompose?: (draft: ComposeInitial) => void }) {
  const [target, setTarget] = createSignal<Target>();
  const scope = createMemo(options.scope), active = createMemo(options.active);
  createEffect(() => { void scope(); void active(); setTarget(undefined); });
  onCleanup(observePreferenceAccount(() => setTarget(undefined)));
  const close = ():void => setTarget(undefined);
  function open(address: string, name: string, anchor: HTMLElement|ContactCardPoint): void {
    const valid = normalizeContactAddress(address);
    if (!valid || !options.active()) return;
    let point: ContactCardPoint;
    if ('ownerDocument' in anchor) {
      const rect = anchor.getBoundingClientRect();
      point = { x: rect.left, y: rect.bottom + 4, document: anchor.ownerDocument, trigger: anchor };
    } else point = anchor;
    setTarget({ ...point, address: valid, name, scope: options.scope() });
  }
  return { open, close, isOpen: () => target() !== undefined, card: ():JSX.Element => <Show when={target()} keyed>{value => <AddressContactCard target={value} onClose={close} {...(options.onCompose ? { onCompose: options.onCompose } : {})}/>}</Show> };
}

/** Actual address buttons are separate from the disclosure's summary control. */
export function ContactAddresses(props: { addresses: EmailAddress[]|null|undefined; onOpen: (address: string, name: string, trigger: HTMLElement) => void }): JSX.Element {
  return <span class="contact-addresses"><For each={props.addresses ?? []}>{(address, index) => <><Show when={index() > 0}><span aria-hidden="true">, </span></Show><Show when={normalizeContactAddress(address.email)} fallback={<bdi>{address.name || address.email}</bdi>}>
    <button type="button" class="contact-address" title={address.email} aria-haspopup="dialog" aria-label={`${t('contacts-address-card')}: ${address.name || address.email}`}
      on:click={event => { event.preventDefault(); event.stopPropagation(); props.onOpen(address.email, address.name ?? '', event.currentTarget); }}><bdi>{address.name || address.email}</bdi><Show when={address.name}><span class="contact-address__email"> &lt;<bdi>{address.email}</bdi>&gt;</span></Show></button>
  </Show></>}</For></span>;
}

function AddressContactCard(props: { target: Target; onClose: () => void; onCompose?: (draft: ComposeInitial) => void }): JSX.Element {
  const app = useApp(), nameId = createUniqueId(), bookId = createUniqueId(), titleId = createUniqueId();
  const [element, setElement] = createSignal<HTMLElement>();
  const [name, setName] = createSignal(props.target.name);
  const [book, setBook] = createSignal('');
  const [saving, setSaving] = createSignal(false);
  const [saved, setSaved] = createSignal<ContactCard>();
  const [saveOpen, setSaveOpen] = createSignal(false);
  const [copied, setCopied] = createSignal(false);
  const [shareSlot, setShareSlot] = createSignal<HTMLElement>();
  // The star flips at once; the store confirms, or the flip is undone.
  const [favoriteOverride, setFavoriteOverride] = createSignal<boolean>();
  const [favoriteBusy, setFavoriteBusy] = createSignal(false);
  let copyTimer: ReturnType<typeof setTimeout>|undefined;
  const [position, setPosition] = createSignal({ left: 8, top: 8, maxHeight: Math.max(44, (props.target.document.defaultView?.innerHeight ?? 600) - 16) });
  let alive = true;
  onCleanup(() => { alive = false; clearTimeout(copyTimer); });
  const [lookup, { refetch }] = createResource(() => props.target.address, address => featureOn('contacts') ? app.lookupContactAddress(address) : Promise.resolve(null));
  // Never read a pending resource: it would suspend/remount the mail workspace.
  const result = () => lookup.state === 'ready' ? lookup() : undefined;
  const contact = () => featureOn('contacts') ? saved() ?? result()?.contact : undefined;
  const fullName = () => contact()?.name.full || props.target.name || props.target.address;
  const books = () => result()?.addressBooks ?? [];
  const canSave = () => featureOn('contacts') && !!result() && !contact() && books().length > 0;
  const isFavorite = () => favoriteOverride() ?? contact()?.isFavorite ?? false;
  const shareCard = (): ContactCard => contact() ?? cardForAddress(props.target.address, name() || props.target.name);
  createEffect(() => { const available = books(); if (available.length && !available.some(value => value.id === book())) setBook(available.find(value => value.isDefault)?.id ?? available[0]!.id); });
  function reposition(): void {
    const node = element(), host = props.target.document.defaultView;
    if (!node || !host) return;
    const viewport = host.visualViewport, width = viewport?.width ?? host.innerWidth, height = viewport?.height ?? host.innerHeight;
    const left = viewport?.offsetLeft ?? 0, top = viewport?.offsetTop ?? 0, rect = node.getBoundingClientRect();
    const next = { left: Math.max(left + 8, Math.min(props.target.x, left + width - rect.width - 8)), top: Math.max(top + 8, Math.min(props.target.y, top + height - Math.min(rect.height, height - 16) - 8)), maxHeight: Math.max(44, height - 16) };
    setPosition(previous => previous.left === next.left && previous.top === next.top && previous.maxHeight === next.maxHeight ? previous : next);
  }
  createEffect(() => { void result(); void contact(); queueMicrotask(reposition); });
  createFocusTrap(element, { onEscape: props.onClose });
  onMount(() => {
    void loadCatalog('contacts');
    const doc = props.target.document, host = doc.defaultView;
    const node = element();
    if (node && host) {
      reposition();
      const observer = typeof host.ResizeObserver === 'function' ? new host.ResizeObserver(reposition) : undefined;
      observer?.observe(node);
      host.addEventListener('resize', reposition);
      host.visualViewport?.addEventListener('resize', reposition);
      host.visualViewport?.addEventListener('scroll', reposition);
      onCleanup(() => { observer?.disconnect(); host.removeEventListener('resize', reposition); host.visualViewport?.removeEventListener('resize', reposition); host.visualViewport?.removeEventListener('scroll', reposition); });
    }
    onCleanup(() => { if (props.target.trigger?.isConnected) props.target.trigger.focus({ preventScroll: true }); });
  });
  function save(event: Event): Promise<void> {
    event.preventDefault();
    return persist(false);
  }
  async function persist(favorite: boolean): Promise<void> {
    if (saving() || !canSave() || !books().some(value => value.id === book())) return;
    setSaving(true);
    try {
      const id = await app.createContact({ name: { full: name().trim(), given: '', surname: '', prefix: '', suffix: '' }, emails: [{ value: props.target.address, context: '', pref: 1 }], ...(favorite ? { isFavorite: true } : {}) }, book());
      if (!alive) return;
      const card = id ? app.contactById(id) : undefined;
      if (card) { setSaved(card); app.showToast('success', t(favorite ? 'contacts-address-saved-favorite' : 'contacts-address-saved')); queueMicrotask(() => element()?.focus({ preventScroll: true })); }
      else app.showToast('error', t('contacts-address-save-failed'));
    } catch { if (alive) app.showToast('error', t('contacts-address-save-failed')); }
    finally { if (alive) setSaving(false); }
  }
  function copyPayload(): string {
    const card = contact(), display = fullName();
    const lines = [display && display !== props.target.address ? `${display} <${props.target.address}>` : props.target.address];
    for (const phone of card?.phones ?? []) lines.push(phone.value);
    for (const org of card?.organizations ?? []) lines.push(org);
    return lines.join('\n');
  }
  async function copy(): Promise<void> {
    const ok = await copyText(props.target.document, copyPayload());
    if (!alive) return;
    app.showToast(ok ? 'success' : 'error', t(ok ? 'contacts-address-copied' : 'contacts-address-copy-failed'));
    if (ok) { setCopied(true); clearTimeout(copyTimer); copyTimer = setTimeout(() => { if (alive) setCopied(false); }, 1500); }
  }
  async function toggleFavorite(): Promise<void> {
    const saved = contact();
    if (!saved) { void persist(true); return; }
    if (favoriteBusy()) return;
    const previous = isFavorite();
    setFavoriteBusy(true); setFavoriteOverride(!previous);
    const ok = (await app.setFavorite([saved.id], !previous)).includes(saved.id);
    if (!alive) return;
    setFavoriteBusy(false);
    if (ok) app.showToast('success', t(previous ? 'contacts-favorite-removed' : 'contacts-favorite-added', { name: fullName() }));
    else { setFavoriteOverride(previous); app.showToast('error', t('contacts-favorite-failed')); }
  }
  function goToContact(id: string): void {
    const host = props.target.document.defaultView;
    props.onClose();
    app.selectContact(id);
    if (host) host.location.hash = routeHash('contacts', id);
  }
  function compose(): void {
    const draft = contactComposeInitial(props.target.address, contact()?.name.full || name());
    if (draft && props.onCompose) { props.onClose(); props.onCompose(draft); }
  }
  return <Portal mount={props.target.document.body}><div class="address-card-backdrop" on:pointerdown={event => { if (event.target === event.currentTarget) props.onClose(); }}>
    <section ref={setElement} class="address-card" role="dialog" aria-modal="true" aria-labelledby={titleId} tabIndex={-1} style={{ left: `${position().left}px`, top: `${position().top}px`, 'max-height': `${position().maxHeight}px` }}>
      <header class="address-card__header"><PersonAvatar address={props.target.address} name={fullName()}/><div><h2 id={titleId}>{fullName()}</h2><bdi class="address-card__address">{props.target.address}</bdi></div><button type="button" class="btn btn--ghost" aria-label={t('common-close')} on:click={props.onClose}><Icon name="close"/></button></header>
      <Show when={lookup.loading}><p role="status">{t('contacts-address-loading')}</p></Show>
      <Show when={lookup.error || (lookup.state === 'ready' && !result())}><p role="status">{t('contacts-address-lookup-failed')}</p><button type="button" class="btn btn--ghost" on:click={() => void refetch()}>{t('common-retry')}</button></Show>
      <Show when={contact()}>{card => <div class="address-card__information">
        <Show when={card().organizations.length}><p><For each={card().organizations}>{value => <span class="address-card__line">{value}</span>}</For></p></Show>
        <Show when={card().titles.length}><p><For each={card().titles}>{value => <span class="address-card__line">{value}</span>}</For></p></Show>
        <dl><For each={card().emails}>{email => <><dt>{email.context || t('contacts-email')}</dt><dd><bdi>{email.value}</bdi></dd></>}</For><For each={card().phones}>{phone => <><dt>{phone.context || t('contacts-phone')}</dt><dd><bdi>{phone.value}</bdi></dd></>}</For></dl>
        <Show when={card().notes}><p class="address-card__notes">{card().notes}</p></Show>
      </div>}</Show>
      <div class="address-card__actions" role="group" aria-label={t('contacts-address-actions')}>
        <Show when={contact()} fallback={<button type="button" class="address-card__action" classList={{ 'address-card__action--on': saveOpen() }} disabled={!canSave()} aria-busy={lookup.loading} aria-expanded={canSave() ? saveOpen() : undefined} title={t('contacts-address-save-to')} aria-label={t('contacts-address-save-to')} on:click={() => setSaveOpen(value => !value)}><Icon name="plus"/></button>}>
          {card => <button type="button" class="address-card__action" title={t('contacts-address-goto')} aria-label={t('contacts-address-goto')} on:click={() => goToContact(card().id)}><Icon name="person"/></button>}
        </Show>
        <button type="button" class="address-card__action" disabled={!props.onCompose} title={t('contacts-address-compose')} aria-label={t('contacts-address-compose')} on:click={compose}><Icon name="mail"/></button>
        <button type="button" class="address-card__action" classList={{ 'address-card__action--on': isFavorite() }} disabled={!contact() && !canSave()} aria-busy={favoriteBusy() || undefined}
          aria-pressed={isFavorite()} aria-label={t('contacts-favorite', { name: fullName() })} title={contact() ? t(isFavorite() ? 'contacts-favorite-remove' : 'contacts-favorite-add') : t('contacts-favorite-save')} on:click={() => void toggleFavorite()}><Icon name={isFavorite() ? 'star' : 'starOutline'}/></button>
        <button type="button" class="address-card__action" title={t('contacts-address-copy')} aria-label={t('contacts-address-copy')} on:click={() => void copy()}><Icon name={copied() ? 'check' : 'copy'}/></button>
        <ContactShareMenu card={shareCard} document={props.target.document} panelHost={shareSlot} triggerClass="address-card__action" onDone={props.onClose} {...(props.onCompose ? { onCompose: props.onCompose } : {})}/>
      </div>
      <div class="address-card__share-slot" ref={setShareSlot}/>
      <Show when={canSave() && saveOpen()}><form class="address-card__save" on:submit={event => void save(event)}><p>{t('contacts-address-unsaved')}</p>
        <label for={nameId}>{t('contacts-full-name')}</label><input id={nameId} class="input" value={name()} on:input={event => setName(event.currentTarget.value)} disabled={saving()}/>
        <label for={bookId}>{t('contacts-address-book')}</label><select id={bookId} value={book()} on:change={event => setBook(event.currentTarget.value)} disabled={saving()}><For each={books()}>{value => <option value={value.id}>{value.name}</option>}</For></select>
        <button type="submit" class="btn" disabled={saving()}>{t('contacts-address-save')}</button>
      </form></Show>
    </section>
  </div></Portal>;
}
