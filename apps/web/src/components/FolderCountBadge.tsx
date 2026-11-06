import {Show,type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import {folderCountMode,folderCountScope,showsFolderCount} from '../api/countBadges.ts';
import {t} from '../i18n';
export type BadgeFolder={role?:string|null;unreadEmails:number;totalEmails:number};
/** What the badge shows, in words: follows the count mode (what the number is), the role (Drafts counts messages, not unread) and the compact size. */
export function folderBadgeTip(folder:BadgeFolder,mode:string,compact:boolean):string {
 const args={unread:folder.unreadEmails,total:folder.totalEmails};
 const what=folder.role==='drafts'&&(mode==='total'||mode==='both')?t('mail-badge-drafts',args):mode==='total'?t('mail-badge-total',args):mode==='both'?t('mail-badge-both',args):t('mail-badge-unread',args);
 return compact&&folder.role!=='inbox'?`${what}. ${t('mail-badge-compact-hint')}`:what;
}
export function FolderCountBadge(props:{folder:BadgeFolder}):JSX.Element {
 const mode=folderCountMode;
 const compact=()=>settings()['mail.compactFolderBadges']?.value!==false;
 const tip=()=>folderBadgeTip(props.folder,mode(),compact());
 const unreadOnly=()=>mode()==='unread'||mode()==='unreadIfAny';
 return <Show when={showsFolderCount(props.folder,mode(),folderCountScope())}><span class="sidebar__badge" data-secondary={compact()&&props.folder.role!=='inbox'?'true':undefined} title={tip()} data-tooltip={tip()} aria-label={tip()}><Show when={unreadOnly()||mode()==='both'}>{props.folder.unreadEmails}</Show><Show when={mode()==='both'}><span class="sidebar__total"> / {props.folder.totalEmails}</span></Show><Show when={mode()==='total'}>{props.folder.totalEmails}</Show></span></Show>;
}
