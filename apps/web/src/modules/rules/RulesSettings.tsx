// Visual rules and the authoritative Sieve script store share one account.
import { createEffect, createMemo, createSignal, For, on, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import { createClient } from '../../api/client.ts';
import { isUnsupportedMethod } from '../../api/unsupported.ts';
import { useApp } from '../../state/context.ts';
import type { MailRule } from '../../api/crypto-types.ts';
import { createRulesSlice, emptyRuleDraft, type RuleDraft, type RulesSlice } from '../../state/slices/rules.ts';
import { RuleBuilder } from './RuleBuilder.tsx';
import { ScriptsPanel } from './ScriptsPanel.tsx';
import { VacationPanel } from './VacationPanel.tsx';
import { ForwardingPanel } from './ForwardingPanel.tsx';
import { vacationApi } from './vacation-api.ts';
import { sieveApi, type SieveApi, type SieveCapability } from './sieve-api.ts';
import * as css from './styles.css.ts';

export interface RulesSettingsProps { accountId: string; slice?: RulesSlice; scriptApi?: SieveApi }
export function RulesSettings(props: RulesSettingsProps): JSX.Element {
  onMount(() => void loadCatalog('rules'));
  const client = createClient();
  const slice = props.slice ?? createRulesSlice({ client, showToast: useApp().showToast }, () => props.accountId);
  const scripts = createMemo(() => props.scriptApi ?? sieveApi(client, props.accountId));
  const [capability, setCapability] = createSignal<SieveCapability | null>(null);
  createEffect(on(scripts, (api) => { setCapability(null); void api.connect().then((cap) => { if (api === scripts()) setCapability(cap); }).catch(() => undefined); }));
  const vacation = createMemo(() => vacationApi(client, props.accountId));
  const [pane, setPane] = createSignal<'builder' | 'raw' | 'vacation' | 'forward'>('builder');
  const [editing, setEditing] = createSignal<RuleDraft | null>(null);
  const [loadFailure, setLoadFailure] = createSignal<{ error: unknown } | null>(null);
  const [editFailure, setEditFailure] = createSignal('');
  const load = (): void => {
    setLoadFailure(null);
    slice.loadRules().catch((error: unknown) => setLoadFailure({ error }));
  };
  createEffect(on(() => props.accountId, () => { setEditing(null); setEditFailure(''); load(); }));
  const unsupported = (): boolean => loadFailure() !== null && isUnsupportedMethod(loadFailure()?.error);
  async function change(work: () => Promise<void>): Promise<void> {
    setEditFailure('');
    try { await work(); } catch (error) { setEditFailure(error instanceof Error ? error.message : String(error)); }
  }
  function edit(rule: MailRule): void { setEditing({ ...rule }); }
  return <section class={css.panel} aria-labelledby="rules-heading">
    <h2 id="rules-heading" class={css.heading}>{t('rules-title')}</h2>
    <p class={css.prose}>{t('rules-intro')}</p>
    <div class={css.tabs} role="tablist" aria-label={t('rules-panes-label')}>
      <For each={['builder', 'raw', 'vacation', 'forward'] as const}>{(id) => <button type="button" role="tab" id={`rules-tab-${id}`} class={css.tab}
        aria-selected={pane() === id} aria-controls={`rules-panel-${id}`} onClick={() => { setPane(id); if (id === 'builder') load(); }}>
        {t(id === 'builder' ? 'rules-tab-builder' : id === 'raw' ? 'rules-tab-raw' : id === 'vacation' ? 'rules-vacation-title' : 'rules-forward-title')}
      </button>}</For>
    </div>
    <Show when={pane() === 'builder'}>
      <div role="tabpanel" id="rules-panel-builder" aria-labelledby="rules-tab-builder">
        <Show when={loadFailure() === null} fallback={<>
          <p class={css.prose} role={unsupported() ? undefined : 'alert'}>{t(unsupported() ? 'rules-unavailable' : 'rules-load-failed')}</p>
          <Show when={!unsupported()}><button type="button" class={css.btn} data-testid="rules-retry" onClick={load}>{t('common-retry')}</button></Show>
        </>}>
          <Show when={editFailure()}><p role="alert">{editFailure()}</p></Show>
          <Show when={slice.rules().length > 0} fallback={<p class={css.prose}>{t('rules-empty')}</p>}>
            <ul class={css.list} aria-label={t('rules-list-label')}><For each={slice.rules()}>{(rule) => <li class={css.ruleRow}>
              <span class={css.ruleName}>{isolate(rule.name)}</span>
              <span class={rule.runsAt === 'server-sieve' ? `${css.badge} ${css.badgeServer}` : css.badge}>{t(rule.runsAt === 'server-sieve' ? 'rules-runs-server' : 'rules-runs-engine')}</span>
              <label class={css.row}><input type="checkbox" checked={rule.enabled} onChange={(event) => {
                const enabled = event.currentTarget.checked; void change(() => slice.toggleRule(rule.id, enabled));
              }} />{t('rules-enabled')}</label>
              <button type="button" class={css.btn} onClick={() => edit(rule)}>{t('common-edit')}</button>
              <button type="button" class={css.btnDanger} aria-label={t('rules-delete-named', { name: isolate(rule.name) })}
                onClick={() => void change(() => slice.deleteRule(rule.id))}>{t('common-delete')}</button>
            </li>}</For></ul>
          </Show>
          <button type="button" class={css.btnPrimary} onClick={() => { const draft=emptyRuleDraft(); if (capability() && !capability()!.sieveExtensions.includes('fileinto')) draft.actions=[{type:'stop',value:null}]; setEditing(draft); }}>{t('rules-new')}</button>
          <Show when={editing()} fallback={<p class={css.prose}>{t('rules-builder-hint')}</p>}>{(draft) =>
            <RuleBuilder initial={draft()} runsAt={capability()?.['urn:mailwoman:policy']?.runsAt ?? draft().runsAt}
              {...(capability() ? { extensions: capability()!.sieveExtensions } : {})}
              onSave={(next) => void change(async () => { await slice.saveRule(next); setEditing(null); })} onCancel={() => setEditing(null)} />
          }</Show>
        </Show>
      </div>
    </Show>
    <Show when={pane() === 'raw'}><div role="tabpanel" id="rules-panel-raw" aria-labelledby="rules-tab-raw"><ScriptsPanel api={scripts()} /></div></Show>
    <Show when={pane() === 'vacation'}><div role="tabpanel" id="rules-panel-vacation" aria-labelledby="rules-tab-vacation"><VacationPanel api={vacation()} /></div></Show>
    <Show when={pane() === 'forward'}><div role="tabpanel" id="rules-panel-forward" aria-labelledby="rules-tab-forward"><ForwardingPanel slice={slice} api={scripts()} /></div></Show>
  </section>;
}
