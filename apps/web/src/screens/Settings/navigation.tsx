import { createContext, createSignal, useContext, type Accessor } from 'solid-js';

export interface SettingsRoute { section: string; editor?: { kind: 'new' | 'edit'; id?: string } }
export interface SettingsNavigation {
  route: Accessor<SettingsRoute>;
  navigate(route: SettingsRoute): Promise<boolean>;
  canLeave(): Promise<boolean>;
  beforeLeave(guard: () => boolean | Promise<boolean>): () => void;
}
export const SettingsNavigationContext = createContext<SettingsNavigation>();
export function useSettingsNavigation(): SettingsNavigation | undefined { return useContext(SettingsNavigationContext); }

/** Switching workspace tabs does not call navigate or unmount the editor. */
export function createSettingsNavigation(initial: SettingsRoute = { section: 'appearance' }): SettingsNavigation {
  const [route, setRoute] = createSignal<SettingsRoute>(initial);
  const guards = new Set<() => boolean | Promise<boolean>>();
  let pending: Promise<boolean> | undefined;
  let navigation = 0;
  async function check(): Promise<boolean> {
    for (const guard of [...guards]) if (!await guard()) return false;
    return true;
  }
  function canLeave(): Promise<boolean> {
    pending ??= check().finally(() => { pending = undefined; });
    return pending;
  }
  return {
    route,
    async navigate(next) {
      const generation = ++navigation;
      if (!await canLeave() || generation !== navigation) return false;
      setRoute({ ...next, ...(next.editor ? { editor: { ...next.editor } } : {}) });
      return true;
    },
    canLeave,
    beforeLeave(guard) { guards.add(guard); return () => { guards.delete(guard); }; },
  };
}
