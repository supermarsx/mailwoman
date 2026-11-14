import {configuredSnoozePresets,FOLLOW_UP_PRESETS_KEY,resolveSnooze,snoozePresetLabel,snoozeZone,snoozeWeekStart,type SnoozePreset} from '../state/snooze.ts';
import {t} from '../i18n';
import {requestCustomFollowUp} from './SnoozeDialog.tsx';
import type {ContextAction} from './ContextMenu.tsx';
let cache:{rules:SnoozePreset[];minute:number;zone:string;week:number;rows:SnoozePreset[]}|undefined;
export function followUpActions(save:(until:string|null)=>Promise<void>,hasFollowUp=false,finish?:()=>Promise<void>):ContextAction[]{
 const rules=configuredSnoozePresets(FOLLOW_UP_PRESETS_KEY),minute=Math.floor(Date.now()/60000),zone=snoozeZone(),week=snoozeWeekStart();
 if(cache?.rules!==rules||cache.minute!==minute||cache.zone!==zone||cache.week!==week)cache={rules,minute,zone,week,rows:rules.filter(rule=>rule.enabled&&resolveSnooze(rule,new Date(),zone,week)!==null)};
 return [{id:'followup:custom',label:t('snooze-custom'),icon:'calendar',run:()=>requestCustomFollowUp(save)},...cache.rows.map(rule=>({id:`followup:${rule.id}`,label:snoozePresetLabel(rule),icon:'flag' as const,run:()=>{const at=resolveSnooze(rule);if(at)return save(at);}})),
  ...(hasFollowUp?[{id:'followup:clear',label:t('mail-clear-flag'),icon:'flag' as const,run:()=>save(null)}]:[]),...(hasFollowUp&&finish?[{id:'followup:finished',label:t('mail-mark-finished'),icon:'check' as const,run:finish}]:[])];
}
