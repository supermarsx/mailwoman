import {settings,type SettingEntry} from '../api/settings.ts';
import {accountUiPreferences} from '../api/accountPreferences.ts';
import {effectiveOverride,OVERRIDE_DEFS,type FolderProps} from './properties/folderOverrides.ts';
import {allowedOverrides,type OverrideKey} from './properties/policy.ts';

export type PreviewLines=0|1|2|3|4|5;
export function resolvePreviewLines(read:(key:string)=>SettingEntry|undefined,folder?:FolderProps,allowed:readonly OverrideKey[]=[]):PreviewLines {
 const legacy=read('mail.messageListPreview'),entry=read('mail.messageListPreviewLines');
 // Preserve saved on/off preferences from older clients until explicitly changed.
 if(legacy?.value===false&&(legacy.locked||(!entry?.locked&&entry?.source!=='user')))return 0;
 const value=effectiveOverride(OVERRIDE_DEFS.find(def=>def.key==='previewLines')!,folder,read,allowed).value;
 return typeof value==='number'&&Number.isInteger(value)&&value>=0&&value<=5?value as PreviewLines:1;
}
export function messagePreviewLines(folderId?:string):PreviewLines {
 return resolvePreviewLines(key=>settings()[key],folderId?accountUiPreferences().folderProps?.[folderId]:undefined,allowedOverrides());
}
