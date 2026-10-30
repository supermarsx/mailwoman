import { createEffect, createSignal, on, Show, type JSX } from 'solid-js';
import { rawT, t } from '../../i18n';
import type { RulesSlice } from '../../state/slices/rules.ts';
import type { SieveApi, SieveCapability } from './sieve-api.ts';
import * as css from './styles.css.ts';

export function ForwardingPanel(props: { slice: RulesSlice; api: SieveApi }): JSX.Element {
  const [cap, setCap] = createSignal<SieveCapability | null>(null);
  const [id, setId] = createSignal<string>();
  const [address, setAddress] = createSignal('');
  const [enabled, setEnabled] = createSignal(false);
  const [keep, setKeep] = createSignal(true);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  createEffect(on(() => props.api, (api) => {
    setCap(null); setError(''); setBusy(false);
    void api.connect().then((value) => { if (api === props.api) { setCap(value); if (!id() && !value?.sieveExtensions.includes('copy')) setKeep(false); } })
      .catch((failure: unknown) => { if (api === props.api) setError(failure instanceof Error ? failure.message : String(failure)); });
  }));
  createEffect(() => {
    const rule = props.slice.rules().find((r) => r.conditions.length === 0 && r.actions.length === 1 && ['forward', 'forwardCopy'].includes(r.actions[0]?.type ?? ''));
    setId(rule?.id); setAddress(rule?.actions[0]?.value ?? ''); setEnabled(rule?.enabled ?? false); setKeep(rule?.actions[0]?.type !== 'forward');
  });
  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault(); const api=props.api; if (busy()) return; setBusy(true); setError('');
    const ruleId=id();
    try { await props.slice.saveRule({ ...(ruleId ? { id:ruleId } : {}), name:rawT('rules-forward-title'), matchAll:true, conditions:[],
      actions:[{ type:keep() ? 'forwardCopy' : 'forward', value:address() }], enabled:enabled(), runsAt:cap()?.['urn:mailwoman:policy']?.runsAt ?? 'server-sieve' }); }
    catch (failure) { if (api===props.api) setError(failure instanceof Error ? failure.message : String(failure)); }
    finally { if (api===props.api) setBusy(false); }
  }
  const externalRefused=(): boolean => {
    const policy=cap()?.['urn:mailwoman:policy']; if (!policy || policy.forwardExternal || !enabled()) return false;
    const domain=address().split('@').at(-1)?.toLowerCase(); return !!domain && !policy.ownDomains.some((own)=>own.toLowerCase()===domain);
  };
  return <form class={css.panel} onSubmit={(event)=>void save(event)} aria-busy={busy()}>
    <h3 class={css.heading}>{t('rules-forward-title')}</h3>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <label class={css.row}><input type="checkbox" checked={enabled()} onChange={(event)=>setEnabled(event.currentTarget.checked)} disabled={busy()}/>{t('rules-enabled')}</label>
    <label class={css.label}>{t('rules-forward-address')}<input class={css.input} type="email" required value={address()} onInput={(event)=>setAddress(event.currentTarget.value)} disabled={busy()}/></label>
    <Show when={cap()?.sieveExtensions.includes('copy')}><label class={css.row}><input type="checkbox" checked={keep()} onChange={(event)=>setKeep(event.currentTarget.checked)} disabled={busy()}/>{t('rules-forward-copy')}</label></Show>
    <Show when={externalRefused()}><p role="alert">{t('rules-forward-refused')}</p></Show>
    <button type="submit" class={css.btnPrimary} disabled={busy() || cap()===null || externalRefused() || (enabled() && keep() && !cap()?.sieveExtensions.includes('copy'))}>{t('common-save')}</button>
  </form>;
}
