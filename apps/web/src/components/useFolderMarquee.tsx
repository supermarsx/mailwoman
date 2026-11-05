// Sidebar folder marquee: wires attachMarquee to the folder panel, keeps the
// shared folder selection in sync with what is visible, and renders the
// polite live region that announces the count.
import { createEffect, createSignal, onCleanup, onMount, on, type JSX } from 'solid-js';
import { loadCatalog, t } from '../i18n';
import { attachMarquee } from './marqueeSelect.ts';
import { clearSelectedFolders, pruneSelectedFolders, selectedFolderIds, setSelectedFolders } from './folderSelection.ts';
import { srOnly } from './mailA11y.css.ts';

/** `visibleIds` lists the folders currently shown (collapsed children excluded); selected ids outside it are dropped. */
export function useFolderMarquee(visibleIds: () => readonly string[]): { ref: (el: HTMLElement) => void; liveRegion: () => JSX.Element } {
  const [announcement, setAnnouncement] = createSignal('');
  onMount(() => { void loadCatalog('folder-select'); });
  const attach = (el: HTMLElement): void => {
    onMount(() => {
      const items = () => Array.from(el.querySelectorAll<HTMLElement>('[data-folder-region] [data-marquee-item]')).map(node => ({ id: node.dataset.folderId ?? '', el: node })).filter(item => item.id);
      const detach = attachMarquee({ container: el, items, selection: selectedFolderIds, onChange: setSelectedFolders, onEmptyClick: clearSelectedFolders });
      onCleanup(detach);
    });
  };
  createEffect(() => pruneSelectedFolders(new Set(visibleIds())));
  createEffect(on(() => selectedFolderIds().size, (count, before) => {
    // Debounced so a drag that sweeps many rows announces its result once.
    const timer = setTimeout(() => {
      if (count > 0) setAnnouncement(t('folder-select-count', { count }));
      else if (before) setAnnouncement(t('folder-select-cleared'));
    }, 350);
    onCleanup(() => clearTimeout(timer));
  }, { defer: true }));
  return { ref: attach, liveRegion: () => <p class={srOnly} role="status" aria-live="polite" data-testid="folder-select-status">{announcement()}</p> };
}
