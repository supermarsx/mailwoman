import { afterEach, expect, it, vi } from 'vitest';
import { activateAccountPreferences, observePreferenceAccount, preferenceAccount, preferenceAccountIsCurrent, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
afterEach(suspendAccountPreferences);
it('invalidates a pending response before the authentication cookie can change', () => {
  activateAccountPreferences('first');
  const old = preferenceAccount()!;
  suspendAccountPreferences();
  expect(preferenceAccountIsCurrent(old)).toBe(false);
  activateAccountPreferences('second');
  expect(preferenceAccountIsCurrent(old)).toBe(false);
  expect(preferenceAccount()?.id).toBe('second');
});
it('does not resurrect a response after returning to the same account', () => {
  activateAccountPreferences('same');
  const old = preferenceAccount()!;
  suspendAccountPreferences();
  activateAccountPreferences('same');
  expect(preferenceAccountIsCurrent(old)).toBe(false);
});
it('notifies suspension synchronously and permits disposal', () => {
  const callback = vi.fn();
  const dispose = observePreferenceAccount(callback);
  activateAccountPreferences('account');
  suspendAccountPreferences();
  expect(callback.mock.calls.at(-1)?.[0]).toBeNull();
  const calls = callback.mock.calls.length;
  dispose();
  activateAccountPreferences('other');
  expect(callback).toHaveBeenCalledTimes(calls);
});
