// Settings panel (plan §3 e4): the user-facing theme / density / accent / font /
// layout controls that drive the theme slice. Rendered as a dismissible dialog;
// every control writes straight through the slice, which reflects onto :root and
// persists to localStorage (V2). Token-native styling (styles/settings.css.ts).

import { For, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { THEME_OPTIONS, ACCENT_PRESETS } from '../theme/tokens.ts';
import type { Density } from '../theme/contract.css.ts';
import type { LayoutMode, UiFont } from '../state/slices/theme.ts';
import * as css from '../styles/settings.css.ts';

const DENSITY_OPTIONS: ReadonlyArray<{ value: Density; label: string }> = [
  { value: 'compact', label: 'Compact' },
  { value: 'cozy', label: 'Cozy' },
  { value: 'relaxed', label: 'Relaxed' },
];

const FONT_OPTIONS: ReadonlyArray<{ value: UiFont; label: string }> = [
  { value: 'default', label: 'Default' },
  { value: 'system', label: 'System' },
  { value: 'serif', label: 'Serif' },
  { value: 'mono', label: 'Mono' },
];

const LAYOUT_OPTIONS: ReadonlyArray<{ value: LayoutMode; label: string }> = [
  { value: 'default', label: 'Default' },
  { value: 'ribbon', label: 'Ribbon' },
];

export interface SettingsProps {
  onClose: () => void;
}

export function Settings(props: SettingsProps): JSX.Element {
  const app = useApp();

  return (
    <div
      class="compose__backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="Settings"
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <section class={css.panel}>
        <header class={css.header}>
          <h2>Appearance</h2>
          <button type="button" class="btn btn--ghost" aria-label="Close settings" onClick={() => props.onClose()}>
            ✕
          </button>
        </header>

        <div class={css.row}>
          <span class={css.label} id="settings-theme">
            Theme
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
            Density
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
                  {o.label}
                </button>
              )}
            </For>
          </div>
        </div>

        <div class={css.row}>
          <span class={css.label} id="settings-accent">
            Accent
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
            Interface font
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
                  {o.label}
                </button>
              )}
            </For>
          </div>
        </div>

        <div class={css.row}>
          <span class={css.label} id="settings-layout">
            Layout
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
                  {o.label}
                </button>
              )}
            </For>
          </div>
        </div>
      </section>
    </div>
  );
}
