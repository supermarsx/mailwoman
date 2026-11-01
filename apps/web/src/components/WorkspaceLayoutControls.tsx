import {WORKSPACE_LAYOUTS,type WorkspaceLayout} from '../realtime/workspaceLayout.ts';
import {t} from '../i18n/index.ts';
import {IconSelect} from './IconSelect.tsx';
import type {IconName} from './Icon.tsx';

const icons:Record<WorkspaceLayout,IconName>={tabs:'layoutTabs','side-by-side':'layoutColumns',stack:'layoutRows','masonry-side-by-side':'layoutMasonryColumns','masonry-stack':'layoutMasonryRows'};

export function WorkspaceLayoutControls(props:{value:WorkspaceLayout;onChange:(value:WorkspaceLayout)=>void}) {
  return <IconSelect label={t('mail-workspace-view-mode')} value={props.value} options={WORKSPACE_LAYOUTS.map(layout=>({value:layout,label:t(`mail-workspace-${layout}`),icon:icons[layout]}))} onChange={value=>props.onChange(value as WorkspaceLayout)}/>;
}
