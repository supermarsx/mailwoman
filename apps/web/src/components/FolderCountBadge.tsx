import {Show,type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import {folderCountMode,folderCountScope,showsFolderCount} from '../api/countBadges.ts';
import {t} from '../i18n';
import {accountUiPreferences} from '../api/accountPreferences.ts';
import {appliedOverride} from './properties/folderOverrides.ts';
export type BadgeFolder={id?:string;role?:string|null;unreadEmails:number;totalEmails:number};
/** What the badge shows, in words: follows the count mode (what the number is), the role (Drafts counts messages, not unread) and the compact size. */
export function folderBadgeTip(folder:BadgeFolder,mode:string,compact:boolean):string {
 const args={unread:folder.unreadEmails,total:folder.totalEmails};
 const what=folder.role==='drafts'&&(mode==='total'||mode==='both')?t('mail-badge-drafts',args):mode==='total'?t('mail-badge-total',args):mode==='both'?t('mail-badge-both',args):t('mail-badge-unread',args);
 return compact&&folder.role!=='inbox'?`${what}. ${t('mail-badge-compact-hint')}`:what;
}
export function FolderCountBadge(props:{folder:BadgeFolder}):JSX.Element {
 // The folder's own override (Properties > Overrides) beats the setting, unless an administrator locked it.
 const own=<T,>(key:'counters'|'compactBadge'|'badgeColour'):T|undefined=>appliedOverride(key,props.folder.id,accountUiPreferences().folderProps) as T|undefined;
 const mode=()=>own<'none'|'unread'|'total'|'both'>('counters')??folderCountMode();
 const compact=()=>own<boolean>('compactBadge')??settings()['mail.compactFolderBadges']?.value!==false;
 const tip=()=>folderBadgeTip(props.folder,mode(),compact());
 const unreadOnly=()=>mode()==='unread'||mode()==='unreadIfAny';
 return <Show when={showsFolderCount(props.folder,mode(),folderCountScope())}><span class="sidebar__badge" data-secondary={compact()&&props.folder.role!=='inbox'?'true':undefined} style={own<string>('badgeColour')?{'--badge-bg':own<string>('badgeColour')!,background:own<string>('badgeColour')!}:undefined} title={tip()} data-tooltip={tip()} aria-label={tip()}><Show when={unreadOnly()||mode()==='both'}>{props.folder.unreadEmails}</Show><Show when={mode()==='both'}><span class="sidebar__total"> / {props.folder.totalEmails}</span></Show><Show when={mode()==='total'}>{props.folder.totalEmails}</Show></span></Show>;
}
