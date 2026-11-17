import {Show,type JSX} from 'solid-js';
import {Icon} from '../Icon.tsx';
import {avatarLook,type MailboxAvatarSpec} from '../../api/mailboxAvatar.ts';
import * as css from './accountIndicator.css.ts';
import '../person-avatar.css';
import '../senderIcons/palette.css';

export function MailboxAvatar(props: { label: string; address: string; size?: 'sm' | 'md' | 'lg' | 'message'; avatar?: MailboxAvatarSpec | undefined }): JSX.Element {
  const look = (): ReturnType<typeof avatarLook> => avatarLook(props.avatar, props.label, props.address);
  return (
    <span
      class={`person-avatar ${css.avatar}`}
      data-size={props.size ?? 'md'}
      data-avatar-color={look().palette}
      data-avatar-kind={look().custom ? look().kind : undefined}
      data-avatar-icon={look().icon}
      style={look().custom ? { 'background-color': look().bg, color: look().fg } : {}}
      aria-hidden="true"
    >
      <Show when={look().glyph} fallback={look().text}>{(glyph) => <Icon name={glyph()} />}</Show>
    </span>
  );
}
