// Where a setting's value comes from (t32-cfg UX9, DC2).
//
// One badge per field: `default` · `stored` · `env pin (MW_X)` · `env fallback
// (MW_X)` · `build` · `locked by <scope>` · `read-only`, plus `restart needed` when
// a stored value only applies after a restart. A pinned, build-time, locked or
// read-only value cannot be changed from the console: `sourceBlocksEdit` says so,
// the field is rendered disabled with `sourceReason` shown under it, and its
// `aria-describedby` points at that reason so it is read with the control.
//
// Replaces `PinnedBadge` over time; `PinnedBadge` uses map to `{kind: 'env-pin'}`.

import { Show, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import type { EffectiveEntry, EffectiveSource, RegistryEntry } from './adminApi.ts';
import { Icon } from './icons.tsx';
import * as ux from './ux.css.ts';

export type SourceKind = 'default' | 'stored' | 'env-pin' | 'env-fallback' | 'build' | 'locked' | 'read-only';

export interface SourceInfo {
  kind: SourceKind;
  /** The environment variable, for `env-pin` and `env-fallback`. */
  envVar?: string;
  /** The scope that locks the value (`global`, a domain), for `locked`. */
  lockedBy?: string;
  /** The value applies only after a restart (boot-time knob, or a stored change to one). */
  restartNeeded?: boolean;
}

/** Map an effective-configuration row (e70a) to a badge. */
export function sourceFromEffective(e: Pick<EffectiveEntry, 'source' | 'envVar' | 'restartNeeded' | 'locked' | 'class'>): SourceInfo {
  const kinds: Record<EffectiveSource, SourceKind> = {
    default: 'default',
    stored: 'stored',
    'env-pin': 'env-pin',
    'env-fallback': 'env-fallback',
    boot: 'env-pin',
    build: 'build',
    refused: 'read-only',
  };
  let kind = kinds[e.source] ?? 'read-only';
  const lockedBy = typeof e.locked === 'string' ? e.locked : e.locked === true ? 'global' : undefined;
  if (lockedBy !== undefined && (kind === 'default' || kind === 'stored')) kind = 'locked';
  if (e.class !== undefined && e.class !== 'Ui' && e.class !== 'ui' && kind !== 'build' && kind !== 'env-pin') kind = 'read-only';
  return {
    kind,
    ...(e.envVar !== undefined ? { envVar: e.envVar } : {}),
    ...(lockedBy !== undefined ? { lockedBy } : {}),
    restartNeeded: e.restartNeeded === true || e.source === 'boot',
  };
}

/** Map a registry entry (e10b) to a badge. */
export function sourceFromRegistry(e: Pick<RegistryEntry, 'envVar' | 'precedence' | 'locked' | 'globalValue'>): SourceInfo {
  if (e.envVar !== undefined && e.precedence === 'pin') return { kind: 'env-pin', envVar: e.envVar };
  if (e.envVar !== undefined && e.precedence === 'boot') return { kind: 'env-pin', envVar: e.envVar, restartNeeded: true };
  if (e.locked) return { kind: 'locked', lockedBy: 'global' };
  const stored = e.globalValue !== undefined && e.globalValue !== null;
  if (!stored && e.envVar !== undefined && e.precedence === 'fallback') return { kind: 'env-fallback', envVar: e.envVar };
  return { kind: stored ? 'stored' : 'default' };
}

/** True when the console cannot change a value from this source. */
export function sourceBlocksEdit(s: SourceInfo | undefined): boolean {
  return s !== undefined && (s.kind === 'env-pin' || s.kind === 'build' || s.kind === 'locked' || s.kind === 'read-only');
}

/** The short badge text. */
export function sourceLabel(s: SourceInfo): string {
  switch (s.kind) {
    case 'default':
      return t('admin-ux-source-default');
    case 'stored':
      return t('admin-ux-source-stored');
    case 'env-pin':
      return t('admin-ux-source-env-pin', { name: isolate(s.envVar ?? '') });
    case 'env-fallback':
      return t('admin-ux-source-env-fallback', { name: isolate(s.envVar ?? '') });
    case 'build':
      return t('admin-ux-source-build');
    case 'locked':
      return t('admin-ux-source-locked', { scope: isolate(s.lockedBy ?? '') });
    case 'read-only':
      return t('admin-ux-source-read-only');
  }
}

/** The sentence that explains a disabled field, or null when the field is editable. */
export function sourceReason(s: SourceInfo): string | null {
  switch (s.kind) {
    case 'env-pin':
      return t('admin-ux-source-env-pin-reason', { name: isolate(s.envVar ?? '') });
    case 'build':
      return t('admin-ux-source-build-reason');
    case 'locked':
      return t('admin-ux-source-locked-reason', { scope: isolate(s.lockedBy ?? '') });
    case 'read-only':
      return t('admin-ux-source-read-only-reason');
    default:
      return null;
  }
}

const KIND_ICON: Record<SourceKind, 'check' | 'info' | 'alert' | 'settings'> = {
  default: 'check',
  stored: 'settings',
  'env-pin': 'alert',
  'env-fallback': 'info',
  build: 'info',
  locked: 'alert',
  'read-only': 'info',
};

export interface SourceBadgeProps {
  source: SourceInfo;
  /** Id of the reason text, for the control's `aria-describedby`. */
  id?: string;
}

export function SourceBadge(props: SourceBadgeProps): JSX.Element {
  return (
    <span class={ux.badges} id={props.id} data-testid="source-badge" data-source={props.source.kind}>
      <span class={ux.badge} data-kind={props.source.kind}>
        <Icon name={KIND_ICON[props.source.kind]} />
        <span>{sourceLabel(props.source)}</span>
      </span>
      <Show when={props.source.restartNeeded}>
        <span class={ux.badge} data-kind="restart">
          <Icon name="alert" />
          <span>{t('admin-ux-source-restart')}</span>
        </span>
      </Show>
    </span>
  );
}
