// Global search (t32-ext.md §K): Ctrl/Cmd+K or the header button.
//
// Matches the console's sections locally, and users, domains and settings keys
// through `GET /admin/search?q=` (t32-e13, capability-filtered on the server).
// Until that route exists the server answers 404 and the palette says that only
// sections are searched; it never shows made-up results.
//
// ARIA combobox: the input owns a listbox, Up/Down move the active option
// (`aria-activedescendant`), Enter opens it, Escape closes the palette and focus
// returns to where it was.

import { For, Show, createEffect, createMemo, createSignal, createUniqueId, onCleanup, untrack, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { t } from '../../../i18n';
import { basePath } from '../../../api/basePath.ts';
import { adminUrl, isConsoleLocation } from '../../../api/adminPath.ts';
import { apiFetch } from '../../../api/http.ts';
import { createFocusTrap } from '../../../components/a11y';
import { Icon, type IconName } from './icons.tsx';
import * as css from './shell.css.ts';

export type SearchHitKind = 'user' | 'domain' | 'setting' | 'section';

export interface SearchHit {
  kind: SearchHitKind;
  label: string;
  detail?: string;
  /** App location to open (`/admin#/...`). */
  path: string;
}

/** Server search: hits, or `unavailable` when the server has no search route. */
export type AdminSearch = (q: string, signal: AbortSignal) => Promise<SearchHit[] | 'unavailable'>;

/** `GET <base>/admin/search?q=` → `{ hits: [{kind, label, detail?, path}] }`. */
export function createAdminSearch(base = basePath()): AdminSearch {
  return async (q, signal) => {
    const res = await apiFetch(adminUrl(`/search?q=${encodeURIComponent(q)}`, base), {
      credentials: 'same-origin',
      signal,
    });
    if (res.status === 404) return 'unavailable';
    if (!res.ok) throw new Error(`search failed (${res.status})`);
    const body = (await res.json()) as { hits?: SearchHit[] };
    return Array.isArray(body.hits) ? body.hits.filter((h) => typeof h.path === 'string' && isConsoleLocation(h.path)) : [];
  };
}

export interface PaletteSection {
  id: string;
  label: string;
  icon: IconName;
  path: string;
}

export interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  /** The sections the admin may open (already capability-filtered). */
  sections: readonly PaletteSection[];
  search: AdminSearch;
  onOpen: (path: string) => void;
}

export function CommandPalette(props: CommandPaletteProps): JSX.Element {
  return (
    <Show when={props.open}>
      <PaletteOpen {...props} />
    </Show>
  );
}

const KIND_LABEL: Record<SearchHitKind, () => string> = {
  section: () => t('admin-shell-palette-kind-section'),
  user: () => t('admin-shell-palette-kind-user'),
  domain: () => t('admin-shell-palette-kind-domain'),
  setting: () => t('admin-shell-palette-kind-setting'),
};

const KIND_ICON: Record<SearchHitKind, IconName> = {
  section: 'chevron',
  user: 'users',
  domain: 'domains',
  setting: 'settings',
};

function PaletteOpen(props: CommandPaletteProps): JSX.Element {
  const id = createUniqueId();
  const [el, setEl] = createSignal<HTMLElement>();
  const [q, setQ] = createSignal('');
  const [active, setActive] = createSignal(0);
  const [remote, setRemote] = createSignal<SearchHit[]>([]);
  const [remoteState, setRemoteState] = createSignal<'idle' | 'busy' | 'unavailable' | 'error'>('idle');
  let input: HTMLInputElement | undefined;

  createFocusTrap(el, { onEscape: () => props.onClose(), initialFocus: () => input });

  const local = createMemo<SearchHit[]>(() => {
    const needle = q().trim().toLowerCase();
    return props.sections
      .filter((s) => needle === '' || s.label.toLowerCase().includes(needle))
      .map((s) => ({ kind: 'section', label: s.label, path: s.path }));
  });
  const hits = createMemo(() => [...local(), ...remote()]);

  // Server search, debounced, cancelled when the text changes again. Only the
  // text is tracked: the state signals are written from inside.
  createEffect(() => {
    const needle = q().trim();
    setActive(0);
    const unavailable = untrack(remoteState) === 'unavailable';
    if (needle.length < 2) {
      setRemote([]);
      if (!unavailable) setRemoteState('idle');
      return;
    }
    if (unavailable) return;
    const ctl = new AbortController();
    const timer = setTimeout(() => {
      setRemoteState('busy');
      props
        .search(needle, ctl.signal)
        .then((r) => {
          if (ctl.signal.aborted) return;
          if (r === 'unavailable') {
            setRemote([]);
            setRemoteState('unavailable');
          } else {
            setRemote(r);
            setRemoteState('idle');
          }
        })
        .catch(() => {
          if (!ctl.signal.aborted) setRemoteState('error');
        });
    }, 200);
    onCleanup(() => {
      clearTimeout(timer);
      ctl.abort();
    });
  });

  function choose(hit: SearchHit | undefined): void {
    if (!hit) return;
    props.onClose();
    props.onOpen(hit.path);
  }

  function onKeyDown(e: KeyboardEvent): void {
    const n = hits().length;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (n > 0) setActive((active() + 1) % n);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (n > 0) setActive((active() - 1 + n) % n);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      choose(hits()[active()]);
    }
  }

  const optionId = (i: number): string => `${id}-opt-${i}`;

  return (
    <Portal>
      <div class={css.dialogScrim} onClick={() => props.onClose()} />
      <div ref={setEl} class={css.palette} role="dialog" aria-modal="true" aria-labelledby={`${id}-title`} tabindex="-1">
        <h2 id={`${id}-title`} class={css.dialogTitle}>
          {t('admin-shell-palette-title')}
        </h2>
        <div class="field">
          <label for={`${id}-input`}>{t('admin-shell-palette-label')}</label>
          <input
            ref={input}
            id={`${id}-input`}
            type="text"
            role="combobox"
            autocomplete="off"
            spellcheck={false}
            aria-expanded={hits().length > 0}
            aria-controls={`${id}-list`}
            aria-autocomplete="list"
            aria-activedescendant={hits().length > 0 ? optionId(active()) : undefined}
            value={q()}
            onInput={(e) => setQ(e.currentTarget.value)}
            onKeyDown={onKeyDown}
          />
        </div>
        <ul id={`${id}-list`} class={css.paletteList} role="listbox" aria-label={t('admin-shell-palette-results')}>
          <For each={hits()}>
            {(hit, i) => (
              <li
                id={optionId(i())}
                role="option"
                class={css.paletteOption}
                aria-selected={i() === active()}
                onPointerMove={() => setActive(i())}
                onClick={() => choose(hit)}
              >
                <Icon name={KIND_ICON[hit.kind] ?? 'chevron'} />
                <span>
                  {hit.label}
                  <Show when={hit.detail}>
                    {(d) => <span class={css.kbd}> {d()}</span>}
                  </Show>
                </span>
                <span class={css.paletteKind}>{(KIND_LABEL[hit.kind] ?? KIND_LABEL.section)()}</span>
              </li>
            )}
          </For>
        </ul>
        <p class={css.help} role="status">
          <Show when={hits().length === 0}>{t('admin-shell-palette-none')} </Show>
          <Show when={remoteState() === 'busy'}>{t('admin-shell-palette-searching')}</Show>
          <Show when={remoteState() === 'unavailable'}>{t('admin-shell-palette-sections-only')}</Show>
          <Show when={remoteState() === 'error'}>{t('admin-shell-palette-error')}</Show>
        </p>
      </div>
    </Portal>
  );
}
