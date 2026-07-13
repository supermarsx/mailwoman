// Capability activation gate (plan §3 e6, risk R2).
//
// The V5 consumer wiring (native new-mail notifications, unread badge, push
// subscribe, deep links) is ADDITIVE and must never perturb a plain browser —
// the hard regression gate. So each consumer is gated here:
//   * inside a Tauri shell (`isTauri()`) every capability is ON;
//   * in a browser it is OFF unless the deployment opts in via the injected
//     `__MW_CONFIG__.capabilities` (a boolean turns them all on, or an object
//     enables named flags). A test harness / a browser that injects nothing gets
//     the pre-V5 behaviour byte-for-byte.
//
// The browser FALLBACKS themselves (browser.ts) are always present and unit
// tested directly; this only governs whether the SPA drives them passively.

import { isTauri } from './index.ts';

export type CapabilityFlag = 'notifications' | 'push' | 'deepLinks';

/** Is the passive consumer wiring for `flag` active in this runtime? */
export function capabilityEnabled(flag: CapabilityFlag): boolean {
  if (isTauri()) return true;
  const caps = (globalThis as { __MW_CONFIG__?: { capabilities?: unknown } }).__MW_CONFIG__
    ?.capabilities;
  if (caps === true) return true;
  if (typeof caps === 'object' && caps !== null) {
    return (caps as Record<string, unknown>)[flag] === true;
  }
  return false;
}
