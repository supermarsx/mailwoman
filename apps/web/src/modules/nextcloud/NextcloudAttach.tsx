// Attach-from-Nextcloud picker (SPEC §18.4, plan §3 e7): browse the linked Nextcloud,
// select one or more files, and hand the materialised attachments back to the composer.
// EXPORTED for e14 to wire into the compose attach menu.

import { createSignal, Show, type JSX } from 'solid-js';
import { NextcloudService, type Fetcher, type WebDavEntry, type AttachedFile } from './service.ts';
import { FilePicker } from './FilePicker.tsx';
import * as css from './styles.css.ts';

export interface NextcloudAttachProps {
  accountId?: string;
  /** The composer receives the materialised attachments. */
  onAttached: (files: AttachedFile[]) => void;
  fetcher?: Fetcher;
  service?: NextcloudService;
}

export function NextcloudAttach(props: NextcloudAttachProps): JSX.Element {
  const service = props.service ?? new NextcloudService(props.fetcher);
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  function toggle(entry: WebDavEntry): void {
    setSelected((cur) => {
      const next = new Set<string>(cur);
      if (next.has(entry.path)) next.delete(entry.path);
      else next.add(entry.path);
      return next;
    });
  }

  async function attach(): Promise<void> {
    setError('');
    const paths = [...selected()];
    if (paths.length === 0) {
      setError('select at least one file to attach');
      return;
    }
    setBusy(true);
    try {
      const files = await service.attach(paths);
      props.onAttached(files);
      setSelected(new Set<string>());
    } catch (e) {
      setError(e instanceof Error ? e.message : 'could not attach the selected files');
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class={css.panel} data-module="nextcloud" aria-label="Attach from Nextcloud">
      <h2 class={css.heading}>Attach from Nextcloud</h2>
      <FilePicker service={service} mode="files" selected={selected()} onToggleFile={toggle} />
      <button type="button" class={css.button} disabled={busy()} onClick={() => void attach()}>
        Attach {selected().size > 0 ? `${selected().size} file${selected().size === 1 ? '' : 's'}` : ''}
      </button>
      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </div>
  );
}

export default NextcloudAttach;
