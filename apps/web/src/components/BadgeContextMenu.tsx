import type {JSX} from 'solid-js';
import {createContextMenu, type ContextAction} from './ContextMenu.tsx';
import {settings, writeSetting} from '../api/settings.ts';
import {badgeKey, badgeLocked, badgeMode, isBadgeMode, type BadgeId} from '../state/readerBadges.ts';
import {t} from '../i18n';

/** Right-click (or the Menu key) on a header badge: hide it, or show it only when notable. Writes the user's setting; an admin lock leaves the items disabled. */
export function BadgeContextMenu(props: {badge: BadgeId; sent?: boolean; children: JSX.Element}): JSX.Element {
  const sentClassification=()=>props.sent===true&&(props.badge==='auth'||props.badge==='spam');
  const key=()=>badgeKey(props.badge)+(sentClassification()?'.sent':'');
  const mode=()=>{const value=settings()[key()]?.value;return sentClassification()?(isBadgeMode(value)?value:'never'):badgeMode(props.badge);};
  const set = (mode: 'never' | 'notable'): void => { void writeSetting(key(), mode).catch(() => undefined); };
  const context = createContextMenu({
    label: () => t('reader-badge-menu'),
    scope: key,
    actions: (): ContextAction[] => {
      const locked = sentClassification()?settings()[key()]?.locked===true:badgeLocked(props.badge), title = locked ? t('reader-badge-locked') : undefined;
      return [
        {id: 'notable', label: t('reader-badge-notable'), icon: 'eye', disabled: locked || mode() === 'notable', ...(title ? {title} : {}), run: () => set('notable')},
        {id: 'hide', label: t('reader-badge-hide'), icon: 'eyeOff', disabled: locked, ...(title ? {title} : {}), run: () => set('never')},
      ];
    },
  });
  return <span class="mw-badge-host" onContextMenu={context.open} onKeyDown={context.keydown}>{props.children}{context.menu()}</span>;
}
