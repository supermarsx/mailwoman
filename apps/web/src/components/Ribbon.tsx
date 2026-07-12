// Optional collapsible Outlook-style ribbon (plan §3 e4, §0.9). A LAYOUT PRESET
// beside the default command-palette/pane layout — OFF unless the `layout`
// setting is `'ribbon'` (see the theme slice + Settings). Tabs: Home / View /
// Folder; collapse hides the group body and leaves just the tab strip.
//
// Styling is token-native (styles/ribbon.css.ts) — new component, so it uses
// `vars.*` directly rather than the legacy app.css bridge.

import { createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { THEME_OPTIONS } from '../theme/tokens.ts';
import type { Density } from '../theme/contract.css.ts';
import * as css from '../styles/ribbon.css.ts';

type RibbonTab = 'home' | 'view' | 'folder';

const TAB_LABELS: ReadonlyArray<{ id: RibbonTab; label: string }> = [
  { id: 'home', label: 'Home' },
  { id: 'view', label: 'View' },
  { id: 'folder', label: 'Folder' },
];

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

  return (
    <div class={css.ribbon} role="toolbar" aria-label="Ribbon">
      <div class={css.tabs}>
        <For each={TAB_LABELS}>
          {(t) => (
            <button
              type="button"
              class={active() === t.id ? `${css.tab} ${css.tabActive}` : css.tab}
              aria-pressed={active() === t.id}
              onClick={() => setActive(t.id)}
            >
              {t.label}
            </button>
          )}
        </For>
        <button
          type="button"
          class={css.collapseBtn}
          aria-expanded={!app.ribbonCollapsed()}
          onClick={() => app.setRibbonCollapsed(!app.ribbonCollapsed())}
        >
          {app.ribbonCollapsed() ? 'Expand ▾' : 'Collapse ▴'}
        </button>
      </div>

      <Show when={!app.ribbonCollapsed()}>
        <div class={css.body}>
          <Show when={active() === 'home'}>
            <div class={css.group}>
              <span class={css.groupLabel}>New</span>
              <div class={css.groupRow}>
                <button type="button" class={css.btn} onClick={() => props.onCompose()}>
                  ✉️ Compose
                </button>
              </div>
            </div>
            <div class={css.group}>
              <span class={css.groupLabel}>Session</span>
              <div class={css.groupRow}>
                <button type="button" class={css.btn} onClick={() => void app.logout()}>
                  ⎋ Log out
                </button>
              </div>
            </div>
          </Show>

          <Show when={active() === 'view'}>
            <div class={css.group}>
              <span class={css.groupLabel}>Theme</span>
              <div class={css.groupRow}>
                <select
                  class={css.btn}
                  aria-label="Theme"
                  value={app.theme()}
                  onChange={(e) => app.setTheme(e.currentTarget.value as (typeof THEME_OPTIONS)[number]['value'])}
                >
                  <For each={THEME_OPTIONS}>
                    {(o) => <option value={o.value}>{o.label}</option>}
                  </For>
                </select>
              </div>
            </div>
            <div class={css.group}>
              <span class={css.groupLabel}>Density</span>
              <div class={css.groupRow}>
                <For each={DENSITIES}>
                  {(d) => (
                    <button
                      type="button"
                      class={css.btn}
                      aria-pressed={app.density() === d}
                      onClick={() => app.setDensity(d)}
                    >
                      {d}
                    </button>
                  )}
                </For>
              </div>
            </div>
            <div class={css.group}>
              <span class={css.groupLabel}>Settings</span>
              <div class={css.groupRow}>
                <button type="button" class={css.btn} onClick={() => props.onOpenSettings()}>
                  ⚙ More…
                </button>
              </div>
            </div>
          </Show>

          <Show when={active() === 'folder'}>
            <div class={css.group}>
              <span class={css.groupLabel}>Folders</span>
              <div class={css.groupRow}>
                <For each={app.mailboxes()}>
                  {(box) => (
                    <button
                      type="button"
                      class={css.btn}
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
