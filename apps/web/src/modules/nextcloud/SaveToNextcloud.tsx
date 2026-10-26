// Save-attachment-to-Nextcloud (SPEC §18.4, plan §3 e7): browse to a destination
// folder and save a message attachment (by blob id) there. EXPORTED for e14 to wire
// into the attachment context menu.

import { createSignal, onMount, Show, type JSX } from 'solid-js';
import { NextcloudService, NextcloudRequestError, type Fetcher, type WebDavEntry } from './service.ts';
import { FilePicker } from './FilePicker.tsx';
import { t, loadCatalog, isolate } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import * as css from './styles.css.ts';

export interface SaveToNextcloudProps {
  accountId?:string;
  /** The message attachment to save. */
  attachment: { emailId: string; blobId: string; name: string };
  onSaved?: (entry: WebDavEntry) => void;
  fetcher?: Fetcher;
  service?: NextcloudService;
}

export function SaveToNextcloud(props: SaveToNextcloudProps): JSX.Element {
  onMount(() => void loadCatalog('nextcloud'));
  const service = (props.service ?? new NextcloudService(props.fetcher)).forAccount(props.accountId);
  const [dir, setDir] = createSignal('/');
  const [ready,setReady]=createSignal(false);
  const [warning,setWarning]=createSignal(false);const [acknowledged,setAcknowledged]=createSignal(false);
  const [busy, setBusy] = createSignal(false);

  // The result is a toast (t32 f39): "Saved to …" or why it failed.
  async function save(): Promise<void> {
    setBusy(true);
    try {
      const entry: WebDavEntry = await service.saveTo(props.attachment.blobId, dir(), props.attachment.emailId,acknowledged());
      toast.success(t('nextcloud-saved', { path: isolate(entry.path) }), { testId: 'nc-saved' });
      props.onSaved?.(entry);
    } catch (e) {
      if(e instanceof NextcloudRequestError&&e.code==='attachmentWarningRequired'){setWarning(true);return;}
      toast.error(e instanceof Error ? e.message : t('nextcloud-error-save-failed'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class={css.panel} data-module="nextcloud" aria-label={t('nextcloud-save-panel-label')}>
      <h2 class={css.heading}>{t('nextcloud-save-title', { name: isolate(props.attachment.name) })}</h2>
      <FilePicker disabled={busy()} service={service} mode="dirs" onDirChange={setDir} onInstanceChange={()=>{setReady(true);setWarning(false);setAcknowledged(false);}} />
      <Show when={warning()}><p role="alert">{t('nextcloud-save-warning')}</p><label><input type="checkbox" checked={acknowledged()} onChange={event=>setAcknowledged(event.currentTarget.checked)}/>{t('nextcloud-save-acknowledge')}</label></Show>
      <button type="button" class={css.button} disabled={busy()||!ready()||(warning()&&!acknowledged())} onClick={() => void save()}>
        {t('nextcloud-save-action', { dir: isolate(dir()) })}
      </button>
    </div>
  );
}

export default SaveToNextcloud;
