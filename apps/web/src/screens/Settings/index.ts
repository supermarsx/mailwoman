// Account-settings surface public API (t16 e15, SPEC §7.4/§19 + W12/W13/W14/W15/W16/W20).
//
// Mounted into `screens/Settings.tsx` for an authenticated account:
//   import { AccountSettings } from './Settings/index.ts';
//   <AccountSettings />
//
// `TwoFactorChallenge` is exported for the LOGIN owner to mount when `/api/login`
// answers `twofaRequired` (S1 login step) — kept out of the login file to respect
// the ownership boundary.

export { AccountSettings, type AccountSettingsProps } from './AccountSettings.tsx';
export { TwoFactor, type TwoFactorProps } from './TwoFactor.tsx';
export { TwoFactorChallenge, type TwoFactorChallengeProps } from './TwoFactorChallenge.tsx';
export { Sessions, type SessionsProps } from './Sessions.tsx';
export { Signatures, type SignaturesProps } from './Signatures.tsx';
export { Identities, type IdentitiesProps } from './Identities.tsx';
export { Notifications, type NotificationsProps } from './Notifications.tsx';
export { SavedSearches, type SavedSearchesProps } from './SavedSearches.tsx';
export { Preferences, type PreferencesProps } from './Preferences.tsx';
export { SettingsService, SettingsError, type Fetcher } from './service.ts';
export {
  loadPrefs,
  savePrefs,
  DEFAULT_PREFS,
  PRESET_BINDINGS,
  type SettingsPrefs,
  type KeyboardPreset,
  type EvictionStrategy,
  type DirectionPref,
} from './prefs.ts';
export * from './types.ts';
export {
  passkeySupported,
  registerPasskey,
  assertPasskey,
  type RegistrationResult,
  type AssertionResult,
} from './webauthn.ts';
