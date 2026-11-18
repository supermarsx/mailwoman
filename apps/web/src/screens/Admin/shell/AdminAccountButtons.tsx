// The signed-in admin as two always-visible square buttons: the user (the
// webmail's `.person-avatar`, palette derived from the name) and Sign out.
// No menu. The user button links to the admin's own preferences page when an
// `href` is given, otherwise it is a plain, inert button that only names the
// account.

import { Show, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import { avatarLook, validAvatar } from '../../../api/mailboxAvatar.ts';
import { Icon as AvatarIcon } from '../../../components/Icon.tsx';
import { useOptionalPersonalPreferences } from './personalPreferences.tsx';
import { Icon } from './icons.tsx';
import '../../../components/person-avatar.css';
import '../../../components/senderIcons/palette.css';
import * as css from './accountButtons.css.ts';

export interface AdminAccountButtonsProps {
  username: string;
  hideUser?: boolean;
  onSignOut: () => void;
  /** The user's own account/preferences page; the user button is a link when set. */
  href?: string | undefined;
  onOpenAccount?: ((e: MouseEvent) => void) | undefined;
  testId?: string | undefined;
}

export function AdminAccountButtons(props: AdminAccountButtonsProps): JSX.Element {
  const preferences = useOptionalPersonalPreferences();
  const look = () => avatarLook(validAvatar(preferences?.data()['avatar']), props.username, props.username);
  const label = () => t('admin-shell-signed-in-as', { name: isolate(props.username) });
  const avatar = (): JSX.Element => (
    <span class={`person-avatar ${css.avatar}`} data-avatar-color={look().palette} data-avatar-kind={look().custom ? look().kind : undefined} data-avatar-icon={look().icon}
      style={look().custom ? {'background-color': look().bg, color: look().fg} : {}} aria-hidden="true">
      <Show when={look().glyph} fallback={look().text}>{glyph => <AvatarIcon name={glyph()} />}</Show>
    </span>
  );

  return (
    <div class={css.row} data-admin-account>
      <Show when={!props.hideUser}><Show
        when={props.href}
        fallback={
          <button type="button" class={css.square} aria-label={label()} title={label()} data-testid={props.testId ?? 'admin-account-user'}>
            {avatar()}
          </button>
        }
      >
        {(href) => (
          <a
            class={css.square}
            href={href()}
            aria-label={label()}
            title={label()}
            data-testid={props.testId ?? 'admin-account-user'}
            onClick={(e) => props.onOpenAccount?.(e)}
          >
            {avatar()}
          </a>
        )}
      </Show></Show>
      <button
        type="button"
        class={css.square}
        aria-label={t('admin-sign-out')}
        title={t('admin-sign-out')}
        data-testid={`${props.testId ?? 'admin-account'}-sign-out`}
        onClick={() => props.onSignOut()}
      >
        <Icon name="signOut" />
      </button>
    </div>
  );
}
