import type {useApp} from '../state/context.ts';
import {smartArchivePolicy,SMART_ARCHIVE_STRATEGIES} from '../state/smartArchive.ts';
import {requestSettings} from '../state/settingsRequest.ts';
import {t} from '../i18n';
import type {ContextAction} from './ContextMenu.tsx';
export function smartArchiveAction(app:ReturnType<typeof useApp>,ids:string|string[]):ContextAction{
 const enabled=smartArchivePolicy().enabled;
 return {id:'smart-archive',label:t('mail-smart-archive'),icon:'smartArchive',run:()=>{},children:[
  {id:'smart-archive:default',label:t('mail-smart-archive-default'),icon:'smartArchive',disabled:!enabled,run:()=>app.smartArchiveMessage(ids)},
  ...SMART_ARCHIVE_STRATEGIES.map(strategy=>({id:`smart-archive:${strategy}`,label:t(`mail-smart-archive-strategy-${strategy}`),icon:'smartArchive' as const,disabled:!enabled,run:()=>app.smartArchiveMessage(ids,strategy)})),
  {id:'smart-archive:settings',label:t(enabled?'mail-smart-archive-settings':'mail-smart-archive-enable'),icon:'settings',run:()=>requestSettings('reader','settings-card-smart-archive')},
 ]};
}
