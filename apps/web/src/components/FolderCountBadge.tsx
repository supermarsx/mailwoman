import {Show,type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import {folderCountMode,folderCountScope,showsFolderCount} from '../api/countBadges.ts';
import {t} from '../i18n';
export function FolderCountBadge(props:{folder:{role?:string|null;unreadEmails:number;totalEmails:number}}):JSX.Element {
 const mode=folderCountMode;
 const compact=()=>settings()['mail.compactFolderBadges']?.value!==false;
 const unreadOnly=()=>mode()==='unread'||mode()==='unreadIfAny';
 return <Show when={showsFolderCount(props.folder,mode(),folderCountScope())}><span class="sidebar__badge" data-secondary={compact()&&props.folder.role!=='inbox'?'true':undefined} title={t('mail-folder-counts',{unread:props.folder.unreadEmails,total:props.folder.totalEmails})} aria-label={t('mail-folder-counts',{unread:props.folder.unreadEmails,total:props.folder.totalEmails})}><Show when={unreadOnly()||mode()==='both'}>{props.folder.unreadEmails}</Show><Show when={mode()==='both'}><span class="sidebar__total"> / {props.folder.totalEmails}</span></Show><Show when={mode()==='total'}>{props.folder.totalEmails}</Show></span></Show>;
}
