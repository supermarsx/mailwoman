import type {JSX} from 'solid-js';
import {createContextMenu, type ContextAction} from './ContextMenu.tsx';
import {writeSetting} from '../api/settings.ts';
import {badgeKey, badgeLocked, badgeMode, type BadgeId} from '../state/readerBadges.ts';
import {t} from '../i18n';

/** Right-click (or the Menu key) on a header badge: hide it, or show it only when notable. Writes the user's setting; an admin lock leaves the items disabled. */
export function BadgeContextMenu(props: {badge: BadgeId; children: JSX.Element}): JSX.Element {
  const set = (mode: 'never' | 'notable'): void => { void writeSetting(badgeKey(props.badge), mode).catch(() => undefined); };
  const context = createContextMenu({
    label: () => t('reader-badge-menu'),
    scope: () => props.badge,
    actions: (): ContextAction[] => {
      const locked = badgeLocked(props.badge), title = locked ? t('reader-badge-locked') : undefined;
      return [
        {id: 'notable', label: t('reader-badge-notable'), icon: 'eye', disabled: locked || badgeMode(props.badge) === 'notable', ...(title ? {title} : {}), run: () => set('notable')},
        {id: 'hide', label: t('reader-badge-hide'), icon: 'eyeOff', disabled: locked, ...(title ? {title} : {}), run: () => set('never')},
      ];
    },
  });
  return <span class="mw-badge-host" onContextMenu={context.open} onKeyDown={context.keydown}>{props.children}{context.menu()}</span>;
}
