import { createResource, For, Show, type JSX } from 'solid-js';
import { listAdminSettings } from '../../api/admin/settings.ts';
import { loadCatalog, t } from '../../i18n';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { AdminSubsection } from './AdminSubsection.tsx';
import { useShell } from './shell/context.ts';
import { hrefFor, sectionPath } from './shell/router.ts';
import { describeAdminError } from './shell/Feedback.tsx';
import { SourceBadge } from './shell/SourceBadge.tsx';
import { sourceOf } from './Settings/SettingDetail.tsx';
import { settingHelp, settingLabel } from './Settings/labels.ts';
import { displayValue } from './Settings/value.ts';
import * as css from './admin.css.ts';
import * as layout from './attachmentPolicy.css.ts';

export function AdminAttachmentPolicy(props:{load?:typeof listAdminSettings}):JSX.Element {
  void loadCatalog('admin-settings');
  const shell=useShell();
  const [settings,{refetch}]=createResource(()=>shell.can.can('settings.read'),async()=>await (props.load??listAdminSettings)());
  const rows=()=>{const result=settings.state==='ready'?settings.latest:undefined;return result?.kind==='ok' ? result.data.filter(row=>row.key.startsWith('attachments.')):[];};
  return <section class={css.section}>
    <h2>{t('admin-attachments-title')}</h2><p>{t('admin-attachments-description')}</p>
    <AdminSubsection title={t('admin-attachments-policy-title')} actions={<button class="btn" disabled={settings.loading} onClick={()=>void refetch()}>{t('admin-attachments-refresh')}</button>}>
      <Show when={settings.loading}><AsyncPending/></Show>
      <Show when={settings.error}><p role="alert">{describeAdminError(settings.error)}</p></Show>
      <Show when={settings.state==='ready' && settings()?.kind!=='ok'}><p role="alert">{t('admin-attachments-unavailable')}</p></Show>
      <For each={rows()}>{row=><div class={layout.row}>
        <div class={layout.content}><h3>{settingLabel(row.key)}</h3><p>{settingHelp(row.key)}</p><code>{displayValue(row.kind,row.effective.value)}</code> <SourceBadge source={sourceOf(row)}/></div>
        <a class={`btn ${layout.action}`} href={hrefFor(sectionPath('settings',[row.key]))} onClick={event=>{event.preventDefault();void shell.router.navigate(sectionPath('settings',[row.key]));}}>{t('admin-attachments-edit')}</a>
      </div>}</For>
    </AdminSubsection>
  </section>;
}
