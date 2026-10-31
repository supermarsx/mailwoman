import {For} from 'solid-js';
import {WORKSPACE_LAYOUTS,type WorkspaceLayout} from '../realtime/workspaceLayout.ts';
import {t} from '../i18n/index.ts';

export function WorkspaceLayoutControls(props:{value:WorkspaceLayout;onChange:(value:WorkspaceLayout)=>void}) {
  return <label class="workspace-layout-modes"><span class="workspace-control-label">{t('mail-workspace-view-mode')}</span>
    <select aria-label={t('mail-workspace-view-mode')} value={props.value} onChange={event=>props.onChange(event.currentTarget.value as WorkspaceLayout)}>
      <For each={WORKSPACE_LAYOUTS}>{layout=><option value={layout}>{t(`mail-workspace-${layout}`)}</option>}</For>
    </select>
  </label>;
}
