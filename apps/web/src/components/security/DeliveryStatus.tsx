import { createResource, For, onMount, Show, type JSX } from 'solid-js';
import { fetchDeliveryReport } from '../../api/dsn.ts';
import { isolate, loadCatalog, t } from '../../i18n/index.ts';
import './deliveryStatus.css';

export function DeliveryStatus(props: {accountId: string; emailId: string; enabled: boolean}): JSX.Element {
  onMount(()=>void loadCatalog('mail-dsn'));
  const [report] = createResource(
    ()=>props.enabled ? {account:props.accountId,email:props.emailId} : null,
    async key=>{
      try { return {...key,report:await fetchDeliveryReport(key.account,key.email)}; }
      catch { return {...key,report:null}; }
    },
  );
  const current = ()=>{
    const result = report.state==='ready' ? report.latest : undefined;
    return props.enabled && result?.account===props.accountId && result.email===props.emailId ? result.report : null;
  };
  return <Show when={current()}>{value=><section class="delivery-status" aria-label={t('dsn-title')}>
    <h3>{t('dsn-title')}</h3><p>{t('dsn-unverified')}</p>
    <p>{t('dsn-reporting-mta')} <bdi>{isolate(value().reportingMta)}</bdi></p>
    <For each={value().recipients}>{row=><details class="delivery-status__recipient" open={row.action==='failed'||row.action==='delayed'}>
      <summary><strong>{t(`dsn-action-${row.action}`)}</strong> <bdi>{isolate(row.finalRecipient)}</bdi> <code>{row.status}</code></summary>
      <Show when={row.originalRecipient}><p>{t('dsn-original-recipient')} <bdi>{isolate(row.originalRecipient!)}</bdi></p></Show>
      <Show when={row.diagnostic}><p class="delivery-status__diagnostic">{isolate(row.diagnostic!)}</p></Show>
      <Show when={row.remoteMta}><p>{t('dsn-remote-mta')} <bdi>{isolate(row.remoteMta!)}</bdi></p></Show>
      <Show when={row.lastAttempt}><p>{t('dsn-last-attempt')} {isolate(row.lastAttempt!)}</p></Show>
      <Show when={row.retryUntil}><p>{t('dsn-retry-until')} {isolate(row.retryUntil!)}</p></Show>
    </details>}</For>
  </section>}</Show>;
}
