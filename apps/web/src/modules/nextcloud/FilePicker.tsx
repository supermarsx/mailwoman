// A minimal WebDAV directory picker (SPEC §18.4, plan §3 e7). Browses Nextcloud
// folders via the injected NextcloudService, letting the caller either navigate into
// directories and select files (attach) or pick a destination directory (save-to).
// Presentational + navigation only — the caller decides what "select" means.

import { createSignal, createResource, For, Show, type JSX } from 'solid-js';
import { NextcloudService, type WebDavEntry } from './service.ts';
import * as css from './styles.css.ts';

export interface FilePickerProps {
  service: NextcloudService;
  /** 'files' → clicking a file toggles selection; 'dirs' → only navigation, the
   *  current directory is the selection. */
  mode: 'files' | 'dirs';
  /** Selected file paths (files mode). */
  selected?: Set<string>;
  /** A file was toggled (files mode). */
  onToggleFile?: (entry: WebDavEntry) => void;
  /** The browsed directory changed (both modes). */
  onDirChange?: (path: string) => void;
}

function humanSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let n = bytes / 1024;
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i += 1;
  }
  return `${n.toFixed(1)} ${units[i]}`;
}

export function FilePicker(props: FilePickerProps): JSX.Element {
  const [dir, setDir] = createSignal('/');
  const [entries] = createResource(dir, (path) => props.service.list(path));

  function navigate(path: string): void {
    setDir(path);
    props.onDirChange?.(path);
  }

  function parentOf(path: string): string {
    const trimmed = path.replace(/\/+$/, '');
    const idx = trimmed.lastIndexOf('/');
    return idx <= 0 ? '/' : trimmed.slice(0, idx);
  }

  return (
    <div class={css.panel} data-testid="nc-picker">
      <div class={css.bar}>
        <button type="button" class={css.ghost} disabled={dir() === '/'} onClick={() => navigate(parentOf(dir()))}>
          Up
        </button>
        <span class={css.crumb} data-testid="nc-cwd">
          {dir()}
        </span>
      </div>

      <Show when={entries.loading}>
        <p class={css.meta}>Loading…</p>
      </Show>
      <Show when={entries.error as unknown}>
        <p class={css.error} role="alert">
          Could not list this folder.
        </p>
      </Show>

      <Show when={!entries.loading && (entries() ?? []).length === 0}>
        <p class={css.meta}>This folder is empty.</p>
      </Show>

      <Show when={(entries() ?? []).length > 0}>
        <ul class={css.list} role="listbox" aria-label="Nextcloud files">
          <For each={entries()}>
            {(entry) => (
              <li
                class={css.item}
                role="option"
                aria-selected={props.mode === 'files' && !entry.isDir && (props.selected?.has(entry.path) ?? false)}
                data-nc-path={entry.path}
                onClick={() => {
                  if (entry.isDir) {
                    navigate(entry.path);
                  } else if (props.mode === 'files') {
                    props.onToggleFile?.(entry);
                  }
                }}
              >
                <span class={css.dirIcon}>{entry.isDir ? '📁' : '📄'}</span>
                <span class={css.name}>{entry.name}</span>
                <Show when={!entry.isDir}>
                  <span class={css.size}>{humanSize(entry.size)}</span>
                </Show>
                <Show when={props.mode === 'files' && !entry.isDir && (props.selected?.has(entry.path) ?? false)}>
                  <span class={css.size} data-testid="nc-selected">✓</span>
                </Show>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </div>
  );
}

export default FilePicker;
