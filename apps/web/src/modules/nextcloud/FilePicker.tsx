import { Icon } from '../../components/Icon.tsx';
// A minimal WebDAV directory picker (SPEC §18.4, plan §3 e7). Browses Nextcloud
// folders via the injected NextcloudService, letting the caller either navigate into
// directories and select files (attach) or pick a destination directory (save-to).
// Presentational + navigation only — the caller decides what "select" means.

import { createSignal, createResource, onMount, For, Show, type JSX } from 'solid-js';
import { NextcloudService, type WebDavEntry } from './service.ts';
import { t, loadCatalog, isolate, formatBytes } from '../../i18n';
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

export function FilePicker(props: FilePickerProps): JSX.Element {
  onMount(() => void loadCatalog('nextcloud'));
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
        <button
          type="button"
          class={css.ghost}
          disabled={dir() === '/'}
          aria-label={t('nextcloud-up')}
          onClick={() => navigate(parentOf(dir()))}
        >
          {t('nextcloud-up')}
        </button>
        <span class={css.crumb} data-testid="nc-cwd">
          {dir()}
        </span>
      </div>

      <Show when={entries.loading}>
        <p class={css.meta}>{t('nextcloud-loading')}</p>
      </Show>
      <Show when={entries.error as unknown}>
        <p class={css.error} role="alert">
          {t('nextcloud-list-error')}
        </p>
      </Show>

      <Show when={!entries.loading && (entries() ?? []).length === 0}>
        <p class={css.meta}>{t('nextcloud-empty')}</p>
      </Show>

      <Show when={(entries() ?? []).length > 0}>
        <ul class={css.list} aria-label={t('nextcloud-file-list')}>
          <For each={entries()}>
            {(entry) => {
              const selectable = (): boolean => props.mode === 'files' && !entry.isDir;
              const isSelected = (): boolean => selectable() && (props.selected?.has(entry.path) ?? false);
              const interactive = entry.isDir || selectable();
              const content = (): JSX.Element => (
                <>
                  <span class={css.dirIcon} aria-hidden="true">
                    <Icon name={entry.isDir ? 'folder' : 'file'} />
                  </span>
                  <span class={css.name}>{entry.name}</span>
                  <Show when={!entry.isDir}>
                    <span class={css.size}>{formatBytes(entry.size)}</span>
                  </Show>
                  <Show when={isSelected()}>
                    <span class={css.size} data-testid="nc-selected" aria-hidden="true">
                      <Icon name="check" />
                    </span>
                  </Show>
                </>
              );
              return (
                <li class={css.item} data-nc-path={entry.path}>
                  <Show when={interactive} fallback={<div class={css.rowStatic}>{content()}</div>}>
                    <button
                      type="button"
                      class={css.row}
                      aria-label={
                        entry.isDir
                          ? t('nextcloud-open-folder', { name: isolate(entry.name) })
                          : t('nextcloud-select-file', { name: isolate(entry.name) })
                      }
                      aria-pressed={selectable() ? isSelected() : undefined}
                      onClick={() => {
                        if (entry.isDir) navigate(entry.path);
                        else props.onToggleFile?.(entry);
                      }}
                    >
                      {content()}
                    </button>
                  </Show>
                </li>
              );
            }}
          </For>
        </ul>
      </Show>
    </div>
  );
}

export default FilePicker;
