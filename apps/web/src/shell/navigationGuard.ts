/** Mounted editors register only while their account-owned draft exists. */
const guards = new Set<() => boolean | Promise<boolean>>();
export function registerNavigationGuard(guard: () => boolean | Promise<boolean>): () => void {
  guards.add(guard);
  return () => { guards.delete(guard); };
}
export function confirmShellNavigation(): boolean | Promise<boolean> {
  if (guards.size === 0) return true;
  const captured = [...guards];
  return (async () => {
    for (const guard of captured) {
      if (!guards.has(guard)) continue;
      if (!await guard()) return false;
    }
    return true;
  })();
}
