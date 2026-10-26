import {For} from 'solid-js';
import {WORKSPACE_LAYOUTS,type WorkspaceLayout} from '../realtime/workspaceLayout.ts';
import {t} from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';

const shapes:Record<WorkspaceLayout,string[]>={
  tabs:['M3 7h18v14H3z','M3 7V3h6v4','M9 3h6v4','M15 3h6v4'],
  'side-by-side':['M3 3h18v18H3z','M12 3v18'],
  stack:['M3 3h18v18H3z','M3 12h18'],
  'masonry-side-by-side':['M3 3h18v18H3z','M12 3v18','M3 15h9','M12 9h9'],
  'masonry-stack':['M3 3h18v18H3z','M3 12h18','M15 3v9','M9 12v9'],
};
export function WorkspaceLayoutControls(props:{value:WorkspaceLayout;onChange:(value:WorkspaceLayout)=>void}) {
  return <div class="workspace-layout-modes" role="group" aria-label={t('mail-workspace-view-mode')}>
    <For each={WORKSPACE_LAYOUTS}>{layout=><button type="button" class={`btn btn--ghost workspace-layout-mode ${a11y.iconButton}`}
      aria-label={t(`mail-workspace-${layout}`)} aria-pressed={props.value===layout}
      data-tooltip={t(`mail-workspace-${layout}-help`)} onClick={()=>props.onChange(layout)}>
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round" aria-hidden="true"><For each={shapes[layout]}>{path=><path d={path}/>}</For></svg>
    </button>}</For>
  </div>;
}
