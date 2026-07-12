// In-app sub-tab strip model (plan §3 e6 sub-tab strip).
//
// A tab strip over the app's surfaces (messages / composers / settings), each
// tab pinnable, cycled by keyboard, and tear-off-able into its own window via
// `window.open` (best-effort re-attach to the SharedWorker session is the new
// window's job — §0 lists cross-window sync beyond re-attach as a cut). Pure
// reactive state so the `SubTabStrip` component is a thin view over it.

import { createSignal, type Accessor } from 'solid-js';

export type SubTabKind = 'messages' | 'composer' | 'settings';

export interface SubTab {
  id: string;
  kind: SubTabKind;
  title: string;
  pinned: boolean;
  /** Kind-specific payload (mailboxId, draftId, settings section, …). */
  data?: unknown;
}

export interface OpenTabInput {
  kind: SubTabKind;
  title: string;
  data?: unknown;
  /** Reuse an existing tab with this id instead of opening a duplicate. */
  id?: string;
  pinned?: boolean;
}

export interface SubTabsModel {
  tabs: Accessor<SubTab[]>;
  activeId: Accessor<string | null>;
  open(input: OpenTabInput): string;
  activate(id: string): void;
  close(id: string): void;
  togglePin(id: string): void;
  /** Move focus by `dir` (+1 next / -1 prev), wrapping. Returns the new id. */
  cycle(dir: 1 | -1): string | null;
  /** Pop a tab into its own window (best-effort) and close it locally. */
  tearOff(id: string): void;
}

export interface SubTabsOptions {
  /** Injectable for tests; defaults to `window.open`. */
  openWindow?: (url: string, target: string) => unknown;
  /** Builds the tear-off URL for a tab; default `?tab=<id>`. */
  tearOffUrl?: (tab: SubTab) => string;
}

let counter = 0;
function genId(): string {
  counter += 1;
  return `tab-${counter}-${Math.random().toString(36).slice(2, 8)}`;
}

export function createSubTabs(opts: SubTabsOptions = {}): SubTabsModel {
  const [tabs, setTabs] = createSignal<SubTab[]>([]);
  const [activeId, setActiveId] = createSignal<string | null>(null);
  const openWindow =
    opts.openWindow ??
    ((url: string, target: string) =>
      typeof window !== 'undefined' ? window.open(url, target) : null);
  const tearOffUrl = opts.tearOffUrl ?? ((tab: SubTab) => `?tab=${encodeURIComponent(tab.id)}`);

  function activate(id: string): void {
    if (tabs().some((t) => t.id === id)) setActiveId(id);
  }

  function open(input: OpenTabInput): string {
    if (input.id !== undefined) {
      const existing = tabs().find((t) => t.id === input.id);
      if (existing !== undefined) {
        setActiveId(existing.id);
        return existing.id;
      }
    }
    const id = input.id ?? genId();
    const tab: SubTab = {
      id,
      kind: input.kind,
      title: input.title,
      pinned: input.pinned ?? false,
      ...(input.data !== undefined ? { data: input.data } : {}),
    };
    setTabs((prev) => [...prev, tab]);
    setActiveId(id);
    return id;
  }

  function close(id: string): void {
    const list = tabs();
    const idx = list.findIndex((t) => t.id === id);
    if (idx === -1) return;
    const next = list.filter((t) => t.id !== id);
    setTabs(next);
    if (activeId() === id) {
      // Focus the neighbour (prefer the one to the left).
      const fallback = next[idx - 1] ?? next[idx] ?? null;
      setActiveId(fallback ? fallback.id : null);
    }
  }

  function togglePin(id: string): void {
    setTabs((prev) => prev.map((t) => (t.id === id ? { ...t, pinned: !t.pinned } : t)));
  }

  function cycle(dir: 1 | -1): string | null {
    const list = tabs();
    if (list.length === 0) return null;
    const cur = activeId();
    const curIdx = cur === null ? -1 : list.findIndex((t) => t.id === cur);
    const nextIdx = (curIdx + dir + list.length) % list.length;
    const next = list[nextIdx];
    if (next === undefined) return null;
    setActiveId(next.id);
    return next.id;
  }

  function tearOff(id: string): void {
    const tab = tabs().find((t) => t.id === id);
    if (tab === undefined) return;
    openWindow(tearOffUrl(tab), '_blank');
    close(id);
  }

  return { tabs, activeId, open, activate, close, togglePin, cycle, tearOff };
}
