import {InstanceSelector} from './InstanceSelector.tsx';
import {applyClick,extendByKey,EMPTY_SELECTION,type SelectionState} from '../../components/folderSelectionModel.ts';
import { InlineLoading } from '../../components/InlineLoading.tsx';
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
  disabled?:boolean;
  /** Selected file paths (files mode). */
  selected?: Set<string>;
  /** A file was toggled (files mode). */
  onToggleFile?: (entry: WebDavEntry) => void;
  /** The browsed directory changed (both modes). */
  onDirChange?: (path: string) => void;
  onInstanceChange?:()=>void;
}

export function FilePicker(props: FilePickerProps): JSX.Element {
  onMount(() => void loadCatalog('nextcloud'));
  const [dir, setDir] = createSignal('/');
  const [instance,setInstance]=createSignal('');
  const [entries] = createResource(()=>instance()?{path:dir(),instance:instance()}:undefined, ({path}) => props.service.list(path));

  const loaded=():WebDavEntry[]=>entries.loading||entries.error?[]:entries()??[];
  let geometry:SelectionState={...EMPTY_SELECTION};
  const fileIds=()=>loaded().filter(entry=>!entry.isDir).map(entry=>entry.path);
  function pick(next:SelectionState):void{
    const before=new Set(props.selected??[]);geometry=next;
    for(const entry of loaded())if(!entry.isDir&&before.has(entry.path)!==next.selected.has(entry.path))props.onToggleFile?.(entry);
  }
  function keydown(entry:WebDavEntry,event:KeyboardEvent):void{
    if(props.disabled||props.mode!=='files'||entry.isDir)return;
    const state={...geometry,selected:props.selected??new Set<string>()};
    if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='a'){event.preventDefault();event.stopPropagation();pick({...state,selected:new Set([...state.selected,...fileIds()]),rangeAdded:[]});}
    else if(event.shiftKey&&(event.key==='ArrowDown'||event.key==='ArrowUp')){const next=extendByKey(state,fileIds(),entry.path,event.key==='ArrowDown'?1:-1);if(next){event.preventDefault();event.stopPropagation();pick(next.state);const root=event.currentTarget instanceof Element?event.currentTarget.closest('ul'):null;[...root?.querySelectorAll<HTMLButtonElement>('[data-file-path]')??[]].find(row=>row.dataset.filePath===next.focus)?.focus();}}
  }

  function navigate(path: string): void {
    geometry={...EMPTY_SELECTION};
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
      <InstanceSelector disabled={props.disabled} service={props.service} onChange={id=>{geometry={...EMPTY_SELECTION};setDir('/');setInstance(id);props.onDirChange?.('/');props.onInstanceChange?.();}}/>
      <div class={css.bar}>
        <button
          type="button"
          class={css.ghost}
          disabled={props.disabled||dir() === '/'}
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
        <InlineLoading message={t('nextcloud-loading')}/>
      </Show>
      <Show when={entries.error as unknown}>
        <p class={css.error} role="alert">
          {t('nextcloud-list-error')}
        </p>
      </Show>

      <Show when={!!instance() && !entries.loading && !entries.error && loaded().length === 0}>
        <p class={css.meta}>{t('nextcloud-empty')}</p>
      </Show>

      <Show when={loaded().length > 0}>
        <ul class={css.list} aria-label={t('nextcloud-file-list')}>
          <For each={loaded()}>
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
                      disabled={props.disabled}
                      aria-label={
                        entry.isDir
                          ? t('nextcloud-open-folder', { name: isolate(entry.name) })
                          : t('nextcloud-select-file', { name: isolate(entry.name) })
                      }
                      aria-pressed={selectable() ? isSelected() : undefined}
                      data-file-path={entry.path}
                      onKeyDown={event=>keydown(entry,event)}
                      onClick={event => {
                        if(props.disabled)return;
                        if (entry.isDir) navigate(entry.path);
                        else pick(applyClick({...geometry,selected:props.selected??new Set<string>()},fileIds(),entry.path,{shift:event.shiftKey,toggle:true}));
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
