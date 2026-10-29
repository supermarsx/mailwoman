import { createContext, createSignal, useContext, type Accessor } from 'solid-js';

export interface SettingsRoute { section: string; editor?: { kind: 'new' | 'edit'; id?: string } }
export interface SettingsNavigation {
  route: Accessor<SettingsRoute>;
  navigate(route: SettingsRoute): Promise<boolean>;
  /** `discard`: the user already chose to discard unsaved changes; guards that report `dirty` do not ask again. */
  canLeave(discard?: boolean): Promise<boolean>;
  /** True when a guard that reports its state has unsaved changes (asked without prompting). */
  isDirty(): boolean;
  beforeLeave(guard: () => boolean | Promise<boolean>, dirty?: () => boolean): () => void;
}
export const SettingsNavigationContext = createContext<SettingsNavigation>();
export function useSettingsNavigation(): SettingsNavigation | undefined { return useContext(SettingsNavigationContext); }

/** Switching workspace tabs does not call navigate or unmount the editor. */
export function createSettingsNavigation(initial: SettingsRoute = { section: 'appearance' }): SettingsNavigation {
  const [route, setRoute] = createSignal<SettingsRoute>(initial);
  const guards = new Map<() => boolean | Promise<boolean>, (() => boolean) | undefined>();
  let pending: Promise<boolean> | undefined;
  let navigation = 0;
  async function check(discard: boolean): Promise<boolean> {
    for (const [guard, dirty] of [...guards]) if (!(discard && dirty) && !await guard()) return false;
    return true;
  }
  function canLeave(discard = false): Promise<boolean> {
    if (discard) return check(true);
    pending ??= check(false).finally(() => { pending = undefined; });
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
    isDirty: () => [...guards.values()].some(dirty => dirty?.() === true),
    beforeLeave(guard, dirty) { guards.set(guard, dirty); return () => { guards.delete(guard); }; },
  };
}
