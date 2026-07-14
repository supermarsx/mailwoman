// Save-attachment-to-Nextcloud (SPEC §18.4, plan §3 e7): browse to a destination
// folder and save a message attachment (by blob id) there. EXPORTED for e14 to wire
// into the attachment context menu.

import { createSignal, Show, type JSX } from 'solid-js';
import { NextcloudService, type Fetcher, type WebDavEntry } from './service.ts';
import { FilePicker } from './FilePicker.tsx';
import * as css from './styles.css.ts';

export interface SaveToNextcloudProps {
  /** The message attachment to save. */
  attachment: { blobId: string; name: string };
  onSaved?: (entry: WebDavEntry) => void;
  fetcher?: Fetcher;
  service?: NextcloudService;
}

export function SaveToNextcloud(props: SaveToNextcloudProps): JSX.Element {
  const service = props.service ?? new NextcloudService(props.fetcher);
  const [dir, setDir] = createSignal('/');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [saved, setSaved] = createSignal<WebDavEntry | null>(null);

  async function save(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const entry = await service.saveTo(props.attachment.blobId, dir(), props.attachment.name);
      setSaved(entry);
      props.onSaved?.(entry);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'could not save to Nextcloud');
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class={css.panel} data-module="nextcloud" aria-label="Save to Nextcloud">
      <h2 class={css.heading}>Save “{props.attachment.name}” to Nextcloud</h2>
      <FilePicker service={service} mode="dirs" onDirChange={setDir} />
      <button type="button" class={css.button} disabled={busy()} onClick={() => void save()}>
        Save here ({dir()})
      </button>
      <Show when={saved()}>
        {(entry) => <p class={css.meta} data-testid="nc-saved">Saved to {entry().path}</p>}
      </Show>
      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </div>
  );
}

export default SaveToNextcloud;
