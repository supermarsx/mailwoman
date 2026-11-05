// The HTTP protocol guard's classes and settings, grouped once for the Protocol
// guard subtab (editors) and the Overview (counters). Server: protocol_guard.rs;
// the registry keys live under `security.protocol.*`.

import type { ProtocolGuardRow } from '../../api/admin/requestProtection.ts';

export type GuardGroupId = 'framing' | 'host' | 'headers' | 'limits' | 'methods';
export type GuardMode = 'off' | 'detect' | 'enforce';

export interface GuardGroup {
  id: GuardGroupId;
  /** Counter class names (the status `class` field); each has a `<class>Mode` key. */
  classes: readonly string[];
  /** Every registry key shown for the group, in order. */
  keys: readonly string[];
}

const key = (name: string): string => `security.protocol.${name}`;

export const GUARD_GROUPS: readonly GuardGroup[] = [
  { id: 'framing', classes: ['framing', 'body'], keys: [key('framingMode'), key('bodyMode')] },
  {
    id: 'host',
    classes: ['host', 'hostPinning', 'pathControl', 'target'],
    keys: [key('hostMode'), key('hostPinningMode'), key('allowedHosts'), key('pathControlMode'), key('targetMode')],
  },
  { id: 'headers', classes: ['duplicateHeaders', 'forwarded', 'headerHygiene'], keys: [key('duplicateHeadersMode'), key('forwardedMode'), key('headerHygieneMode')] },
  {
    id: 'limits',
    classes: ['pathLength'],
    keys: [key('pathLengthMode'), key('maxPathBytes'), key('maxQueryBytes'), key('maxHeaderCount'), key('maxHeaderValueBytes'), key('maxCookieBytes'), key('maxBodyBytes')],
  },
  { id: 'methods', classes: ['method'], keys: [key('methodMode'), key('allowedMethods'), key('exemptPrefixes')] },
];

export const MODE_KEYS: readonly string[] = GUARD_GROUPS.flatMap((g) => g.classes).map((c) => key(`${c}Mode`));
export const ALL_GUARD_KEYS: readonly string[] = GUARD_GROUPS.flatMap((g) => g.keys);
export const GUARD_MODES: readonly GuardMode[] = ['off', 'detect', 'enforce'];

export const classOfKey = (settingKey: string): string | null =>
  settingKey.endsWith('Mode') && settingKey.startsWith('security.protocol.') ? settingKey.slice('security.protocol.'.length, -'Mode'.length) : null;

export interface GroupTotals {
  detected: number;
  enforced: number;
  lastSeen: number | null;
}
export function totalsOf(rows: readonly ProtocolGuardRow[]): GroupTotals {
  let lastSeen: number | null = null;
  let detected = 0;
  let enforced = 0;
  for (const r of rows) {
    detected += r.detected;
    enforced += r.enforced;
    if (r.lastSeen !== null && (lastSeen === null || r.lastSeen > lastSeen)) lastSeen = r.lastSeen;
  }
  return { detected, enforced, lastSeen };
}

/** One mode when every class agrees, else `mixed`. */
export function overallMode(modes: readonly (string | undefined)[]): GuardMode | 'mixed' {
  const first = modes[0];
  if (modes.length === 0 || !modes.every((m) => m === first)) return 'mixed';
  return first === 'off' || first === 'detect' || first === 'enforce' ? first : 'mixed';
}

/** Detections in classes that only detect: what enforcing would have refused since start. */
export function wouldRefuse(rows: readonly ProtocolGuardRow[]): number {
  return rows.filter((r) => r.mode === 'detect').reduce((sum, r) => sum + r.detected, 0);
}

/** Highest count first; ties keep the declared order. */
export function byCount(rows: readonly ProtocolGuardRow[]): ProtocolGuardRow[] {
  return rows
    .map((r, i) => ({ r, i }))
    .sort((a, b) => b.r.detected + b.r.enforced - (a.r.detected + a.r.enforced) || a.i - b.i)
    .map((x) => x.r);
}

/** Every class sits in exactly one group and every `<class>Mode` key is listed in its group. */
export function groupsCover(): boolean {
  const classes = GUARD_GROUPS.flatMap((g) => g.classes);
  return new Set(classes).size === classes.length && GUARD_GROUPS.every((g) => g.classes.every((c) => g.keys.includes(`security.protocol.${c}Mode`)));
}
