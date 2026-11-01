import {For} from 'solid-js';
import {WORKSPACE_LAYOUTS,type WorkspaceLayout} from '../realtime/workspaceLayout.ts';
import {t} from '../i18n/index.ts';
import {IconSelect} from './IconSelect.tsx';
import type {IconName} from './Icon.tsx';

const icons:Record<WorkspaceLayout,IconName>={tabs:'layoutTabs','side-by-side':'layoutColumns',stack:'layoutRows','masonry-side-by-side':'layoutMasonryColumns','masonry-stack':'layoutMasonryRows'};

export function WorkspaceLayoutControls(props:{value:WorkspaceLayout;onChange:(value:WorkspaceLayout)=>void}) {
  return <IconSelect label={t('mail-workspace-view-mode')} currentLabel={t(`mail-workspace-${props.value}`)} icon={icons[props.value]} value={props.value} onChange={value=>props.onChange(value as WorkspaceLayout)}>
      <For each={WORKSPACE_LAYOUTS}>{layout=><option value={layout}>{t(`mail-workspace-${layout}`)}</option>}</For>
  </IconSelect>;
}
