// Admin › Security › Console address (t76).
//
// The console answers at one path only. This page shows that path, says where it
// came from, and lets an admin with `security_policy.write` store a new one (their
// own, or a generated one). The server asks for step-up, writes an audit entry and
// applies the new path at its next start, which also ends every admin session.
import {createSignal,onMount,Show,type JSX} from 'solid-js';
import {consolePathApi,validChosenPath,type ConsolePathApi,type ConsolePathChange,type ConsolePathView} from '../../api/admin/consolePath.ts';
import {basePath} from '../../api/basePath.ts';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {loadCatalog,t} from '../../i18n';
import {ConfirmDialog} from './shell/ConfirmDialog.tsx';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';

function address(path:string):string{
  const origin=typeof location==='undefined'?'':location.origin;
  return `${origin}${basePath()}${path}`;
}

export function ConsolePath(props:{readOnly:boolean;api?:ConsolePathApi}):JSX.Element {
  const api=props.api??consolePathApi;
  const [view,setView]=createSignal<ConsolePathView>();
  const [etag,setEtag]=createSignal<string|null>(null);
  const [draft,setDraft]=createSignal('');
  const [busy,setBusy]=createSignal(false);
  const [error,setError]=createSignal('');
  const [saved,setSaved]=createSignal('');
  const [confirm,setConfirm]=createSignal<ConsolePathChange|null>(null);
  async function load():Promise<void>{
    setBusy(true);setError('');
    try{const r=await api.read();setView(r.data);setEtag(r.etag);}
    catch(cause){setError(describeAdminError(cause));}
    finally{setBusy(false);}
  }
  onMount(()=>{void loadCatalog('admin-security');void load();});
  const pinned=():boolean=>view()?.pinnedBy!=null;
  const min=():number=>view()?.minLength??12;
  const draftValid=():boolean=>validChosenPath(draft(),min());
  async function save():Promise<void>{
    const change=confirm(),tag=etag();
    if(!change||!tag||props.readOnly)return;
    setBusy(true);setError('');
    try{
      const r=await api.save(change,tag);
      setView(r.data);setEtag(r.etag);setDraft('');
      setSaved(t('admin-console-path-saved'));
    }catch(cause){setError(describeAdminError(cause));}
    finally{setBusy(false);setConfirm(null);}
  }
  return <section class={css.section} aria-label={t('admin-console-path-title')}>
    <h2 class={css.heading}>{t('admin-console-path-title')}</h2>
    <p class={css.note}>{t('admin-console-path-help')}</p>
    <p class={css.note}>{t('admin-console-path-limits')}</p>
    <Feedback kind="error" message={error()}/><Feedback kind="success" as="toast" message={saved()}/>
    <Show when={busy()&&!view()}><AsyncPending/></Show>
    <Show when={error()&&!view()}><button class="btn" onClick={()=>void load()}>{t('common-retry')}</button></Show>
    <Show when={view()}>{v=><div class={css.subsectionBody}>
      <p><strong>{t('admin-console-path-current')}</strong> <code data-testid="console-path-current">{address(v().path)}</code></p>
      <p class={css.note}>{t(`admin-console-path-source-${v().source}`)}</p>
      <Show when={v().legacyDefault}><p role="alert" class={css.error}>{t('admin-console-path-legacy')}</p></Show>
      <Show when={v().pending}>{p=><p role="status" data-testid="console-path-pending">{t('admin-console-path-pending',{address:address(p())})}</p>}</Show>
      <Show when={!pinned()&&!props.readOnly}>
        <form class={css.subsectionBody} onSubmit={event=>{event.preventDefault();if(draftValid())setConfirm({path:draft()});}}>
          <label class="field"><span>{t('admin-console-path-new')}</span>
            <input aria-label={t('admin-console-path-new')} type="text" autocomplete="off" spellcheck={false} value={draft()} placeholder="/" disabled={busy()} aria-invalid={draft()!==''&&!draftValid()} onInput={event=>{setDraft(event.currentTarget.value);setSaved('');}}/>
            <small>{t('admin-console-path-new-help',{min:min()})}</small>
          </label>
          <Show when={draft()!==''&&!draftValid()}><p role="alert" class={css.error}>{t('admin-console-path-invalid',{min:min()})}</p></Show>
          <div class={ux.pageActions}>
            <button type="button" class="btn" disabled={busy()||!etag()} onClick={()=>setConfirm({generate:true})}>{t('admin-console-path-generate')}</button>
            <button type="submit" class="btn btn--primary" disabled={busy()||!etag()||!draftValid()}>{t('admin-console-path-save')}</button>
          </div>
        </form>
      </Show>
    </div>}</Show>
    <ConfirmDialog open={confirm()!==null} title={t('admin-console-path-confirm')} body={t('admin-console-path-impact')} confirmLabel={t('admin-console-path-save')} onCancel={()=>setConfirm(null)} onConfirm={save}/>
  </section>;
}
