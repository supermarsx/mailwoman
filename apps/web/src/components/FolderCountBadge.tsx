import {Show,type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import {t} from '../i18n';
export function FolderCountBadge(props:{folder:{role?:string|null;unreadEmails:number;totalEmails:number}}):JSX.Element {
 const mode=()=>settings()['mail.folderCounts']?.value??'both';
 const compact=()=>settings()['mail.compactFolderBadges']?.value!==false;
 return <Show when={mode()!=='none'&&Number.isFinite(props.folder.totalEmails)}><span class="sidebar__badge" data-secondary={compact()&&props.folder.role!=='inbox'?'true':undefined} title={t('mail-folder-counts',{unread:props.folder.unreadEmails,total:props.folder.totalEmails})} aria-label={t('mail-folder-counts',{unread:props.folder.unreadEmails,total:props.folder.totalEmails})}><Show when={mode()!=='total'}>{props.folder.unreadEmails}</Show><Show when={mode()==='both'}><span class="sidebar__total"> / {props.folder.totalEmails}</span></Show><Show when={mode()==='total'}>{props.folder.totalEmails}</Show></span></Show>;
}
