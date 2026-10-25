// Account settings surface (t16 e15) — the aggregator mounted into the settings
// panel for an authenticated account. Composes the 2FA, session, signature,
// identity, notification, saved-search, and device-preference screens. Owns the
// device-preference working copy so the whole surface can mirror (dir=rtl, W20)
// and persists it on change.
//
// Mounted from `screens/Settings.tsx` alongside the existing feature modules
// (PasswordChange/RulesSettings/ApiKeys/…): `<AccountSettings />`.
//
// t32-h6: each screen sits in its own boundary. The screens read their resource
// straight in JSX and Solid rethrows a failed load on read, so one failed
// `/api/account/*` request used to reach the app's boundary and replace the
// mailbox. Now it replaces only that screen, with an error and a retry that
// mounts it again (which reloads it).

import { createEffect, createSignal, Show, type JSX } from 'solid-js';
import { useI18n } from '../../i18n';
import { AsyncBoundary } from '../../components/ErrorBoundary.tsx';
import { SettingsService } from './service.ts';
import { loadPrefs, savePrefs, type SettingsPrefs } from './prefs.ts';
import { TwoFactor } from './TwoFactor.tsx';
import { Sessions } from './Sessions.tsx';
import { Signatures } from './Signatures.tsx';
import { Identities } from './Identities.tsx';
import { Notifications } from './Notifications.tsx';
import { SavedSearches } from './SavedSearches.tsx';
import { Preferences } from './Preferences.tsx';
import * as css from './styles.css.ts';

export interface AccountSettingsProps {
  hideSecurity?: boolean;
  onSearchFoldersChanged?: () => void | Promise<void>;
  /** A workspace section; omitted keeps the legacy aggregate surface. */
  section?: () => string;
  /** Injected in tests; production uses the same-origin cookie service. */
  service?: SettingsService;
}

/** The active writing direction, tolerant of being rendered without a provider
 *  (unit tests): defaults to LTR when no `<LocaleProvider>` is present. */
function activeDir(): 'ltr' | 'rtl' {
  try {
    return useI18n().dir();
  } catch {
    return 'ltr';
  }
}

export function AccountSettings(props: AccountSettingsProps): JSX.Element {
  const selected = (section: string): boolean => !props.section || props.section() === section;
  const service = props.service ?? new SettingsService();
  const [prefs, setPrefs] = createSignal<SettingsPrefs>(loadPrefs());
  createEffect(() => setPrefs(loadPrefs()));

  const onPrefsChange = (next: SettingsPrefs): void => {
    setPrefs(next);
    savePrefs(next);
  };

  // W20: an explicit direction pref overrides the locale-derived direction; "auto"
  // follows the negotiated locale. Applied to the whole account surface so it
  // mirrors as one.
  const effectiveDir = (): 'ltr' | 'rtl' =>
    prefs().direction === 'auto' ? activeDir() : (prefs().direction as 'ltr' | 'rtl');

  return (
    <div class={css.stack} dir={effectiveDir()} data-testid="account-settings">
      <Show when={!props.hideSecurity && selected('security')}>
        <AsyncBoundary><TwoFactor service={service} /></AsyncBoundary>
        <AsyncBoundary><Sessions service={service} /></AsyncBoundary>
      </Show>
      <Show when={selected('signatures')}>
      <AsyncBoundary>
        <Signatures service={service} />
      </AsyncBoundary>
      </Show>
      <Show when={selected('identities')}>
      <AsyncBoundary>
        <Identities service={service} />
      </AsyncBoundary>
      </Show>
      <Show when={selected('notifications')}>
      <AsyncBoundary>
        <Notifications service={service} />
      </AsyncBoundary>
      </Show>
      <Show when={selected('searches')}>
      <AsyncBoundary>
        <SavedSearches service={service} {...(props.onSearchFoldersChanged ? {onChanged: props.onSearchFoldersChanged} : {})} />
      </AsyncBoundary>
      </Show>
      <Show when={selected('preferences')}>
      <AsyncBoundary>
        <Preferences prefs={prefs} onChange={onPrefsChange} />
      </AsyncBoundary>
      </Show>
    </div>
  );
}

export default AccountSettings;
