// The console's appearance control (t32-cfg DC5, UX1): a header button that opens
// a dialog with the mode (light, dark, match the system), every built-in theme
// grouped by pack, and the ten accents plus the theme's own.
//
// A pick applies at once (the console itself is the preview) and is kept for this
// browser (`appearanceStore.ts`). "Use the webmail theme" forgets the choice.
// The dialog is modal: focus moves in, Tab stays inside, Escape and Close return
// focus to the button. Theme cards and accent swatches are toggle buttons
// (`aria-pressed`); the selected one also shows a check, so the state is not
// carried by colour alone. Theme previews are classes generated from the theme
// registry, so no inline style attribute is needed (CSP `style-src 'self'`).

import { For, Show, createContext, createSignal, createUniqueId, onMount, useContext, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { loadCatalog, t } from '../../../i18n';
import { createFocusTrap } from '../../../components/a11y';
import { ACCENT_LIST, THEME_DEFAULT_ACCENT_LABEL_ID, THEME_GROUPS } from '../../../theme/registry.ts';
import { ADMIN_THEME_MODES, type AdminAppearanceController, type AdminThemeMode } from './appearanceStore.ts';
import { Icon } from './icons.tsx';
import * as css from './shell.css.ts';
import * as ux from './ux.css.ts';

export const AdminAppearanceContext = createContext<AdminAppearanceController>();

/** The controller provided by the console root, or undefined (tests, the sign-in gate). */
export function useAdminAppearance(): AdminAppearanceController | undefined {
  return useContext(AdminAppearanceContext);
}

const MODE_LABEL: Record<AdminThemeMode, () => string> = {
  light: () => t('admin-ux-appearance-mode-light'),
  dark: () => t('admin-ux-appearance-mode-dark'),
  system: () => t('admin-ux-appearance-mode-system'),
};

/** A half-filled circle: light and dark. Decorative. */
function AppearanceGlyph(): JSX.Element {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true" data-icon="appearance">
      <circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="2" />
      <path d="M8 2 A6 6 0 0 1 8 14 Z" fill="currentColor" />
    </svg>
  );
}

export interface AdminAppearanceProps {
  controller: AdminAppearanceController;
  /** Extra classes for the header button (the shell's icon-button classes). */
  buttonClass?: string;
  /** Show only the glyph (phone top bar); the name stays for screen readers. */
  compact?: boolean;
}

/** The header button and its dialog. */
export function AdminAppearance(props: AdminAppearanceProps): JSX.Element {
  const [open, setOpen] = createSignal(false);
  return (
    <>
      <button
        type="button"
        class={['btn btn--ghost', ux.appearanceButton, props.buttonClass ?? ''].join(' ').trim()}
        aria-haspopup="dialog"
        aria-expanded={open()}
        data-testid="admin-appearance-button"
        onClick={() => setOpen(true)}
      >
        <AppearanceGlyph />
        <span class={props.compact ? css.srOnly : undefined}>{t('admin-ux-appearance')}</span>
      </button>
      <Show when={open()}>
        <AppearanceDialog controller={props.controller} onClose={() => setOpen(false)} />
      </Show>
    </>
  );
}

function AppearanceDialog(props: { controller: AdminAppearanceController; onClose: () => void }): JSX.Element {
  const id = createUniqueId();
  const [el, setEl] = createSignal<HTMLElement>();
  const c = props.controller;
  let closeButton: HTMLButtonElement | undefined;
  onMount(() => void loadCatalog('appearance'));
  createFocusTrap(el, { onEscape: () => props.onClose() });

  return (
    <Portal>
      <div class={css.dialogScrim} onClick={() => props.onClose()} />
      <div
        ref={setEl}
        class={[css.dialog, ux.appearanceDialog].join(' ')}
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        aria-describedby={`${id}-note`}
        tabindex="-1"
        data-testid="admin-appearance"
      >
        <h2 id={`${id}-title`} class={css.dialogTitle}>
          {t('admin-ux-appearance-title')}
        </h2>
        <p id={`${id}-note`} class={css.help}>
          {c.chosen() ? t('admin-ux-appearance-note') : t('admin-ux-appearance-following')}
        </p>

        <AdminAppearanceFields controller={c} />

        <div class={css.dialogActions}>
          <Show when={c.chosen()}>
            <button
              type="button"
              class="btn btn--ghost"
              onClick={() => {
                c.reset();
                // The button goes away with the choice; keep focus inside the dialog.
                queueMicrotask(() => closeButton?.focus());
              }}
            >
              {t('admin-ux-appearance-reset')}
            </button>
          </Show>
          <button ref={closeButton} type="button" class="btn btn--primary" onClick={() => props.onClose()}>
            {t('admin-ux-close')}
          </button>
        </div>
      </div>
    </Portal>
  );
}

/** Inline fields for the personal preferences section. */
export function AdminAppearanceFields(props: { controller: AdminAppearanceController }): JSX.Element {
  const id = createUniqueId();
  const c = props.controller;
  onMount(() => void loadCatalog('appearance'));
  return <>
        <fieldset class={ux.appearanceSection}>
          <legend class={ux.appearanceLegend}>{t('admin-ux-appearance-mode')}</legend>
          <div class={ux.modeGroup}>
            <For each={ADMIN_THEME_MODES}>
              {(mode) => (
                <label class={ux.modeOption}>
                  <input
                    type="radio"
                    name={`${id}-mode`}
                    value={mode}
                    checked={c.prefs().mode === mode}
                    onChange={() => c.setMode(mode)}
                  />
                  <span>{MODE_LABEL[mode]()}</span>
                </label>
              )}
            </For>
          </div>
        </fieldset>

        <fieldset class={ux.appearanceSection}>
          <legend class={ux.appearanceLegend}>{t('admin-ux-appearance-theme')}</legend>
          <For each={THEME_GROUPS.filter((g) => g.themes.length > 0)}>
            {(group) => (
              <div class={ux.appearanceSection} role="group" aria-labelledby={`${id}-family-${group.family}`}>
                <p id={`${id}-family-${group.family}`} class={ux.familyLabel}>
                  {t(group.labelId)}
                </p>
                <div class={ux.themeGrid}>
                  <For each={group.themes}>
                    {(entry) => (
                      <button
                        type="button"
                        class={ux.themeCard}
                        aria-pressed={c.resolved() === entry.id}
                        data-theme-option={entry.id}
                        onClick={() => c.setTheme(entry.id)}
                      >
                        <span class={[ux.preview, ux.previewBg[entry.id]].join(' ')} aria-hidden="true">
                          <span class={[ux.previewPanel, ux.previewPanelBg[entry.id]].join(' ')} />
                          <span class={[ux.previewText, ux.previewTextBg[entry.id]].join(' ')} />
                          <span class={[ux.previewAccent, ux.previewAccentBg[entry.id]].join(' ')} />
                        </span>
                        <span class={ux.themeName}>
                          <Show when={c.resolved() === entry.id}>
                            <Icon name="check" />
                          </Show>
                          {t(entry.labelId)}
                        </span>
                        <span class={ux.themeMeta}>
                          {entry.appearance === 'dark' ? t('admin-ux-appearance-dark') : t('admin-ux-appearance-light')}
                        </span>
                      </button>
                    )}
                  </For>
                </div>
              </div>
            )}
          </For>
        </fieldset>

        <fieldset class={ux.appearanceSection}>
          <legend class={ux.appearanceLegend}>{t('admin-ux-appearance-accent')}</legend>
          <div class={ux.accentRow}>
            <button type="button" class={ux.accentDefault} aria-pressed={c.prefs().accent === ''} onClick={() => c.setAccent('')}>
              {t(THEME_DEFAULT_ACCENT_LABEL_ID)}
            </button>
            <For each={ACCENT_LIST}>
              {(def) => (
                <button
                  type="button"
                  class={[ux.swatch, ux.swatchAccent[def.id]].join(' ')}
                  aria-label={t(def.labelId)}
                  title={t(def.labelId)}
                  aria-pressed={c.prefs().accent === def.id}
                  data-accent-option={def.id}
                  onClick={() => c.setAccent(def.id)}
                >
                  <Show when={c.prefs().accent === def.id}>
                    <Icon name="check" />
                  </Show>
                </button>
              )}
            </For>
          </div>
        </fieldset>

  </>;
}
