import { Icon } from './Icon.tsx';
// Optional Outlook-style ribbon (plan §3 e4, §0.9). A LAYOUT PRESET beside the
// default command-palette/pane layout — OFF unless the `layout` setting is
// `'ribbon'` (see the theme slice + Settings). Tabs: Home / View / Folder.
//
// t38: the ribbon is a `CollapsibleBar` (`ui.bar.mailRibbon`). Pinned, the
// tab names and the groups are always shown; auto-hide keeps only the tab
// names, and the groups slide out over the content on hover, a click on a tab
// or keyboard focus (Esc closes them and returns focus to the tab). Every
// group is a `RibbonGroup`: exactly three rows of controls, never more, never
// scrolling vertically; when a tab's groups are too wide, the lowest-priority
// ones fold into dropdown buttons (user decision D1).
//
// a11y (t8-e1): the tab strip is a WAI-ARIA `tablist` with roving tabindex —
// exactly one tab is Tab-reachable, Arrow/Home/End move between tabs with
// selection following focus, and each tab `aria-controls` its `tabpanel`. The
// panel groups are labelled groups. Styling is token-native
// (styles/ribbon.css.ts); focus rings come from the shared a11y contract.

import { createEffect, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { loadCatalog, t } from '../i18n/index.ts';
import { THEME_OPTIONS } from '../theme/tokens.ts';
import type { Density } from '../theme/contract.css.ts';
import { settings } from '../api/settings.ts';
import { migrateLegacyRibbonCollapsed } from '../state/legacyRibbonCollapsed.ts';
import { CollapsibleBar, RibbonGroup, RibbonItem, RibbonPanel, useCollapsibleBar, type BarModeHandle } from './bars/index.ts';
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
  /** The pin/auto-hide handle; defaults to `useBarMode('mailRibbon')` (tests pass one). */
  bar?: BarModeHandle;
}

export function Ribbon(props: RibbonProps): JSX.Element {
  const app = useApp();
  // The theme names in the View tab come from the appearance catalog.
  onMount(() => void loadCatalog('appearance'));
  // The pre-t38 device-local collapse becomes `ui.bar.mailRibbon = auto`, once.
  createEffect(() => {
    settings();
    void migrateLegacyRibbonCollapsed();
  });
  const [active, setActive] = createSignal<RibbonTab>('home');

  const btn = `${css.btn} ${a11y.focusable}`;
  const large = `${css.btn} ${css.btnLarge} ${a11y.focusable}`;

  return (
    <CollapsibleBar
      id="mailRibbon"
      revealPolicy="overlay"
      {...(props.bar ? { bar: props.bar } : {})}
      class={css.ribbon}
      panelClass={css.panel}
      header={<RibbonTabs active={active()} onSelect={setActive} />}
    >
      <div id="ribbon-panel" role="tabpanel" aria-labelledby={`ribbon-tab-${active()}`} class={css.body}>
        <Show when={active() === 'home'}>
          <RibbonPanel label={t('mail-ribbon-tab-home')}>
            <RibbonGroup label={t('mail-ribbon-group-new')} priority={30} icon="mail" testId="ribbon-group-new">
              <RibbonItem size="large">
                <button type="button" class={large} onClick={() => props.onCompose()}>
                  <Icon name="mail" />
                  <span class={css.label}>{t('mail-ribbon-compose')}</span>
                </button>
              </RibbonItem>
            </RibbonGroup>
            <RibbonGroup label={t('mail-ribbon-group-session')} priority={10} icon="signOut" testId="ribbon-group-session">
              <RibbonItem>
                <button type="button" class={btn} onClick={() => void app.logout()}>
                  <Icon name="signOut" />
                  <span class={css.label}>{t('mail-ribbon-logout')}</span>
                </button>
              </RibbonItem>
            </RibbonGroup>
          </RibbonPanel>
        </Show>

        <Show when={active() === 'view'}>
          <RibbonPanel label={t('mail-ribbon-tab-view')}>
            <RibbonGroup label={t('mail-ribbon-group-theme')} priority={30} icon="branding" testId="ribbon-group-theme">
              <RibbonItem inlineSize="11rem">
                <select
                  class={`${css.btn} ${css.select} ${a11y.focusable}`}
                  aria-label={t('mail-ribbon-group-theme')}
                  value={app.theme()}
                  onChange={(e) => app.setTheme(e.currentTarget.value as (typeof THEME_OPTIONS)[number]['value'])}
                >
                  <For each={THEME_OPTIONS}>
                    {(o) => <option value={o.value}>{t(o.labelId)}</option>}
                  </For>
                </select>
              </RibbonItem>
            </RibbonGroup>
            <RibbonGroup label={t('mail-ribbon-group-density')} priority={20} icon="preferences" testId="ribbon-group-density">
              <For each={DENSITIES}>
                {(d) => (
                  <RibbonItem>
                    <button type="button" class={btn} aria-pressed={app.density() === d} onClick={() => app.setDensity(d)}>
                      <span class={css.label}>{t(`mail-density-${d}`)}</span>
                    </button>
                  </RibbonItem>
                )}
              </For>
            </RibbonGroup>
            <RibbonGroup label={t('mail-ribbon-group-settings')} priority={10} icon="settings" testId="ribbon-group-settings">
              <RibbonItem size="large">
                <button type="button" class={large} onClick={() => props.onOpenSettings()}>
                  <Icon name="settings" />
                  <span class={css.label}>{t('mail-ribbon-more')}</span>
                </button>
              </RibbonItem>
            </RibbonGroup>
          </RibbonPanel>
        </Show>

        <Show when={active() === 'folder'}>
          <RibbonPanel label={t('mail-ribbon-tab-folder')}>
            <RibbonGroup label={t('mail-ribbon-group-folders')} priority={10} icon="folder" testId="ribbon-group-folders">
              <For each={app.mailboxes()}>
                {(box) => (
                  <RibbonItem>
                    <button
                      type="button"
                      class={btn}
                      aria-pressed={app.selectedMailboxId() === box.id}
                      data-tooltip={box.name}
                      onClick={() => void app.selectMailbox(box.id)}
                    >
                      <span class={css.label}>{box.name}</span>
                    </button>
                  </RibbonItem>
                )}
              </For>
            </RibbonGroup>
          </RibbonPanel>
        </Show>
      </div>
    </CollapsibleBar>
  );
}

/** The always-visible tab names; in auto-hide mode a click (or focus) opens the bar. */
function RibbonTabs(props: { active: RibbonTab; onSelect: (tab: RibbonTab) => void }): JSX.Element {
  const bar = useCollapsibleBar();
  const tabEls: (HTMLButtonElement | undefined)[] = [];

  /** Roving-tabindex keyboard handler: Arrow/Home/End move + activate the tab. */
  function onTabKeyDown(e: KeyboardEvent): void {
    const idx = TABS.indexOf(props.active);
    let next = idx;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = (idx + 1) % TABS.length;
    else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = (idx - 1 + TABS.length) % TABS.length;
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = TABS.length - 1;
    else return;
    e.preventDefault();
    props.onSelect(TABS[next]!);
    tabEls[next]?.focus();
  }

  return (
    <div class={css.tabs}>
      <div class={css.tablist} role="tablist" aria-label={t('mail-ribbon-label')} onKeyDown={onTabKeyDown}>
        <For each={TABS}>
          {(id, i) => (
            <button
              type="button"
              ref={(el) => (tabEls[i()] = el)}
              role="tab"
              id={`ribbon-tab-${id}`}
              aria-controls="ribbon-panel"
              aria-selected={props.active === id}
              aria-expanded={bar?.mode() === 'auto' ? (bar.expanded() ? 'true' : 'false') : undefined}
              tabindex={props.active === id ? 0 : -1}
              class={`${props.active === id ? `${css.tab} ${css.tabActive}` : css.tab} ${a11y.focusable}`}
              onClick={(e) => {
                props.onSelect(id);
                bar?.open(e.currentTarget);
              }}
            >
              {t(`mail-ribbon-tab-${id}`)}
            </button>
          )}
        </For>
      </div>
    </div>
  );
}
