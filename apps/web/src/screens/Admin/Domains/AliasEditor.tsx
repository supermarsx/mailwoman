import {createSignal,Show,type JSX} from 'solid-js';
import {t} from '../../../i18n';
import {useDirtyGuard} from '../shell/useDirtyGuard.ts';
import {describeAdminError} from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';

export function AliasEditor(props:{source:string;mode:'new'|'edit'|'clone';save:(value:string)=>Promise<void>;close:()=>void}):JSX.Element {
 const [value,setValue]=createSignal(props.source),[baseline,setBaseline]=createSignal(props.source),[busy,setBusy]=createSignal(false),[error,setError]=createSignal('');
 useDirtyGuard(()=>value()!==baseline());
 async function submit(event:SubmitEvent):Promise<void>{event.preventDefault();setBusy(true);setError('');try{await props.save(value().trim());setBaseline(value());props.close();}catch(error){setError(describeAdminError(error));}finally{setBusy(false);}}
 return <form class={css.card} onSubmit={event=>void submit(event)} aria-label={t(`domain-alias-${props.mode}`)}><h4>{t(`domain-alias-${props.mode}`)}</h4><p>{t('domain-alias-editor-help')}</p><Show when={error()}><p role="alert">{error()}</p></Show><label class="field"><span>{t('domain-alias-name')}</span><input required value={value()} disabled={busy()} onInput={event=>setValue(event.currentTarget.value)}/></label><div class={css.headingRow}><button type="submit" class="btn btn--primary" disabled={busy()||!value().trim()||value().trim()===props.source}>{t('common-save')}</button><button type="button" class="btn" disabled={busy()} onClick={props.close}>{t('common-cancel')}</button></div></form>;
}
