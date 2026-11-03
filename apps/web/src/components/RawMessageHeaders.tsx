import {createEffect,createSignal,onCleanup,Show,type JSX} from 'solid-js';
import {fetchMailHeaders} from '../api/mailHeaders.ts';
import {t} from '../i18n/index.ts';
import {SecurityPopup} from './SecurityPanel.tsx';
import {Icon} from './Icon.tsx';
import * as css from './rawMessageHeaders.css.ts';

/** Fetch only the bounded header block, on demand. Message text and attachments
 * never enter the dialog; untrusted header values are displayed as plain text. */
export function RawMessageHeaders(props:{url:string;scope:string;onClose:()=>void}):JSX.Element {
  const [raw,setRaw]=createSignal<string|null>(null);
  const [loading,setLoading]=createSignal(true);
  const [failed,setFailed]=createSignal(false);
  const [copied,setCopied]=createSignal<'idle'|'copied'|'failed'>('idle');
  const [retry,setRetry]=createSignal(0);
  let generation=0;
  createEffect(()=>{
    const url=props.url;void props.scope;void retry();
    const epoch=++generation,controller=new AbortController();
    setRaw(null);setLoading(true);setFailed(false);setCopied('idle');
    const timeout=setTimeout(()=>{controller.abort();if(epoch===generation){setLoading(false);setFailed(true);}},10000);
    onCleanup(()=>{generation++;clearTimeout(timeout);controller.abort();});
    void fetchMailHeaders(url,controller.signal).then(headers=>{
      if(epoch===generation&&!controller.signal.aborted)setRaw(headers);
    },()=>{if(epoch===generation&&!controller.signal.aborted)setFailed(true);}).finally(()=>{
      clearTimeout(timeout);if(epoch===generation&&!controller.signal.aborted)setLoading(false);
    });
  });
  async function copy():Promise<void>{
    const value=raw(),epoch=generation;if(value===null)return;
    try{await navigator.clipboard.writeText(value);if(epoch===generation)setCopied('copied');}
    catch{if(epoch===generation)setCopied('failed');}
  }
  return <SecurityPopup title={t('mail-headers-title')} onClose={props.onClose}>
    <div class={css.sheet} aria-busy={loading()}>
      <p class={css.note}>{t('mail-headers-help')}</p>
      <Show when={loading()}><p role="status">{t('mail-headers-loading')}</p></Show>
      <Show when={failed()}><p role="alert">{t('mail-headers-failed')}</p><div><button type="button" class="btn btn--ghost" onClick={()=>setRetry(value=>value+1)}><Icon name="refresh"/>{t('common-retry')}</button></div></Show>
      <Show when={raw()!==null}><div class={css.actions}><button type="button" class="btn btn--ghost" onClick={()=>void copy()}><Icon name="clone"/>{t('mail-headers-copy')}</button><Show when={copied()!=='idle'}><span role="status">{t(copied()==='copied'?'mail-headers-copied':'mail-headers-copy-failed')}</span></Show></div><pre class={css.headers} aria-label={t('mail-headers-title')} tabIndex={0} dir="ltr">{raw()}</pre></Show>
    </div>
  </SecurityPopup>;
}
