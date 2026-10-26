import {onMount,Show,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useShell} from './shell/context.ts';
import {SectionTabs} from './shell/SectionTabs.tsx';
import {sectionPath} from './shell/router.ts';
import {MaintenanceMode} from './MaintenanceMode.tsx';
import {RethreadMaintenance} from './RethreadMaintenance.tsx';
import * as ux from './shell/ux.css.ts';
export function Maintenance():JSX.Element {
  const shell=useShell();onMount(()=>void loadCatalog('admin-maintenance'));
  const jobs=()=>shell.can.can('maintenance.run')&&(shell.router.route().rest[0]==='jobs'||!shell.can.can('maintenance.read'));
  return <div class={ux.page}><SectionTabs label={t('admin-nav-rethread')} selected={jobs()?'jobs':'mode'} items={[
    ...(shell.can.can('maintenance.read')?[{id:'mode',label:t('maintenance-mode-title'),path:sectionPath('maintenance')}]:[]),
    ...(shell.can.can('maintenance.run')?[{id:'jobs',label:t('maintenance-mode-jobs'),path:sectionPath('maintenance',['jobs'])}]:[]),
  ]} navigate={path=>{void shell.router.navigate(path);}}/>
    <Show when={jobs()} fallback={<Show when={shell.can.can('maintenance.read')}><MaintenanceMode readOnly={!shell.can.can('maintenance.write')}/></Show>}><RethreadMaintenance/></Show>
  </div>;
}
