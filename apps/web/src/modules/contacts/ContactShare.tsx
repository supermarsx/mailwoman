// "Share contact": one trigger and a small menu with the four ways to hand a
// contact on. Used by the message address card and by the contacts module, so
// the actions behave the same in both. For an address that is not saved, the
// caller passes a card built from the address and the display name.

import { createSignal, createUniqueId, onCleanup, Show, For, type Accessor, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import type { ContactCard } from '../../api/pim-types.ts';
import type { ComposeInitial } from '../../components/compose/reply.ts';
import { Icon } from '../../components/Icon.tsx';
import '../../components/address-contact-card.css';
import { useApp } from '../../state/context.ts';
import { t } from '../../i18n';
import { canShareFile, contactShareDraft, contactSummary, contactVCard, copyText, downloadText, shareVCardFile, vcardFileName, VCARD_MIME } from './share.ts';

export interface ContactShareProps {
  card: () => ContactCard;
  /** The document the controls live in (a message can sit in a detached window). */
  document?: Document;
  /** Opens the composer with the draft. Without it the email action is disabled. */
  onCompose?: (draft: ComposeInitial) => void;
  /** Where the menu is drawn (a slot inside a dialog); without it the menu hangs under the trigger. */
  panelHost?: Accessor<HTMLElement | undefined>;
  /** Called after the email and system-share actions, which leave the host popover behind. */
  onDone?: () => void;
  triggerClass: string;
  menuClass?: string;
  itemClass?: string;
  /** Visible label next to the icon (contacts module); the card shows the icon only. */
  label?: boolean;
}

export interface ShareActions {
  email(): Promise<void>;
  copy(): Promise<void>;
  download(): void;
  native(): Promise<void>;
  /** The system share sheet exists here and can take a .vcf. */
  nativeAvailable(): boolean;
}

type AppForShare = Pick<ReturnType<typeof useApp>, 'accountId' | 'uploadComposeBlob' | 'showToast'>;

/** The four share actions for one card; the menu and the list's context menu both call these. */
export function createShareActions(app: AppForShare, card: () => ContactCard, doc: () => Document, onCompose?: (draft: ComposeInitial) => void, onLeave?: () => void): ShareActions {
  const fileName = (): string => vcardFileName(card().name.full || card().emails[0]?.value || '');
  return {
    nativeAvailable: () => {
      const nav = doc().defaultView?.navigator;
      return !!nav && typeof nav.share === 'function' && canShareFile(nav, new File([''], 'contact.vcf', { type: VCARD_MIME }));
    },
    async email() {
      const value = card(), account = app.accountId();
      onLeave?.();
      const vcf = contactVCard(value);
      // An attachment needs a stored blob; when that fails the vCard travels in the text instead.
      const stored = account ? await app.uploadComposeBlob(account, fileName(), vcf, VCARD_MIME) : null;
      if (!stored) app.showToast('info', t('contacts-share-attach-failed'));
      onCompose?.(contactShareDraft(value, stored));
    },
    async copy() {
      const ok = await copyText(doc(), contactVCard(card()).replace(/\r\n$/, ''));
      app.showToast(ok ? 'success' : 'error', t(ok ? 'contacts-share-copied' : 'contacts-share-copy-failed'));
    },
    download() {
      downloadText(doc(), fileName(), contactVCard(card()));
      app.showToast('success', t('contacts-share-downloaded'));
    },
    async native() {
      const value = card();
      onLeave?.();
      const result = await shareVCardFile(doc().defaultView?.navigator, fileName(), contactVCard(value), value.name.full, contactSummary(value));
      if (result === 'failed' || result === 'unsupported') app.showToast('error', t('contacts-share-failed'));
    },
  };
}

export function ContactShareMenu(props: ContactShareProps): JSX.Element {
  const app = useApp(), menuId = createUniqueId();
  const [open, setOpen] = createSignal(false);
  let trigger: HTMLButtonElement | undefined, menu: HTMLDivElement | undefined;
  let alive = true;
  onCleanup(() => { alive = false; });
  const doc = (): Document => props.document ?? document;
  const items = (): HTMLButtonElement[] => Array.from(menu?.querySelectorAll<HTMLButtonElement>('button[role="menuitem"]:not(:disabled)') ?? []);
  function close(restore = true): void { setOpen(false); if (restore) trigger?.focus({ preventScroll: true }); }
  function toggle(): void {
    const next = !open();
    setOpen(next);
    if (next) queueMicrotask(() => items()[0]?.focus({ preventScroll: true }));
  }
  function keydown(event: KeyboardEvent): void {
    const rows = items(), at = rows.indexOf(doc().activeElement as HTMLButtonElement);
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); close(); }
    else if (event.key === 'ArrowDown') { event.preventDefault(); rows[(at + 1) % rows.length]?.focus(); }
    else if (event.key === 'ArrowUp') { event.preventDefault(); rows[(at - 1 + rows.length) % rows.length]?.focus(); }
    else if (event.key === 'Home') { event.preventDefault(); rows[0]?.focus(); }
    else if (event.key === 'End') { event.preventDefault(); rows[rows.length - 1]?.focus(); }
    else if (event.key === 'Tab') close(false);
  }
  const actions = createShareActions(app, props.card, doc, props.onCompose, () => { close(false); props.onDone?.(); });

  const entries = (): { id: string; label: string; icon: 'mail' | 'copy' | 'download' | 'share'; disabled?: boolean; run: () => void }[] => [
    { id: 'email', label: t('contacts-share-email'), icon: 'mail', disabled: !props.onCompose, run: () => { void actions.email(); } },
    { id: 'copy', label: t('contacts-share-copy'), icon: 'copy', run: () => { void actions.copy().then(() => { if (alive) close(); }); } },
    { id: 'download', label: t('contacts-share-download'), icon: 'download', run: () => { actions.download(); close(); } },
    ...(actions.nativeAvailable() ? [{ id: 'native', label: t('contacts-share-native'), icon: 'share' as const, run: () => { void actions.native(); } }] : []),
  ];

  return <span class="contact-share">
    <button ref={trigger} type="button" class={props.triggerClass} classList={{ 'address-card__action--on': open() }} aria-haspopup="menu" aria-expanded={open()} aria-controls={open() ? menuId : undefined}
      title={t('contacts-share')} aria-label={t('contacts-share')} on:click={toggle}><Icon name="share"/><Show when={props.label}> {t('contacts-share')}</Show></button>
    <Show when={open()}>
      <Portal mount={props.panelHost?.() ?? trigger?.parentElement ?? doc().body} useShadow={false}>
      <div ref={menu} id={menuId} role="menu" class={props.menuClass ?? 'contact-share__menu'} aria-label={t('contacts-share')} on:keydown={keydown}>
        <For each={entries()}>{entry => <button type="button" role="menuitem" class={props.itemClass ?? 'contact-share__item'} disabled={entry.disabled} on:click={entry.run}><Icon name={entry.icon}/> <span>{entry.label}</span></button>}</For>
      </div>
      </Portal>
    </Show>
  </span>;
}
