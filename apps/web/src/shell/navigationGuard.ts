import {createContext,useContext} from 'solid-js';
export const PaneNavigationContext=createContext<string>();
/** Mounted editors register only while their account-owned draft exists. */
const guards = new Map<() => boolean | Promise<boolean>,string|undefined>();
export function registerNavigationGuard(guard: () => boolean | Promise<boolean>,scope=useContext(PaneNavigationContext)): () => void {
  guards.set(guard,scope);
  return () => { guards.delete(guard); };
}
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
