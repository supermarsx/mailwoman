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

import { createSignal, For, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, loadCatalog } from '../i18n';
import { createFocusTrap } from '../components/a11y';
import { ServerSettings } from '../platform/ServerSettings.tsx';
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
import { createAclClient } from '../api/acl-types.ts';
import { createConfiguredClient } from '../api/transport.ts';
import { startAppearanceSync, type SyncStatus } from '../api/prefs.ts';
import { ACCENT_PRESETS } from '../theme/tokens.ts';
import {
  isThemeName,
  THEME_GROUPS,
  themesByAppearance,
  type ThemeEntry,
} from '../theme/registry.ts';
import { vars, type Density } from '../theme/contract.css.ts';
import type { LayoutMode, ThemeMode, UiFont } from '../state/slices/theme.ts';
import * as css from '../styles/settings.css.ts';

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
];

const LAYOUT_OPTIONS: ReadonlyArray<{ value: LayoutMode; label: string }> = [
  { value: 'default', label: 'settings-layout-default' },
  { value: 'ribbon', label: 'settings-layout-ribbon' },
];

export interface SettingsProps {
  onClose: () => void;
}

export function Settings(props: SettingsProps): JSX.Element {
  const app = useApp();
  let panel!: HTMLElement;
  onMount(() => {
    void loadCatalog('settings');
    void loadCatalog('appearance');
  });
  // Modal focus management: trap Tab inside the panel, restore focus to the
  // opener on close, and close on Esc (WCAG 2.2 — dialog pattern).
  createFocusTrap(() => panel, { onEscape: () => props.onClose() });

  return (
    <div
      class="compose__backdrop"
      role="dialog"
      aria-modal="true"
      aria-label={t('settings-title')}
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <section ref={panel} class={css.panel} tabindex="-1">
        <header class={css.header}>
          <h2>{t('settings-appearance')}</h2>
          <button type="button" class="btn btn--ghost" aria-label={t('settings-close')} onClick={() => props.onClose()}>
            ✕
          </button>
        </header>

        <ThemeMode />
        <ThemeGallery />
        <AppearanceSyncStatus />

        <div class={css.row}>
          <span class={css.label} id="settings-density">
            {t('settings-density')}
          </span>
          <div class={css.options} role="group" aria-labelledby="settings-density">
            <For each={DENSITY_OPTIONS}>
              {(o) => (
                <button
                  type="button"
                  class={css.option}
                  aria-pressed={app.density() === o.value}
                  onClick={() => app.setDensity(o.value)}
                >
                  {t(o.label)}
                </button>
              )}
            </For>
          </div>
        </div>

        <div class={css.row}>
          <span class={css.label} id="settings-accent">
            {t('settings-accent')}
          </span>
          <div class={css.options} role="group" aria-labelledby="settings-accent">
            <For each={ACCENT_PRESETS}>
              {(o) =>
                o.value === '' ? (
                  <button
                    type="button"
                    class={css.option}
                    aria-pressed={app.accent() === ''}
                    onClick={() => app.setAccent('')}
                  >
                    {o.label}
                  </button>
                ) : (
                  <button
                    type="button"
                    class={css.swatch}
                    aria-label={o.label}
                    aria-pressed={app.accent() === o.value}
                    style={{ background: o.value }}
                    onClick={() => app.setAccent(o.value)}
                  />
                )
              }
            </For>
          </div>
        </div>

        <div class={css.row}>
          <span class={css.label} id="settings-font">
            {t('settings-font')}
          </span>
          <div class={css.options} role="group" aria-labelledby="settings-font">
            <For each={FONT_OPTIONS}>
              {(o) => (
                <button
                  type="button"
                  class={css.option}
                  aria-pressed={app.uiFont() === o.value}
                  onClick={() => app.setUiFont(o.value)}
                >
                  {t(o.label)}
                </button>
              )}
            </For>
          </div>
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

        {/* Native-shell multi-server management; renders nothing in a browser. */}
        <ServerSettings />

        {/* V6 security & integrations — only for an authenticated account. The
            `app.me` optional call keeps this inert in the theme-only unit test. */}
        <Show when={app.me?.()?.accountId ?? null}>
          {(accountId) => (
            <>
              <PasswordChange accountId={accountId()} />
              <AccountSettings />
              <RulesSettings accountId={accountId()} />
              <ZeroAccessBlock />
              <ApiKeys accountId={accountId()} />
              <McpKeys accountId={accountId()} />
              <ServerMetadataSection accountId={accountId()} />
            </>
          )}
        </Show>
      </section>
    </div>
  );
}

// ── Theme (t19 e13) ──────────────────────────────────────────────────────────
//
// Three pieces, in reading order: WHEN the theme changes (the mode tri-state,
// plus the pair/schedule inputs the automatic modes need), WHICH theme (the
// gallery), and WHERE the choice is kept (the sync line).

/** Small dim explanatory line. `styles/settings.css.ts` has no class for this
 *  and belongs to another lane, so the two typographic properties are inline —
 *  the tightened CSP keeps `style-src-attr 'unsafe-inline'` for exactly this. */
function Hint(props: { children: JSX.Element }): JSX.Element {
  return (
    <p style={{ margin: 0, 'font-size': '0.78rem', color: vars.color.textDim }}>{props.children}</p>
  );
}

/** `fixed` | `system` | `schedule`, plus whatever that mode needs configured. */
function ThemeMode(): JSX.Element {
  const app = useApp();
  return (
    <div class={css.row}>
      <span class={css.label} id="appearance-mode">
        {t('appearance-mode')}
      </span>
      <div class={css.options} role="group" aria-labelledby="appearance-mode">
        <For each={MODE_OPTIONS}>
          {(o) => (
            <button
              type="button"
              class={css.option}
              aria-pressed={app.themeMode() === o.value}
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
                {(entry) => <option value={entry.id}>{entry.label}</option>}
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
                {(entry) => <option value={entry.id}>{entry.label}</option>}
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
function ThemeGallery(): JSX.Element {
  const app = useApp();
  return (
    <div class={css.row}>
      <span class={css.label} id="settings-theme">
        {t('settings-theme')}
      </span>
      <Hint>{t('appearance-gallery-hint')}</Hint>
      <For each={THEME_GROUPS}>
        {(group) => (
          <Show when={group.themes.length > 0}>
            <div class={css.row}>
              <span class={css.label} id={`appearance-family-${group.family}`}>
                {group.label}
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
                      class={css.option}
                      // The accessible name is the pack name alone: the preview is
                      // decorative and the description/appearance are supporting
                      // detail, not part of what the control is called.
                      aria-label={entry.label}
                      aria-pressed={app.theme() === entry.id}
                      onClick={() => app.setTheme(entry.id)}
                      style={{
                        display: 'flex',
                        'flex-direction': 'column',
                        gap: '6px',
                        'align-items': 'stretch',
                        'text-align': 'start',
                        width: '9.5rem',
                        padding: '0.5rem',
                      }}
                    >
                      <ThemePreview entry={entry} />
                      <span style={{ 'font-weight': '600' }}>{entry.label}</span>
                      <span style={{ 'font-size': '0.72rem', opacity: '0.85' }}>
                        {entry.description}
                      </span>
                      <span style={{ 'font-size': '0.7rem', opacity: '0.7' }}>
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
 * A miniature of the pack painted in its OWN colours — page, panel, text, accent
 * and border — so the gallery previews the theme rather than naming it. Purely
 * decorative (`aria-hidden`): every colour it shows is already stated by the
 * card's label and appearance line.
 */
function ThemePreview(props: { entry: ThemeEntry }): JSX.Element {
  const p = (): ThemeEntry['palette'] => props.entry.palette;
  return (
    <span
      aria-hidden="true"
      style={{
        display: 'block',
        height: '3rem',
        padding: '0.3rem',
        'border-radius': '4px',
        border: `1px solid ${p().border}`,
        background: p().bg,
      }}
    >
      <span
        style={{
          display: 'block',
          height: '0.85rem',
          'border-radius': '2px',
          background: p().surface,
          'margin-bottom': '0.3rem',
        }}
      />
      <span
        style={{
          display: 'block',
          height: '0.35rem',
          width: '70%',
          'border-radius': '2px',
          background: p().text,
          'margin-bottom': '0.25rem',
        }}
      />
      <span
        style={{
          display: 'block',
          height: '0.35rem',
          width: '35%',
          'border-radius': '2px',
          background: p().accent,
        }}
      />
    </span>
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
