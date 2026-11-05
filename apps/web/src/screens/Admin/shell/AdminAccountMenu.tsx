// The signed-in admin as an avatar with a small menu (language, Sign out).
//
// The avatar is the webmail's: `.person-avatar` with the palette colour that
// `avatarPresentation` derives from the name, so one username is always the same
// colour. The popover is `position: fixed` (the sidebar scrolls and would clip
// an absolute one). Escape closes it and returns focus to the button, Arrow Up
// and Down, Home and End move between its controls, and a press outside closes.

import { Show, createSignal, onCleanup, createEffect, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import { focusableWithin } from '../../../components/a11y';
import { avatarPresentation } from '../../../components/avatarPresentation.ts';
import { Icon } from './icons.tsx';
import '../../../components/person-avatar.css';
import '../../../components/senderIcons/palette.css';
import * as css from './accountMenu.css.ts';

const POPOVER_WIDTH = 288;
let sequence = 0;

export interface AdminAccountMenuProps {
  username: string;
  onSignOut: () => void;
  /** The language picker slot. */
  languagePicker?: JSX.Element | undefined;
  /** `full` shows the name beside the avatar; `icon` is the avatar alone. */
  variant: 'full' | 'icon';
  /** Open upward (sidebar footer) or downward (headers). */
  placement: 'up' | 'down';
  testId?: string;
}

export function AdminAccountMenu(props: AdminAccountMenuProps): JSX.Element {
  const id = `admin-account-menu-${++sequence}`;
  const [open, setOpen] = createSignal(false);
  const [pos, setPos] = createSignal<JSX.CSSProperties>({ left: '0px' });
  let root: HTMLDivElement | undefined;
  let button: HTMLButtonElement | undefined;
  let popover: HTMLDivElement | undefined;

  const look = () => avatarPresentation(props.username, props.username);
  const label = () => t('admin-account-menu', { name: isolate(props.username) });

  function place(): void {
    if (!button) return;
    const r = button.getBoundingClientRect();
    const left = `${Math.max(8, Math.min(r.left, window.innerWidth - POPOVER_WIDTH - 8))}px`;
    setPos(
      props.placement === 'up'
        ? { bottom: `${Math.max(8, window.innerHeight - r.top + 4)}px`, left }
        : { top: `${r.bottom + 4}px`, left },
    );
  }

  function close(restoreFocus: boolean): void {
    setOpen(false);
    if (restoreFocus) button?.focus();
  }

  function toggle(): void {
    if (open()) return close(false);
    place();
    setOpen(true);
  }

  createEffect(() => {
    if (!open()) return;
    queueMicrotask(() => (popover ? focusableWithin(popover)[0] : undefined)?.focus());
    const onDown = (e: Event): void => {
      if (root && !root.contains(e.target as Node)) close(false);
    };
    const onResize = (): void => close(false);
    document.addEventListener('pointerdown', onDown, true);
    window.addEventListener('resize', onResize);
    onCleanup(() => {
      document.removeEventListener('pointerdown', onDown, true);
      window.removeEventListener('resize', onResize);
    });
  });

  function onKeyDown(e: KeyboardEvent): void {
    if (!open()) {
      if (e.target === button && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
        e.preventDefault();
        toggle();
      }
      return;
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      close(true);
      return;
    }
    if (!popover) return;
    // Arrow keys inside a select keep their own meaning.
    if (e.target instanceof HTMLSelectElement && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) return;
    const items = focusableWithin(popover);
    if (items.length === 0) return;
    const at = items.indexOf(document.activeElement as HTMLElement);
    let next: number | undefined;
    if (e.key === 'ArrowDown') next = at < 0 ? 0 : (at + 1) % items.length;
    else if (e.key === 'ArrowUp') next = at <= 0 ? items.length - 1 : at - 1;
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = items.length - 1;
    if (next === undefined) return;
    e.preventDefault();
    items[next]?.focus();
  }

  function onFocusOut(e: FocusEvent): void {
    const to = e.relatedTarget as Node | null;
    if (open() && to && root && !root.contains(to)) close(false);
  }

  return (
    <div ref={root} class={css.root} data-admin-account-menu={props.variant} onKeyDown={onKeyDown} onFocusOut={onFocusOut}>
      <button
        ref={button}
        type="button"
        class={[css.trigger, props.variant === 'icon' ? css.triggerIcon : ''].join(' ')}
        aria-haspopup="true"
        aria-expanded={open()}
        aria-controls={open() ? id : undefined}
        aria-label={label()}
        data-testid={props.testId ?? 'admin-account-button'}
        title={props.variant === 'icon' ? label() : undefined}
        onClick={toggle}
      >
        <span class={`person-avatar ${css.avatar}`} data-avatar-color={look().palette} aria-hidden="true">
          {look().initials}
        </span>
        <Show when={props.variant === 'full'}>
          <span class={css.name} aria-hidden="true">{isolate(props.username)}</span>
          <Icon name="chevronDown" />
        </Show>
      </button>
      <Show when={open()}>
        <div ref={popover} id={id} class={css.popover} role="group" aria-label={label()} style={pos()}>
          <div class={css.identity}>
            <span class={`person-avatar ${css.avatar}`} data-avatar-color={look().palette} aria-hidden="true">{look().initials}</span>
            <span class={css.identityText}>{t('admin-shell-signed-in-as', { name: isolate(props.username) })}</span>
          </div>
          <Show when={props.languagePicker}>
            <div class={css.picker}>{props.languagePicker}</div>
          </Show>
          <button type="button" class="btn btn--ghost" onClick={() => { close(false); props.onSignOut(); }}>
            <Icon name="signOut" />
            <span>{t('admin-sign-out')}</span>
          </button>
        </div>
      </Show>
    </div>
  );
}
