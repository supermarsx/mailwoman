// t94: the user's Focused inbox overrides, loaded from the server for the
// signed-in mailbox and reloaded when the mailbox changes. Before t94 they
// lived in this browser's localStorage (`mw.focused.v1`, written by the
// never-shown "train sender" action): the first successful load moves any such
// entries to the server, rows already there win, and the local copy is then
// removed. A failed import keeps the local copy for the next load.
import {createSignal, getOwner, onCleanup, type Accessor} from 'solid-js';
import {observePreferenceAccount, preferenceAccount, type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {importFocusedSenders, loadFocusedSenders, saveFocusedSender, type FocusTab, type FocusedSender} from '../api/focusedSenders.ts';

export const LEGACY_FOCUS_KEY = 'mw.focused.v1';

export interface FocusedSendersApi {
  load(): Promise<FocusedSender[]>;
  save(email: string, tab: FocusTab | null): Promise<void>;
  import(senders: Record<string, FocusTab>): Promise<FocusedSender[]>;
}

const serverApi: FocusedSendersApi = {load: () => loadFocusedSenders(), save: saveFocusedSender, import: importFocusedSenders};

export interface FocusedSenders {
  /** Lower-cased sender address -> the tab the user chose for it. */
  overrides: Accessor<Record<string, FocusTab>>;
  /** The rows with their write times, for the Settings list. */
  rows: Accessor<FocusedSender[]>;
  /** Set (`tab`) or remove (`null`) one sender's override. Applied at once;
   *  reverted, and the promise rejected, when the server refuses it. */
  set(email: string, tab: FocusTab | null): Promise<void>;
  reload(): Promise<void>;
}

/** The old local overrides, valid entries only. */
export function readLegacyOverrides(): Record<string, FocusTab> {
  try {
    const raw = globalThis.localStorage?.getItem(LEGACY_FOCUS_KEY);
    if (raw == null) return {};
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return {};
    const out: Record<string, FocusTab> = {};
    for (const [email, tab] of Object.entries(parsed as Record<string, unknown>)) {
      if ((tab === 'focused' || tab === 'other') && email.includes('@')) out[email.trim().toLowerCase()] = tab;
    }
    return out;
  } catch {
    return {};
  }
}

function clearLegacy(): void {
  try {globalThis.localStorage?.removeItem(LEGACY_FOCUS_KEY);} catch {/* storage unavailable */}
}

export function createFocusedSenders(api: FocusedSendersApi = serverApi): FocusedSenders {
  const [rows, setRowsSignal] = createSignal<FocusedSender[]>([]);
  const [overrides, setOverrides] = createSignal<Record<string, FocusTab>>({});
  function setRows(next: FocusedSender[]): void {
    setRowsSignal(next);
    setOverrides(Object.fromEntries(next.map(row => [row.email, row.tab])));
  }
  let generation = 0;
  let owner: PreferenceAccount | null = preferenceAccount();

  async function reload(): Promise<void> {
    const account = preferenceAccount();
    const mine = ++generation;
    if (!account) {setRows([]); return;}
    let next = await api.load();
    const legacy = readLegacyOverrides();
    if (Object.keys(legacy).length > 0) {
      // Malformed or corrupt storage was already dropped; anything left moves.
      try {next = await api.import(legacy); clearLegacy();} catch {/* kept for the next load */}
    } else {
      clearLegacy();
    }
    if (mine === generation && account === preferenceAccount()) setRows(next);
  }

  async function set(email: string, tab: FocusTab | null): Promise<void> {
    const key = email.trim().toLowerCase();
    const account = preferenceAccount();
    const before = rows();
    const rest = before.filter(row => row.email !== key);
    setRows(tab ? [...rest, {email: key, tab, updatedAt: Date.now()}].sort((a, b) => a.email.localeCompare(b.email)) : rest);
    try {
      await api.save(key, tab);
    } catch (error) {
      if (account === preferenceAccount()) {
        const previous = before.find(row => row.email === key);
        const now = rows().filter(row => row.email !== key);
        setRows(previous ? [...now, previous].sort((a, b) => a.email.localeCompare(b.email)) : now);
      }
      throw error;
    }
  }

  const stop = observePreferenceAccount(next => {
    if (next === owner) return;
    owner = next;
    setRows([]);
    void reload().catch(() => undefined);
  });
  if (getOwner()) onCleanup(stop);
  if (owner) void reload().catch(() => undefined);
  return {overrides, rows, set, reload};
}
