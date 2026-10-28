import {createSignal,Show,type JSX} from 'solid-js';
import {useApp} from '../state/context.ts';
import {t} from '../i18n';
import {Icon} from './Icon.tsx';
import {EmptyFolderDialog,useCanEmpty,useEmptyAbility} from './EmptyFolderDialog.tsx';

/** The Trash toolbar's "Empty trash": the general empty-folder flow (t45) on Trash. */
export function EmptyTrash():JSX.Element {
 const app=useApp(),ability=useEmptyAbility(),can=useCanEmpty();const [open,setOpen]=createSignal(false);
 const folder=()=>app.mailboxes().find(box=>box.id===app.selectedMailboxId()&&box.role==='trash');
 return <Show when={folder()}>{box=><Show when={can(box())&&ability()}>{value=><><button type="button" class="btn btn--ghost" aria-label={t('mail-empty-trash')} title={t('mail-trash-help')} onClick={()=>setOpen(true)}><Icon name="delete"/></button><Show when={open()}><EmptyFolderDialog box={box()} ability={value()} onClose={()=>setOpen(false)}/></Show></>}</Show>}</Show>;
}
