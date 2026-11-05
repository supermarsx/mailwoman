// A folder chosen from its context menu ("Retention…"): Settings > Retention
// starts a new rule for it the next time that section opens.
import { createSignal } from 'solid-js';

export const [retentionFolder, setRetentionFolder] = createSignal<string | null>(null);
