// Optional collapsible Outlook-style ribbon (plan §3 e4, §0.9). A LAYOUT PRESET
// beside the default command-palette/pane layout — OFF unless the `layout`
// setting is `'ribbon'` (see the theme slice + Settings). Tabs: Home / View /
// Folder; collapse hides the group body and leaves just the tab strip.
//
// a11y (t8-e1): the tab strip is a WAI-ARIA `tablist` with roving tabindex —
// exactly one tab is Tab-reachable, Arrow/Home/End move between tabs with
// selection following focus, and each tab `aria-controls` its `tabpanel`. The
// panel groups are labelled toolbars. Styling is token-native
// (styles/ribbon.css.ts); focus rings come from the shared a11y contract.

import { createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';
import { THEME_OPTIONS } from '../theme/tokens.ts';
import type { Density } from '../theme/contract.css.ts';
import * as css from '../styles/ribbon.css.ts';
import * as a11y from './mailA11y.css.ts';

type RibbonTab = 'home' | 'view' | 'folder';

const TABS: readonly RibbonTab[] = ['home', 'view', 'folder'];
const DENSITIES: readonly Density[] = ['compact', 'cozy', 'relaxed'];

export interface RibbonProps {
  /** Open the composer (owned by the mailbox screen). */
  onCompose: () => void;
  /** Open the Settings panel. */
  onOpenSettings: () => void;
}

export function Ribbon(props: RibbonProps): JSX.Element {
  const app = useApp();
  const [active, setActive] = createSignal<RibbonTab>('home');
  const tabEls: (HTMLButtonElement | undefined)[] = [];

  /** Roving-tabindex keyboard handler: Arrow/Home/End move + activate the tab. */
  function onTabKeyDown(e: KeyboardEvent): void {
    const idx = TABS.indexOf(active());
    let next = idx;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = (idx + 1) % TABS.length;
    else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = (idx - 1 + TABS.length) % TABS.length;
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = TABS.length - 1;
    else return;
    e.preventDefault();
    const tab = TABS[next]!;
    setActive(tab);
    tabEls[next]?.focus();
  }

  return (
    <div class={css.ribbon}>
      <div class={css.tabs}>
        <div role="tablist" aria-label={t('mail-ribbon-label')} onKeyDown={onTabKeyDown} style={{ display: 'flex', gap: '0' }}>
          <For each={TABS}>
            {(id, i) => (
              <button
                type="button"
                ref={(el) => (tabEls[i()] = el)}
                role="tab"
                id={`ribbon-tab-${id}`}
                aria-controls="ribbon-panel"
                aria-selected={active() === id}
                tabindex={active() === id ? 0 : -1}
                class={`${active() === id ? `${css.tab} ${css.tabActive}` : css.tab} ${a11y.focusable}`}
                onClick={() => setActive(id)}
              >
                {t(`mail-ribbon-tab-${id}`)}
              </button>
            )}
          </For>
        </div>
        <button
          type="button"
          class={`${css.collapseBtn} ${a11y.focusable}`}
          aria-expanded={!app.ribbonCollapsed()}
          onClick={() => app.setRibbonCollapsed(!app.ribbonCollapsed())}
        >
          {app.ribbonCollapsed() ? `${t('mail-ribbon-expand')} ▾` : `${t('mail-ribbon-collapse')} ▴`}
        </button>
      </div>

      <Show when={!app.ribbonCollapsed()}>
        <div id="ribbon-panel" role="tabpanel" aria-labelledby={`ribbon-tab-${active()}`} class={css.body}>
          <Show when={active() === 'home'}>
            <div class={css.group} role="group" aria-label={t('mail-ribbon-group-new')}>
              <span class={css.groupLabel}>{t('mail-ribbon-group-new')}</span>
              <div class={css.groupRow}>
                <button type="button" class={`${css.btn} ${a11y.focusable}`} onClick={() => props.onCompose()}>
                  ✉️ {t('mail-ribbon-compose')}
                </button>
              </div>
            </div>
            <div class={css.group} role="group" aria-label={t('mail-ribbon-group-session')}>
              <span class={css.groupLabel}>{t('mail-ribbon-group-session')}</span>
              <div class={css.groupRow}>
                <button type="button" class={`${css.btn} ${a11y.focusable}`} onClick={() => void app.logout()}>
                  ⎋ {t('mail-ribbon-logout')}
                </button>
              </div>
            </div>
          </Show>

          <Show when={active() === 'view'}>
            <div class={css.group} role="group" aria-label={t('mail-ribbon-group-theme')}>
              <span class={css.groupLabel}>{t('mail-ribbon-group-theme')}</span>
              <div class={css.groupRow}>
                <select
                  class={`${css.btn} ${a11y.focusable}`}
                  aria-label={t('mail-ribbon-group-theme')}
                  value={app.theme()}
                  onChange={(e) => app.setTheme(e.currentTarget.value as (typeof THEME_OPTIONS)[number]['value'])}
                >
                  <For each={THEME_OPTIONS}>
                    {(o) => <option value={o.value}>{o.label}</option>}
                  </For>
                </select>
              </div>
            </div>
            <div class={css.group} role="group" aria-label={t('mail-ribbon-group-density')}>
              <span class={css.groupLabel}>{t('mail-ribbon-group-density')}</span>
              <div class={css.groupRow}>
                <For each={DENSITIES}>
                  {(d) => (
                    <button
                      type="button"
                      class={`${css.btn} ${a11y.focusable}`}
                      aria-pressed={app.density() === d}
                      onClick={() => app.setDensity(d)}
                    >
                      {t(`mail-density-${d}`)}
                    </button>
                  )}
                </For>
              </div>
            </div>
            <div class={css.group} role="group" aria-label={t('mail-ribbon-group-settings')}>
              <span class={css.groupLabel}>{t('mail-ribbon-group-settings')}</span>
              <div class={css.groupRow}>
                <button type="button" class={`${css.btn} ${a11y.focusable}`} onClick={() => props.onOpenSettings()}>
                  ⚙ {t('mail-ribbon-more')}
                </button>
              </div>
            </div>
          </Show>

          <Show when={active() === 'folder'}>
            <div class={css.group} role="group" aria-label={t('mail-ribbon-group-folders')}>
              <span class={css.groupLabel}>{t('mail-ribbon-group-folders')}</span>
              <div class={css.groupRow}>
                <For each={app.mailboxes()}>
                  {(box) => (
                    <button
                      type="button"
                      class={`${css.btn} ${a11y.focusable}`}
                      aria-pressed={app.selectedMailboxId() === box.id}
                      onClick={() => void app.selectMailbox(box.id)}
                    >
                      {box.name}
                    </button>
                  )}
                </For>
              </div>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}
