import {AdminSubtabPanel} from './shell/AdminSubtabPanel.tsx';
import {lazy,onMount,Switch,Match,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {useShell} from './shell/context.ts';
import {SectionTabs} from './shell/SectionTabs.tsx';
import {sectionPath} from './shell/router.ts';
import {MaintenanceMode} from './MaintenanceMode.tsx';
import {CleanupMaintenance} from './CleanupMaintenance.tsx';
import {RethreadMaintenance} from './RethreadMaintenance.tsx';
import * as ux from './shell/ux.css.ts';
const DatabaseMaintenance=lazy(()=>import('./DatabaseMaintenance.tsx').then(module=>({default:module.DatabaseMaintenance})));
const CacheMaintenance=lazy(()=>import('./CacheMaintenance.tsx').then(module=>({default:module.CacheMaintenance})));
export function Maintenance():JSX.Element {
  const shell=useShell();onMount(()=>void loadCatalog('admin-maintenance'));
  const selected=()=>{
    const tab=shell.router.route().rest[0];
    if(shell.can.can('maintenance.read')&&(tab==='database'||tab==='cache'))return tab;
    if(shell.can.can('maintenance.run')&&(tab==='jobs'||!shell.can.can('maintenance.read')))return 'jobs';
    return 'mode';
  };
  return <div class={ux.page}><SectionTabs label={t('admin-nav-rethread')} selected={selected()} items={[
    ...(shell.can.can('maintenance.read')?[{id:'mode',label:t('maintenance-mode-title'),path:sectionPath('maintenance')}]:[]),
    ...(shell.can.can('maintenance.read')?[{id:'database',label:t('maintenance-tab-database'),path:sectionPath('maintenance',['database'])},{id:'cache',label:t('maintenance-tab-cache'),path:sectionPath('maintenance',['cache'])}]:[]),
    ...(shell.can.can('maintenance.run')?[{id:'jobs',label:t('maintenance-mode-jobs'),path:sectionPath('maintenance',['jobs'])}]:[]),
  ]} navigate={path=>{void shell.router.navigate(path);}}/>
    <AdminSubtabPanel><Switch>
      <Match when={selected()==='database'}><DatabaseMaintenance/></Match>
      <Match when={selected()==='cache'}><CacheMaintenance/></Match>
      <Match when={selected()==='jobs'}><CleanupMaintenance/><RethreadMaintenance/></Match>
      <Match when={shell.can.can('maintenance.read')}><MaintenanceMode readOnly={!shell.can.can('maintenance.write')}/></Match>
    </Switch></AdminSubtabPanel>
  </div>;
}
