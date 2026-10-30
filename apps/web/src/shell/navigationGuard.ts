import {createContext,useContext} from 'solid-js';
export const PaneNavigationContext=createContext<string>();
/** Mounted editors register only while their account-owned draft exists. */
const guards = new Map<() => boolean | Promise<boolean>,string|undefined>();
export function registerNavigationGuard(guard: () => boolean | Promise<boolean>,scope=useContext(PaneNavigationContext)): () => void {
  guards.set(guard,scope);
  return () => { guards.delete(guard); };
}
/** Whether any mounted editor holds unsaved work (t102: no automatic reload then). */
export function hasNavigationGuards(): boolean { return guards.size > 0; }
export function confirmShellNavigation(scope?:string): boolean | Promise<boolean> {
  if (guards.size === 0) return true;
  const captured = [...guards].filter(([,owner])=>scope===undefined||owner===scope).map(([guard])=>guard);
  return (async () => {
    for (const guard of captured) {
      if (!guards.has(guard)) continue;
      if (!await guard()) return false;
    }
    return true;
  })();
}
