import { preferenceAccount } from './accountPreferenceLifecycle.ts';

/** Bind a private-settings request to the verified account captured by the UI. */
export function storageAuthorityHeaders(expected?: string): Record<string, string> {
  const authority = preferenceAccount()?.storageAuthority;
  if (!authority || (expected !== undefined && expected !== authority)) throw new Error('The signed-in account changed. Reload these settings.');
  return {'x-mailwoman-storage-authority': authority};
}
