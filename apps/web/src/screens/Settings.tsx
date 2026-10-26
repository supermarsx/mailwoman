import { NextcloudAccounts } from './Settings/NextcloudAccounts.tsx';
import { MailTransfer } from './Settings/MailTransfer.tsx';
import { Receipts } from './Settings/Receipts.tsx';
import { FolderPreferences } from './Settings/FolderPreferences.tsx';
import { createSettingsNavigation, SettingsNavigationContext, type SettingsNavigation } from './Settings/navigation.tsx';
import * as workspace from './Settings/workspace.css.ts';
import { Icon } from '../components/Icon.tsx';
// Settings panel (plan §3 e4): the user-facing theme / density / accent / font /
// layout controls that drive the theme slice. Rendered as a dismissible dialog;
// every control writes straight through the slice, which reflects onto :root and
// persists to localStorage (V2). Token-native styling (styles/settings.css.ts).
//
// t19 e13 replaced the flat theme button row with a GALLERY over the theme
// registry (t19 e6): one group per pack family, each entry previewing its own
// palette, plus the `fixed`/`system`/`schedule` mode tri-state, the light/dark
// pair the two automatic modes switch between, and the schedule window. The same
// lane wired the per-account server sync (SPEC §17.3) — `api/prefs.ts` — which is
// started here as a fallback for app builds that do not start it at boot.
//
// t32 settings model C (worker f01): an administrator can hold the theme, the
// accent and the density (`appearance.*` keys, global, per domain or per user).
// A held control is disabled, says "Set by your administrator" (LockNote), and
// the held value is applied here and by the server on every appearance write.
// The language picker (`Settings/Language.tsx`) writes `ui.language`.

import { createEffect, createSignal, For, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, loadCatalog } from '../i18n';
import { createFocusTrap } from '../components/a11y';
// V6 (plan §3 e8/e11): the zero-access storage, scoped API-key, and MCP-key
// sections — additive, rendered only for an authenticated account. The normal
// appearance controls above are byte-unchanged.
import { ZeroAccessSettings, spawnZeroAccessWorker } from '../modules/zeroaccess/index.ts';
import { ApiKeys, McpKeys } from '../modules/apikeys/index.ts';
// V7 (plan §3 e14): in-app password change (SPEC §18.3). Lazily importable module;
// mounted into the authenticated settings block.
import { PasswordChange } from '../modules/passwd/index.ts';
// t12 (audit #1, SPEC §6.1/§10.5): mail rules/filters — condition/action builder,
// raw-Sieve editor, where-it-runs indicator, and dry-run. Self-contained module;
// rides the existing MailRule JMAP + server Sieve codegen/PUTSCRIPT path.
import { RulesSettings } from '../modules/rules/index.ts';
// t13 (26.13, E9 mount): the server-level METADATA (RFC 5464) view. Self-contained
// module; rides the same JMAP surface as `SecurityVerdict/get`. Mounted read-only
// here — editing server annotations is an admin concern (HUMAN-DECISION flag 3,
// METADATA scope, is still open), and the plain settings surface resolves no admin
// right, so `canEdit` stays false (honest: read-only unless the permission is known).
import { MetadataView } from '../modules/servermeta/index.ts';
// t16 (26.16, e15): the account-settings surface — 2FA enrolment/verification,
// active sessions, signatures, identities, notification rules, saved-search
// folders, and device preferences. Self-contained; mounted for an authenticated
// account alongside the existing feature modules.
import { AccountSettings } from './Settings/index.ts';
import { Language } from './Settings/Language.tsx';
import { SecurityCenter } from './Settings/SecurityCenter.tsx';
import { Privacy } from './Settings/Privacy.tsx';
import { AnalyticsConsent } from './Settings/AnalyticsConsent.tsx';
import { ComposePreferences } from './Settings/ComposePreferences.tsx';
import { KeyboardShortcuts } from './Settings/KeyboardShortcuts.tsx';
import { TimeZones } from './Settings/TimeZones.tsx';
import { Packages } from './Settings/Packages.tsx';
import { Labels } from './Settings/Labels.tsx';
import {ContactAppearance} from './Settings/ContactAppearance.tsx';
import { ReaderAppearanceSettings } from './Settings/ReaderAppearance.tsx';
import { LockNote } from './Settings/LockNote.tsx';
import { useSetting } from './Settings/useSetting.ts';
import { AsyncBoundary } from '../components/ErrorBoundary.tsx';
import { createAclClient } from '../api/acl-types.ts';
import { createConfiguredClient } from '../api/transport.ts';
import { startAppearanceSync, type SyncStatus } from '../api/prefs.ts';
import {
  ACCENT_LIST,
  accentShade,
  isThemeName,
  THEME_DEFAULT_ACCENT_LABEL_ID,
  THEME_GROUPS,
  themesByAppearance,
  type ThemeEntry,
} from '../theme/registry.ts';
import type { Density } from '../theme/contract.css.ts';
import type { LayoutMode, ThemeMode, UiFont } from '../state/slices/theme.ts';
import * as css from '../styles/settings.css.ts';
import * as picker from '../theme/picker.css.ts';

// Option labels are Fluent ids resolved through `t()` at render (reactive).
const MODE_OPTIONS: ReadonlyArray<{ value: ThemeMode; label: string }> = [
  { value: 'fixed', label: 'appearance-mode-fixed' },
  { value: 'system', label: 'appearance-mode-system' },
  { value: 'schedule', label: 'appearance-mode-schedule' },
];

const MODE_HINTS: Record<ThemeMode, string> = {
  fixed: 'appearance-mode-hint-fixed',
  system: 'appearance-mode-hint-system',
  schedule: 'appearance-mode-hint-schedule',
};

const SYNC_MESSAGES: Record<SyncStatus, string> = {
  idle: 'appearance-sync-idle',
  loading: 'appearance-sync-loading',
  synced: 'appearance-sync-synced',
  'local-only': 'appearance-sync-local-only',
  error: 'appearance-sync-error',
};

const DENSITY_OPTIONS: ReadonlyArray<{ value: Density; label: string }> = [
  { value: 'compact', label: 'settings-density-compact' },
  { value: 'cozy', label: 'settings-density-cozy' },
  { value: 'relaxed', label: 'settings-density-relaxed' },
];

const FONT_OPTIONS: ReadonlyArray<{ value: UiFont; label: string }> = [
  { value: 'default', label: 'settings-font-default' },
  { value: 'system', label: 'settings-font-system' },
  { value: 'serif', label: 'settings-font-serif' },
  { value: 'mono', label: 'settings-font-mono' },
  ...(['inter', 'roboto', 'open-sans', 'jost'] as const).map(value => ({value, label: `settings-font-${value}`})),
];

const LAYOUT_OPTIONS: ReadonlyArray<{ value: LayoutMode; label: string }> = [
  { value: 'default', label: 'settings-layout-default' },
  { value: 'ribbon', label: 'settings-layout-ribbon' },
];

export interface SettingsProps {
  onClose: () => void;
  embedded?: boolean;
  navigation?: SettingsNavigation;
}

export function Settings(props: SettingsProps): JSX.Element {
  const app = useApp();
  const navigation=props.navigation ?? createSettingsNavigation();
  const visible=(section:string):boolean=>!props.embedded || navigation.route().section===section;
  const sections=[['appearance','settings-appearance'],['reader','reader-settings-title'],['compose','settings-compose-title'],['shortcuts','settings-kbd-title'],['time','settings-time-title'],['packages','packages-title'],['labels','labels-title'],['security','security-center-title'],['privacy','privacy-title'],['signatures','settings-sig-title'],['identities','settings-ident-title'],['notifications','settings-notif-title'],['searches','settings-search-title'],['folders','settings-folders-title'],['mail-transfer','mail-transfer-title'],['receipts','receipts-title'],['nextcloud','nextcloud-account-title'],['preferences','settings-nav-preferences'],['rules','settings-nav-rules'],['access','settings-nav-access'],['metadata','settings-nav-metadata']] as const;

  let panel!: HTMLElement;
  onMount(() => {
    void loadCatalog('settings');
    void loadCatalog('mail-transfer');
    void loadCatalog('receipts');
    void loadCatalog('packages');
    void loadCatalog('labels');
    void loadCatalog('appearance');
    void loadCatalog('settings-policy');
    void loadCatalog('nextcloud'); void loadCatalog('reader'); void loadCatalog('privacy'); void loadCatalog('security');
  });
  const themeHold = useSetting('appearance.theme', '');
  const accentHold = useSetting('appearance.accent', '');
  const densityHold = useSetting('appearance.density', '');
  const fontHold = useSetting('appearance.font', 'default');
  createEffect(() => {
    const value = fontHold.value();
    if (fontHold.locked() && app.uiFont() !== value && FONT_OPTIONS.some(option => option.value === value)) {
      app.setUiFont(value as UiFont);
    }
  });
  // A held value is the one in force: apply it to this device as well.
  createEffect(() => {
    const v = themeHold.value();
    if (themeHold.locked() && isThemeName(v) && (app.theme() !== v || app.themeMode() !== 'fixed')) {
      app.setTheme(v);
    }
  });
  createEffect(() => {
    const v = accentHold.value();
    if (accentHold.locked() && app.accentId() !== v && (v === '' || ACCENT_LIST.some((a) => a.id === v))) {
      app.setAccentId(v as Parameters<typeof app.setAccentId>[0]);
    }
  });
  createEffect(() => {
    const v = densityHold.value();
    if (densityHold.locked() && app.density() !== v && DENSITY_OPTIONS.some((o) => o.value === v)) {
      app.setDensity(v as Density);
    }
  });
  // Modal focus management: trap Tab inside the panel, restore focus to the
  // opener on close, and close on Esc (WCAG 2.2 — dialog pattern).
  createFocusTrap(() => panel, { active:()=>!props.embedded, onEscape: () => props.onClose() });

  return (
    <SettingsNavigationContext.Provider value={navigation}><div
      class={props.embedded ? workspace.root : "compose__backdrop"}
      role={props.embedded ? "region" : "dialog"}
      aria-modal={props.embedded ? undefined : "true"}
      aria-label={t('settings-title')}
      onClick={(e) => {
        if (!props.embedded && e.target === e.currentTarget) props.onClose();
      }}
    >
      <section ref={panel} class={`${css.panel} ${props.embedded ? workspace.panel : ""}`} tabindex="-1">
        <header class={css.header}>
          <h2>{t(props.embedded ? 'settings-title' : 'settings-appearance')}</h2>
          <button type="button" class="btn btn--ghost" aria-label={t('settings-close')} onClick={() => props.onClose()}>
            <Icon name="close" />
          </button>
        </header>

        <div class={props.embedded ? workspace.body : workspace.legacy}>
          <Show when={props.embedded}><nav class={workspace.nav} aria-label={t('settings-sections')}><For each={sections}>{([id,label])=><button type="button" aria-current={navigation.route().section===id ? "page" : undefined} onClick={()=>void navigation.navigate({section:id})}>{t(label)}</button>}</For></nav></Show>
          <div class={workspace.content}>
          <div hidden={!visible('appearance')} class={workspace.appearance}>
        <ThemeMode locked={themeHold.locked()} />
        <ThemeGallery locked={themeHold.locked()} />
        <AppearanceSyncStatus />

        <div class={css.row}>
          <span class={css.label} id="settings-density">
            {t('settings-density')}
          </span>
          <div
            class={css.options}
            role="group"
            aria-labelledby="settings-density"
            aria-describedby={densityHold.locked() ? 'settings-density-lock' : undefined}
          >
            <For each={DENSITY_OPTIONS}>
              {(o) => (
                <button
                  type="button"
                  class={css.option}
                  aria-pressed={app.density() === o.value}
                  disabled={densityHold.locked()}
                  onClick={() => app.setDensity(o.value)}
                >
                  {t(o.label)}
                </button>
              )}
            </For>
          </div>
          <LockNote id="settings-density-lock" locked={densityHold.locked()} />
        </div>

        <AccentPicker locked={accentHold.locked()} />

        <div class={css.row}>
          <span class={css.label} id="settings-font">
            {t('settings-font')}
          </span>
          <div class={css.options} role="group" aria-labelledby="settings-font" aria-describedby={fontHold.locked() ? 'settings-font-lock' : undefined}>
            <For each={FONT_OPTIONS}>
              {(o) => (
                <button
                  type="button"
                  class={css.option}
                  aria-pressed={app.uiFont() === o.value}
                  disabled={fontHold.locked()}
                  onClick={() => app.setUiFont(o.value)}
                >
                  {t(o.label)}
                </button>
              )}
            </For>
          </div>
          <LockNote id="settings-font-lock" locked={fontHold.locked()} />
        </div>

        <div class={css.row}>
          <span class={css.label} id="settings-layout">
            {t('settings-layout')}
          </span>
          <div class={css.options} role="group" aria-labelledby="settings-layout">
            <For each={LAYOUT_OPTIONS}>
              {(o) => (
                <button
                  type="button"
                  class={css.option}
                  aria-pressed={app.layout() === o.value}
                  onClick={() => app.setLayout(o.value)}
                >
                  {t(o.label)}
                </button>
              )}
            </For>
          </div>
        </div>

        <Language />
        </div>

        {/* V6 security & integrations — only for an authenticated account. The
            `app.me` optional call keeps this inert in the theme-only unit test.
            Each section has its own boundary (t32-h6): a section whose REST or
            JMAP load fails shows its own error and retry instead of taking the
            whole dialog down with it. */}
        <Show when={app.me?.()?.accountId ?? null}>
          {(accountId) => (
            <>
              <Show when={visible('security')}><AsyncBoundary><PasswordChange accountId={accountId()} /></AsyncBoundary></Show>
              <AsyncBoundary>
                <AccountSettings hideSecurity onSearchFoldersChanged={() => app.refreshFolders()} {...(props.embedded ? {section:()=>navigation.route().section} : {})} />
              </AsyncBoundary>
              <Show when={visible('security')}><AsyncBoundary><SecurityCenter /></AsyncBoundary></Show>
              <Show when={visible('privacy')}><AsyncBoundary><Privacy /><AnalyticsConsent /></AsyncBoundary></Show>
              <Show when={visible('nextcloud')}><AsyncBoundary><NextcloudAccounts /></AsyncBoundary></Show>
              <Show when={visible('folders')}><AsyncBoundary><FolderPreferences folders={app.mailboxes} tags={app.tags} navigation={navigation}/></AsyncBoundary></Show>
              <Show when={visible('mail-transfer')}><AsyncBoundary><MailTransfer accountId={accountId()} mailboxes={app.mailboxes()} onImported={()=>void app.refreshFolders()}/></AsyncBoundary></Show>
              <Show when={visible('receipts')}><AsyncBoundary><Receipts accountId={accountId()}/></AsyncBoundary></Show>
              <Show when={visible('compose')}><AsyncBoundary><ComposePreferences /></AsyncBoundary></Show>
              <Show when={visible('shortcuts')}><AsyncBoundary><KeyboardShortcuts navigation={navigation}/></AsyncBoundary></Show>
              <Show when={visible('time')}><AsyncBoundary><TimeZones navigation={navigation}/></AsyncBoundary></Show>
              <Show when={visible('packages')}><AsyncBoundary><Packages accountId={accountId()} navigation={navigation}/></AsyncBoundary></Show>
              <Show when={visible('labels')}><AsyncBoundary><Labels navigation={navigation}/></AsyncBoundary></Show>
              <Show when={visible('reader')}><AsyncBoundary><ReaderAppearanceSettings /><ContactAppearance /></AsyncBoundary></Show>
              <Show when={visible('rules')}>              <AsyncBoundary>
                <RulesSettings accountId={accountId()} />
              </AsyncBoundary></Show>
              <Show when={visible('security')}>              <AsyncBoundary>
                <ZeroAccessBlock />
              </AsyncBoundary></Show>
              <Show when={visible('access')}>              <AsyncBoundary>
                <ApiKeys accountId={accountId()} />
              </AsyncBoundary></Show>
              <Show when={visible('access')}>              <AsyncBoundary>
                <McpKeys accountId={accountId()} />
              </AsyncBoundary></Show>
              <Show when={visible('metadata')}>              <AsyncBoundary>
                <ServerMetadataSection accountId={accountId()} />
              </AsyncBoundary></Show>
            </>
          )}
        </Show>
        </div></div>
      </section>
    </div></SettingsNavigationContext.Provider>
  );
}

// ── Theme (t19 e13) ──────────────────────────────────────────────────────────
//
// Three pieces, in reading order: WHEN the theme changes (the mode tri-state,
// plus the pair/schedule inputs the automatic modes need), WHICH theme (the
// gallery), and WHERE the choice is kept (the sync line).

/** Small dim explanatory line. */
function Hint(props: { children: JSX.Element }): JSX.Element {
  return <p class={css.hint}>{props.children}</p>;
}

/** `fixed` | `system` | `schedule`, plus whatever that mode needs configured. */
function ThemeMode(props: { locked: boolean }): JSX.Element {
  const app = useApp();
  return (
    <div class={css.row}>
      <span class={css.label} id="appearance-mode">
        {t('appearance-mode')}
      </span>
      <div
        class={css.options}
        role="group"
        aria-labelledby="appearance-mode"
        aria-describedby={props.locked ? 'settings-theme-lock' : undefined}
      >
        <For each={MODE_OPTIONS}>
          {(o) => (
            <button
              type="button"
              class={css.option}
              aria-pressed={app.themeMode() === o.value}
              disabled={props.locked}
              onClick={() => app.setThemeMode(o.value)}
            >
              {t(o.label)}
            </button>
          )}
        </For>
      </div>
      <Hint>{t(MODE_HINTS[app.themeMode()])}</Hint>

      {/* Both automatic modes switch between a PAIR of packs, so following the
          system is not limited to the two neutral themes. */}
      <Show when={app.themeMode() !== 'fixed'}>
        <div class={css.options}>
          <label class="field">
            <span class={css.label}>{t('appearance-pair-light')}</span>
            <select
              class={css.select}
              value={app.lightTheme()}
              onChange={(e) => {
                // `isThemeName` is the registry's sanctioned narrowing; a value
                // that is not a known pack is ignored rather than cast in.
                const v = e.currentTarget.value;
                if (isThemeName(v)) app.setLightTheme(v);
              }}
            >
              <For each={themesByAppearance('light')}>
                {(entry) => <option value={entry.id}>{t(entry.labelId)}</option>}
              </For>
            </select>
          </label>
          <label class="field">
            <span class={css.label}>{t('appearance-pair-dark')}</span>
            <select
              class={css.select}
              value={app.darkTheme()}
              onChange={(e) => {
                const v = e.currentTarget.value;
                if (isThemeName(v)) app.setDarkTheme(v);
              }}
            >
              <For each={themesByAppearance('dark')}>
                {(entry) => <option value={entry.id}>{t(entry.labelId)}</option>}
              </For>
            </select>
          </label>
        </div>
      </Show>

      <Show when={app.themeMode() === 'schedule'}>
        <div class={css.options}>
          <label class="field">
            <span class={css.label}>{t('appearance-schedule-start')}</span>
            <input
              type="time"
              class={css.select}
              aria-label={t('appearance-schedule-start')}
              value={app.schedule().darkStart}
              onChange={(e) =>
                app.setSchedule({ ...app.schedule(), darkStart: e.currentTarget.value })
              }
            />
          </label>
          <label class="field">
            <span class={css.label}>{t('appearance-schedule-end')}</span>
            <input
              type="time"
              class={css.select}
              aria-label={t('appearance-schedule-end')}
              value={app.schedule().darkEnd}
              onChange={(e) =>
                app.setSchedule({ ...app.schedule(), darkEnd: e.currentTarget.value })
              }
            />
          </label>
        </div>
      </Show>
    </div>
  );
}

/**
 * One card per pack, grouped by family (`THEME_GROUPS`). Picking a card is an
 * explicit choice, so it goes through `setTheme()` — which switches the mode to
 * `fixed` and seeds that appearance's pair member, so a later return to an
 * automatic mode stays inside the pack the user liked. The pair selects above
 * are how the automatic modes are steered without leaving them.
 */
function ThemeGallery(props: { locked: boolean }): JSX.Element {
  const app = useApp();
  return (
    <div class={css.row}>
      <span class={css.label} id="settings-theme">
        {t('settings-theme')}
      </span>
      <LockNote id="settings-theme-lock" locked={props.locked} />
      <Hint>{t('appearance-gallery-hint')}</Hint>
      <For each={THEME_GROUPS}>
        {(group) => (
          <Show when={group.themes.length > 0}>
            <div class={css.row}>
              <span class={css.label} id={`appearance-family-${group.family}`}>
                {t(group.labelId)}
              </span>
              <div
                class={css.options}
                role="group"
                aria-labelledby={`appearance-family-${group.family}`}
              >
                <For each={group.themes}>
                  {(entry) => (
                    <button
                      type="button"
                      class={`${css.option} ${picker.card}`}
                      // The accessible name is the pack name alone: the preview is
                      // decorative and the description/appearance are supporting
                      // detail, not part of what the control is called.
                      aria-label={t(entry.labelId)}
                      aria-pressed={app.theme() === entry.id}
                      aria-describedby={props.locked ? 'settings-theme-lock' : undefined}
                      disabled={props.locked}
                      onClick={() => app.setTheme(entry.id)}
                    >
                      <ThemePreview entry={entry} />
                      <span class={picker.cardName}>{t(entry.labelId)}</span>
                      <span class={picker.cardDescription}>{t(entry.descriptionId)}</span>
                      <span class={picker.cardMeta}>
                        {entry.appearance === 'dark' ? t('appearance-dark') : t('appearance-light')}
                        <Show when={app.theme() === entry.id}> · {t('appearance-active')}</Show>
                      </span>
                    </button>
                  )}
                </For>
              </div>
            </div>
          </Show>
        )}
      </For>
    </div>
  );
}

/**
 * A miniature of the pack painted in its OWN colours — page, panel, text, the
 * chosen accent as that pack shades it, and the control border — so the gallery
 * previews the theme rather than naming it. Purely decorative (`aria-hidden`):
 * every colour it shows is already stated by the card's label and appearance
 * line. Only the colours are inline; the shape is `picker.css.ts`.
 */
function ThemePreview(props: { entry: ThemeEntry }): JSX.Element {
  const app = useApp();
  const p = (): ThemeEntry['palette'] => props.entry.palette;
  return (
    <span
      aria-hidden="true"
      class={picker.preview}
      style={{ background: p().bg, 'border-color': p().borderStrong }}
    >
      <span class={picker.previewPanel} style={{ background: p().surface }} />
      <span class={picker.previewText} style={{ background: p().text }} />
      <span
        class={picker.previewAccent}
        style={{ background: accentShade(props.entry.id, app.accentId()).accent }}
      />
    </span>
  );
}

/**
 * The ten named accents plus the pack's own (t32 V). Each swatch is filled with
 * the ACTIVE theme's shade of the accent, which is what picking it paints, so
 * the row changes with the theme. The page itself is the preview: a pick
 * applies at once through `data-accent`.
 */
function AccentPicker(props: { locked: boolean }): JSX.Element {
  const app = useApp();
  return (
    <div class={css.row}>
      <span class={css.label} id="settings-accent">
        {t('settings-accent')}
      </span>
      <div
        class={css.options}
        role="group"
        aria-labelledby="settings-accent"
        aria-describedby={props.locked ? 'settings-accent-lock' : undefined}
      >
        <button
          type="button"
          class={css.option}
          aria-pressed={app.accentId() === ''}
          disabled={props.locked}
          onClick={() => app.setAccentId('')}
        >
          {t(THEME_DEFAULT_ACCENT_LABEL_ID)}
        </button>
        <For each={ACCENT_LIST}>
          {(def) => {
            const shade = () => accentShade(app.theme(), def.id);
            return (
              <button
                type="button"
                class={picker.swatch}
                aria-label={t(def.labelId)}
                title={t(def.labelId)}
                aria-pressed={app.accentId() === def.id}
                disabled={props.locked}
                style={{ background: shade().accent, color: shade().accentText }}
                onClick={() => app.setAccentId(def.id)}
              >
                <Show when={app.accentId() === def.id}>
                  <span aria-hidden="true"><Icon name="check" /></span>
                </Show>
              </button>
            );
          }}
        </For>
      </div>
      <LockNote id="settings-accent-lock" locked={props.locked} />
    </div>
  );
}

/**
 * Where the appearance is kept (SPEC §17.3). The sync is app-wide and idempotent
 * — this mount is a FALLBACK so the feature works even in a build that does not
 * start it at boot; when it is already running, `startAppearanceSync` returns the
 * running instance and nothing restarts.
 */
function AppearanceSyncStatus(): JSX.Element {
  const app = useApp();
  const [status, setStatus] = createSignal<SyncStatus>('idle');
  const [cleared, setCleared] = createSignal(false);

  const sync = startAppearanceSync(app);
  setStatus(sync.status());
  onCleanup(sync.subscribeStatus(setStatus));
  // Anything the user changed in this panel should not wait out the coalescing
  // window when they close it.
  onCleanup(() => void sync.flush());

  // The deployment default arrives with the load, which also moves the status —
  // reading `status()` first is what makes this re-render when it lands.
  const deploymentTheme = (): string | undefined => {
    status();
    const d = sync.deploymentDefault();
    return d !== null && d.theme !== '' ? d.theme : undefined;
  };

  return (
    <div class={css.row}>
      <span class={css.label} id="appearance-sync">
        {t('appearance-sync')}
      </span>
      <Hint>{t(SYNC_MESSAGES[status()])}</Hint>
      <Show when={deploymentTheme()}>
        {(theme) => <Hint>{t('appearance-sync-deployment', { theme: theme() })}</Hint>}
      </Show>
      <div class={css.options}>
        <button
          type="button"
          class={css.option}
          onClick={() => {
            void sync.reset().then(() => setCleared(true));
          }}
        >
          {t('appearance-sync-reset')}
        </button>
      </div>
      <Show when={cleared()}>
        <p class={css.label} role="status">
          {t('appearance-sync-reset-done')}
        </p>
      </Show>
    </div>
  );
}

// t13 (26.13, E9 mount): server-level METADATA, read-only. Builds the production
// ACL client (same-origin JMAP transport) once and renders the E8 view with no
// `mailboxId` (server-level scope). `canEdit` is omitted → read-only.
function ServerMetadataSection(props: { accountId: string }): JSX.Element {
  const client = createAclClient(props.accountId, createConfiguredClient().jmap);
  return <MetadataView client={client} />;
}

// The zero-access section. Its wasm-backed worker is spawned lazily on mount (and
// only where `Worker` exists — never under jsdom), so the appearance-only unit
// test and SSR are unaffected.
function ZeroAccessBlock(): JSX.Element {
  if (typeof Worker === 'undefined') return <></>;
  return <ZeroAccessSettings za={spawnZeroAccessWorker()} />;
}
