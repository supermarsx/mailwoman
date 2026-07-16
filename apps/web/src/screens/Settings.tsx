// Settings panel (plan §3 e4): the user-facing theme / density / accent / font /
// layout controls that drive the theme slice. Rendered as a dismissible dialog;
// every control writes straight through the slice, which reflects onto :root and
// persists to localStorage (V2). Token-native styling (styles/settings.css.ts).

import { For, onMount, Show, type JSX } from 'solid-js';
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
import { THEME_OPTIONS, ACCENT_PRESETS } from '../theme/tokens.ts';
import type { Density } from '../theme/contract.css.ts';
import type { LayoutMode, UiFont } from '../state/slices/theme.ts';
import * as css from '../styles/settings.css.ts';

// Option labels are Fluent ids resolved through `t()` at render (reactive).
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
  onMount(() => void loadCatalog('settings'));
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

        <div class={css.row}>
          <span class={css.label} id="settings-theme">
            {t('settings-theme')}
          </span>
          <div class={css.options} role="group" aria-labelledby="settings-theme">
            <For each={THEME_OPTIONS}>
              {(o) => (
                <button
                  type="button"
                  class={css.option}
                  aria-pressed={app.theme() === o.value}
                  onClick={() => app.setTheme(o.value)}
                >
                  {o.label}
                </button>
              )}
            </For>
          </div>
        </div>

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
              <RulesSettings accountId={accountId()} />
              <ZeroAccessBlock />
              <ApiKeys accountId={accountId()} />
              <McpKeys accountId={accountId()} />
            </>
          )}
        </Show>
      </section>
    </div>
  );
}

// The zero-access section. Its wasm-backed worker is spawned lazily on mount (and
// only where `Worker` exists — never under jsdom), so the appearance-only unit
// test and SSR are unaffected.
function ZeroAccessBlock(): JSX.Element {
  if (typeof Worker === 'undefined') return <></>;
  return <ZeroAccessSettings za={spawnZeroAccessWorker()} />;
}
