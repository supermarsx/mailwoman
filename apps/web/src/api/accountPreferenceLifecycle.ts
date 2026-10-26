/** Authority boundary shared by account preference transports. Call suspend before
 * a request may replace the authentication cookie, not after its response. */
export interface PreferenceAccount { readonly id: string; readonly generation: number; readonly storageAuthority?: string }
type Listener = (account: PreferenceAccount | null) => void;
let generation = 0;
let current: PreferenceAccount | null = null;
const listeners = new Set<Listener>();

export function preferenceAccount(): PreferenceAccount | null { return current; }
export function preferenceAccountIsCurrent(account: PreferenceAccount): boolean {
  return current?.generation === account.generation && current.id === account.id;
}
export function suspendAccountPreferences(): void {
  generation += 1;
  current = null;
  for (const listener of listeners) listener(null);
}
/** Only a server-authenticated effective account ID may enter here. */
export function activateAccountPreferences(id: string, storageAuthority?:string): void {
  if (!id) { suspendAccountPreferences(); return; }
  const authority=typeof storageAuthority==='string'&&/^[A-Za-z0-9_-]{16,256}$/.test(storageAuthority)?storageAuthority:undefined;
  if (current?.id === id && current.storageAuthority===authority) return;
  current = Object.freeze({ id, generation: ++generation,...(authority?{storageAuthority:authority}:{}) });
  for (const listener of listeners) listener(current);
}
export function observePreferenceAccount(listener: Listener): () => void {
  listeners.add(listener);
  listener(current);
  return () => { listeners.delete(listener); };
}
