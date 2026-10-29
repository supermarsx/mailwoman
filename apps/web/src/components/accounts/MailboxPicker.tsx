// The mailbox picker (t40, t48 k): the one active-account indicator and
// switcher in the sidebar head. `full` shows the active mailbox (avatar,
// nickname or address, unread count) as the trigger; `icon` is a one-letter
// avatar button for the compact sidebar and the hidden-sidebar top bar. Both
// open one menu.
//
// It is a switcher only: each entry is the avatar, nickname or address, unread
// count and the active check, in the owner's saved order. Reordering, renaming,
// the primary mailbox, renewing access and unlinking are managed in Settings ›
// Linked mailboxes, never here. Choosing a link whose access expired opens the
// caller's renew flow (`onRenew`) instead of failing.
//
// The menu is an ARIA menu of `menuitemradio` entries: Up/Down move, Home/End
// jump, a printed character jumps to the next mailbox whose label starts with
// the typed text, Enter/Space activate, Esc closes and returns focus, Tab
// closes. With one mailbox the caller renders `MailboxPlain`: no menu at all.
import { For, Show, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { isolate, loadCatalog, t } from '../../i18n/index.ts';
import { useCollapsibleBar } from '../bars/index.ts';
import { Icon } from '../Icon.tsx';
import { avatarPresentation } from '../PersonAvatar.tsx';
import * as a11y from '../mailA11y.css.ts';
import * as css from './accountIndicator.css.ts';
import '../person-avatar.css';
import '../senderIcons/palette.css';

/**
 * One picker entry. Also the shape other lanes feed (t46: live unread counts):
 * `unread` is the inbox unread count or null when unknown.
 */
export interface MailboxChoice {
  /** Null means the sign-in (parent) account; every child ID comes from the owned server list. */
  linkId: string | null;
  address: string;
  /** The link's display name from linking, if any. */
  name?: string;
  /** The owner's display-only nickname (t40 g). */
  nickname?: string;
  /** The sign-in account. */
  signIn?: boolean;
  available: boolean;
  unread: number | null;
  /** t48: access ended and can be renewed in place ("Renew access"). */
  renew?: boolean;
}

let sequence = 0;

export function accountIndicatorLabel(address: string, linked: boolean, nickname?: string): string {
  const shown = nickname ? `${nickname} (${address})` : address;
  return linked
    ? t('mail-account-linked-active', { address: isolate(shown) })
    : t('mail-account-active', { address: isolate(shown) });
}

/** The main label: nickname, else a distinct link name, else the address. */
export function choiceLabel(choice: Pick<MailboxChoice, 'nickname' | 'name' | 'address'>): string {
  return choice.nickname || (choice.name && choice.name !== choice.address ? choice.name : choice.address);
}

export function MailboxAvatar(props: { label: string; address: string; size?: 'sm' | 'md' }): JSX.Element {
  const look = (): ReturnType<typeof avatarPresentation> => avatarPresentation(props.label === props.address ? '' : props.label, props.address);
  return (
    <span class={`person-avatar ${css.avatar}`} data-size={props.size ?? 'md'} data-avatar-color={look().palette} aria-hidden="true">
      {look().initials}
    </span>
  );
}

function UnreadBadge(props: { count: number | null }): JSX.Element {
  return (
    <Show when={props.count !== null && props.count > 0}>
      <span class={`sidebar__badge ${css.unread}`} data-mailbox-unread>
        {props.count! > 999 ? '999+' : props.count}
        <span class={a11y.srOnly}> {t('linked-accounts-unread', { count: props.count! })}</span>
      </span>
    </Show>
  );
}

/** One mailbox, shown as plain text: nothing to choose (t40 b). */
export function MailboxPlain(props: { variant: 'full' | 'icon'; address: string; nickname?: string | undefined; unread: number | null }): JSX.Element {
  const label = (): string => accountIndicatorLabel(props.address, false, props.nickname);
  return (
    <Show
      when={props.variant === 'icon'}
      fallback={
        <span class={css.plain} data-account-indicator="plain" title={props.nickname ? `${props.nickname}\n${props.address}` : props.address} data-account-address>
          <span class={css.plainText}>{isolate(props.nickname || props.address)}</span>
          <UnreadBadge count={props.unread} />
        </span>
      }
    >
      <span
        class={`${css.iconButton} ${css.iconStatic} ${a11y.focusable}`}
        data-account-indicator="icon"
        data-static="true"
        role="img"
        tabIndex={0}
        aria-label={label()}
        data-tooltip={label()}
      >
        <span aria-hidden="true">{Array.from((props.nickname || props.address).trim())[0]?.toLocaleUpperCase() ?? '?'}</span>
      </span>
    </Show>
  );
}

export function MailboxPicker(props: {
  variant: 'full' | 'icon';
  /** In the owner's saved order. */
  choices: readonly MailboxChoice[];
  activeLinkId: string | null;
  busy: boolean;
  error?: string;
  onSelect: (linkId: string | null) => void;
  /** t48: choosing a link whose access expired opens this (the renew flow). */
  onRenew?: (linkId: string) => void;
}): JSX.Element {
  onMount(() => {
    void loadCatalog('bars');
    void loadCatalog('linked-accounts');
  });
  const bar = useCollapsibleBar();
  const menuId = `mw-mailbox-menu-${++sequence}`;
  const [open, setOpen] = createSignal(false);
  const [position, setPosition] = createSignal({ top: 0, left: 0 });
  let trigger: HTMLButtonElement | undefined;
  let menu: HTMLDivElement | undefined;
  let release: (() => void) | undefined;
  let typed = '';
  let typedAt = 0;

  const active = (): MailboxChoice | undefined =>
    props.choices.find((choice) => choice.linkId === props.activeLinkId) ?? props.choices[0];
  const linked = (): boolean => props.activeLinkId !== null;
  const label = (): string => accountIndicatorLabel(active()?.address ?? '', linked(), active()?.nickname);
  const fullTitle = (choice: MailboxChoice): string => (choiceLabel(choice) === choice.address ? choice.address : `${choiceLabel(choice)}\n${choice.address}`);


  function place(): void {
    if (!trigger) return;
    const rect = trigger.getBoundingClientRect();
    const view = trigger.ownerDocument.defaultView;
    const width = menu?.offsetWidth ?? 0;
    const maxLeft = Math.max(8, (view?.innerWidth ?? rect.right) - width - 8);
    setPosition({ top: rect.bottom + 4, left: Math.min(Math.max(8, rect.left), maxLeft) });
  }
  const items = (): HTMLButtonElement[] => Array.from(menu?.querySelectorAll<HTMLButtonElement>('[data-mailbox-item]') ?? []);
  function focusItem(index: number): void {
    const list = items();
    if (list.length === 0) return;
    list[((index % list.length) + list.length) % list.length]?.focus();
  }
  function show(focus: 'active' | 'last' = 'active'): void {
    if (open()) return;
    release = bar?.hold();
    place();
    setOpen(true);
    queueMicrotask(() => {
      place();
      const list = items();
      if (focus === 'last') focusItem(list.length - 1);
      else focusItem(Math.max(0, list.findIndex((item) => item.getAttribute('aria-checked') === 'true')));
    });
  }
  function close(restoreFocus = false): void {
    if (!open()) return;
    setOpen(false);
    release?.();
    release = undefined;
    if (restoreFocus) trigger?.focus();
  }
  const renewable = (choice: MailboxChoice): boolean => !!choice.renew && choice.linkId !== null && !!props.onRenew;
  function choose(choice: MailboxChoice): void {
    if (props.busy) return;
    if (renewable(choice)) {
      close(true);
      props.onRenew?.(choice.linkId!);
      return;
    }
    if (!choice.available) return;
    close(true);
    if (choice.linkId !== props.activeLinkId) props.onSelect(choice.linkId);
  }

  function onMenuKeyDown(event: KeyboardEvent): void {
    const target = event.target instanceof HTMLElement ? event.target : null;
    const list = items();
    const main = target?.closest<HTMLButtonElement>('[data-mailbox-item]') ?? null;
    const index = main ? list.indexOf(main) : -1;
    switch (event.key) {
      case 'ArrowDown': event.preventDefault(); focusItem(index + 1); return;
      case 'ArrowUp': event.preventDefault(); focusItem(index < 0 ? list.length - 1 : index - 1); return;
      case 'Home': event.preventDefault(); focusItem(0); return;
      case 'End': event.preventDefault(); focusItem(list.length - 1); return;
      case 'Escape': event.preventDefault(); event.stopPropagation(); close(true); return;
      case 'Tab': close(); return;
      default: break;
    }
    if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey && event.key.trim() !== '') {
      const now = Date.now();
      typed = now - typedAt > 700 ? event.key.toLocaleLowerCase() : typed + event.key.toLocaleLowerCase();
      typedAt = now;
      const start = typed.length === 1 ? index + 1 : Math.max(index, 0);
      for (let step = 0; step < list.length; step++) {
        const candidate = list[(start + step) % list.length];
        const text = `${candidate?.dataset['label'] ?? ''}\n${candidate?.dataset['address'] ?? ''}`.toLocaleLowerCase();
        if (text.split('\n').some((part) => part.startsWith(typed))) {
          event.preventDefault();
          candidate?.focus();
          return;
        }
      }
    }
  }

  const onDocumentPointerDown = (event: Event): void => {
    const target = event.target;
    if (!open() || !(target instanceof Node) || trigger?.contains(target) || menu?.contains(target)) return;
    close();
  };
  // Esc closes the menu even when focus is outside it (after a pointer drag).
  const onDocumentKeyDown = (event: KeyboardEvent): void => {
    if (!open() || event.key !== 'Escape' || event.defaultPrevented || (event.target instanceof Node && menu?.contains(event.target))) return;
    event.preventDefault();
    close(true);
  };
  onMount(() => {
    const doc = trigger?.ownerDocument ?? document;
    doc.addEventListener('pointerdown', onDocumentPointerDown, true);
    doc.addEventListener('keydown', onDocumentKeyDown);
    onCleanup(() => {
      doc.removeEventListener('pointerdown', onDocumentPointerDown, true);
      doc.removeEventListener('keydown', onDocumentKeyDown);
    });
  });
  onCleanup(() => {
    release?.();
    release = undefined;
  });

  const triggerKeys = (event: KeyboardEvent): void => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      show(event.key === 'ArrowUp' ? 'last' : 'active');
    }
  };
  const triggerName = (): string => {
    const choice = active();
    if (!choice) return t('linked-accounts-select');
    return `${t('linked-accounts-select')}: ${choice.nickname ? `${choice.nickname} (${choice.address})` : choice.address}`;
  };

  return (
    <div class={css.picker} data-account-picker={props.variant} aria-busy={props.busy}>
      <Show
        when={props.variant === 'icon'}
        fallback={
          <button
            ref={trigger}
            type="button"
            class={`${css.trigger} ${a11y.focusable}`}
            data-account-indicator="picker"
            data-linked={linked() ? 'true' : 'false'}
            aria-haspopup="menu"
            aria-expanded={open() ? 'true' : 'false'}
            aria-controls={open() ? menuId : undefined}
            aria-label={triggerName()}
            title={active() ? fullTitle(active()!) : undefined}
            disabled={props.busy}
            onClick={() => (open() ? close() : show())}
            onKeyDown={triggerKeys}
          >
            <MailboxAvatar label={active() ? choiceLabel(active()!) : ''} address={active()?.address ?? ''} size="sm" />
            <span class={css.triggerText}>
              <span class={css.triggerAddress} data-mailbox-label>{isolate(active()?.nickname || (active()?.address ?? ''))}</span>
              <Show when={active()?.nickname}><span class={css.triggerSecondary}>{isolate(active()!.address)}</span></Show>
            </span>
            <UnreadBadge count={active()?.unread ?? null} />
            <span class={css.chevron} aria-hidden="true"><Icon name="chevronDown" /></span>
          </button>
        }
      >
        <button
          ref={trigger}
          type="button"
          class={`btn btn--ghost ${css.iconButton}`}
          data-account-indicator="icon"
          data-linked={linked() ? 'true' : 'false'}
          aria-label={label()}
          data-tooltip={label()}
          aria-haspopup="menu"
          aria-expanded={open() ? 'true' : 'false'}
          aria-controls={open() ? menuId : undefined}
          disabled={props.busy}
          onClick={() => (open() ? close() : show())}
          onKeyDown={triggerKeys}
        >
          <span aria-hidden="true">{Array.from((active() ? choiceLabel(active()!) : '').trim())[0]?.toLocaleUpperCase() ?? '?'}</span>
        </button>
      </Show>
      <Show when={props.error}><span role="alert" class={css.error}>{props.error}</span></Show>
      <Show when={open()}>
        <Portal mount={trigger?.ownerDocument.body ?? document.body}>
          <div
            ref={menu}
            id={menuId}
            role="menu"
            aria-label={t('linked-accounts-select')}
            class={css.popover}
            data-mailbox-menu
            style={{ top: `${position().top}px`, left: `${position().left}px` }}
            onKeyDown={onMenuKeyDown}
            onFocusOut={(event) => {
              const next = event.relatedTarget;
              if (next instanceof Node && (menu?.contains(next) || trigger?.contains(next))) return;
              if (next instanceof Node) close();
            }}
          >
            <For each={props.choices}>
              {(choice) => {
                const isActive = (): boolean => choice.linkId === props.activeLinkId;
                return (
                  <div class={css.row} role="none" data-mailbox-row data-active={isActive() ? 'true' : undefined}>
                    <button
                      type="button"
                      role="menuitemradio"
                      tabIndex={-1}
                      class={css.item}
                      aria-checked={isActive() ? 'true' : 'false'}
                      aria-disabled={(!choice.available && !renewable(choice)) || props.busy ? 'true' : undefined}
                      data-mailbox-item
                      data-link={choice.linkId ?? ''}
                      data-label={choiceLabel(choice)}
                      data-address={choice.address}
                      data-expired={choice.renew ? 'true' : undefined}
                      title={fullTitle(choice)}
                      onClick={() => choose(choice)}
                    >
                      <MailboxAvatar label={choiceLabel(choice)} address={choice.address} />
                      <span class={css.itemText}>
                        <span class={css.itemName} data-mailbox-label>{isolate(choiceLabel(choice))}</span>
                        <Show when={choiceLabel(choice) !== choice.address}><span class={css.itemAddress}>{isolate(choice.address)}</span></Show>
                        {/* No status on the entry (t48 j/k); screen readers still hear why it opens renewal. */}
                        <Show when={!choice.available}><span class={a11y.srOnly}>{t(choice.renew ? 'linked-accounts-expired' : 'linked-accounts-unavailable')}</span></Show>
                      </span>
                      <UnreadBadge count={choice.unread} />
                      <span class={css.check} aria-hidden="true"><Show when={isActive()}><Icon name="check" /></Show></span>
                    </button>
                  </div>
                );
              }}
            </For>
          </div>
        </Portal>
      </Show>
    </div>
  );
}
