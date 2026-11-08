// Small helpers shared by the folder dialog's header and tabs.
import {t} from '../../i18n';
import {toast} from '../toast/toast.ts';
import type {Mailbox} from '../../api/jmap-types.ts';

export const dateText=(iso:string|null|undefined)=>{if(!iso)return t('props-none');const d=new Date(iso);return Number.isNaN(d.getTime())?iso:d.toLocaleDateString(undefined,{year:'numeric',month:'short',day:'numeric'});};
export const dateTimeText=(iso:string|null|undefined)=>{if(!iso)return t('props-none');const d=new Date(iso);return Number.isNaN(d.getTime())?iso:d.toLocaleString();};

/** Copy `text`, then say so (or say it failed). */
export async function copyText(text:string,done=t('props-act-copied')):Promise<void> {
 try{await navigator.clipboard.writeText(text);toast.success(done);}catch{toast.error(t('props-act-copy-failed'));}
}

/** The folder's ancestors then itself, top first. */
export function pathOf(boxes:readonly Mailbox[],box:Mailbox):Mailbox[] {
 const chain:Mailbox[]=[box],seen=new Set([box.id]);
 let at:Mailbox|undefined=box;
 while(at?.parentId&&!seen.has(at.parentId)){seen.add(at.parentId);at=boxes.find(value=>value.id===at!.parentId);if(at)chain.unshift(at);}
 return chain;
}

const ROLE_KNOWN=new Set(['inbox','drafts','sent','junk','trash','archive']);
export function folderTypeKey(box:Pick<Mailbox,'role'|'mailwomanSearchQuery'>):string {
 if(box.mailwomanSearchQuery!==undefined)return 'virtual';
 return box.role&&ROLE_KNOWN.has(box.role)?box.role:'user';
}
