import { createResource, For, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { fetchDeliveryReport, fetchProxyDeliveryReport } from '../../api/dsn.ts';
import { isolate, loadCatalog, t } from '../../i18n/index.ts';
import './deliveryStatus.css';
import { deliveryStatusMeaning } from './deliveryStatusMeaning.ts';

export function DeliveryStatus(props: {accountId: string; emailId: string; enabled: boolean; proxyUrl?: string | undefined}): JSX.Element {
  onMount(()=>void loadCatalog('mail-dsn'));
  let currentAbort: AbortController | undefined;
  onCleanup(()=>currentAbort?.abort());
  const [report] = createResource(
    ()=>props.enabled || props.proxyUrl ? {account:props.accountId,email:props.emailId,engine:props.enabled,url:props.proxyUrl} : null,
    async key=>{
      currentAbort?.abort(); const abort = new AbortController(); currentAbort=abort;
      const timeout=setTimeout(()=>abort.abort(),15000);
      try { return {...key,report:key.engine ? await fetchDeliveryReport(key.account,key.email) : await fetchProxyDeliveryReport(key.url!,abort.signal)}; }
      catch { return {...key,report:null}; }
      finally { clearTimeout(timeout); }
    },
  );
  const current = ()=>{
    const result = report.state==='ready' ? report.latest : undefined;
    return result?.account===props.accountId && result.email===props.emailId && result.engine===props.enabled && result.url===props.proxyUrl ? result.report : null;
  };
  return <Show when={current()}>{value=><section class="delivery-status" aria-label={t('dsn-title')}>
    <h3>{t('dsn-title')}</h3><p>{t('dsn-unverified')}</p>
    <p>{t('dsn-reporting-mta')} <bdi>{isolate(value().reportingMta)}</bdi></p>
    <For each={value().recipients}>{row=><details class="delivery-status__recipient" open={row.action==='failed'||row.action==='delayed'}>
      <summary><strong>{t(`dsn-action-${row.action}`)}</strong> <bdi>{isolate(row.finalRecipient)}</bdi> <code>{row.status}</code></summary>
      <Show when={deliveryStatusMeaning(row.status,row)}>{meaning=><div class="delivery-status__meaning">
        <h4>{t('dsn-meaning-title')}</h4>
        <Show when={meaning().provider}>{provider=><p class="delivery-status__provider">{t('dsn-provider-detected')} <strong>{t(provider().providerKey)}</strong></p>}</Show>
        <p>{t(meaning().classKey)}</p>
        <p>{t(meaning().reasonKey)}</p>
        <p><strong>{t('dsn-next-step')}</strong> {t(meaning().adviceKey)}</p>
        <Show when={meaning().provider}>{provider=><p><a href={provider().sourceUrl} target="_blank" rel="noopener noreferrer">{t('dsn-provider-help')}</a></p>}</Show>
      </div>}</Show>
      <Show when={row.originalRecipient}><p>{t('dsn-original-recipient')} <bdi>{isolate(row.originalRecipient!)}</bdi></p></Show>
      <Show when={row.diagnostic}><div class="delivery-status__diagnostic"><strong>{t('dsn-server-diagnostic')}</strong><p>{isolate(row.diagnostic!)}</p></div></Show>
      <Show when={row.remoteMta}><p>{t('dsn-remote-mta')} <bdi>{isolate(row.remoteMta!)}</bdi></p></Show>
      <Show when={row.lastAttempt}><p>{t('dsn-last-attempt')} {isolate(row.lastAttempt!)}</p></Show>
      <Show when={row.retryUntil}><p>{t('dsn-retry-until')} {isolate(row.retryUntil!)}</p></Show>
    </details>}</For>
  </section>}</Show>;
}
