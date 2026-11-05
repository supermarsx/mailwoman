// The Adaptive subtab shows a line about the simulator's last run. The
// simulator lives in the policy editor, so its result is kept here for the
// session (memory only; the sample it ran with is never stored).

import { createSignal } from 'solid-js';
import type { SimResult } from '../../api/admin/requestProtection.ts';

export interface RememberedSimulation { result: SimResult; at: number }
const [last, setLast] = createSignal<RememberedSimulation | undefined>(undefined);
export const lastSimulation = last;
export function rememberSimulation(result: SimResult | undefined): void {
  setLast(result ? { result, at: Date.now() } : undefined);
}
