// One engine plugin, in the Plugins list's detail pane.
//
// Panels, each with its own action:
//   • status — the server's word on whether it runs and why not, and the
//     capabilities it runs with;
//   • grants — the declared capabilities as checkboxes; Save replaces the grant
//     (the edit starts from, and follows, what the server stored);
//   • network and limits — the hosts it may contact, memory, time and fuel;
//   • spam classifier (role `spam-classifier` only) — the daemon address and a
//     test call, whose verdict and raw answer are shown as the component gave them;
//   • unsigned (third-party without a signature only) — allowing it to run
//     unsigned asks first; withdrawing does not;
//   • remove — Uninstall asks for the plugin id to be typed, since it deletes the
//     plugin's stored data for every account.

import { For, Show, createEffect, createSignal, createUniqueId, type JSX } from 'solid-js';
import { PluginsApiError, type ClassifierTest, type PluginCapability, type PluginInfo, type PluginsSlice } from '../../../state/slices/plugins.ts';
import { isolate, t } from '../../../i18n';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback } from '../shell/Feedback.tsx';
import { HelpNote } from '../shell/HelpNote.tsx';
import { adminDocsHref } from '../shell/docsLink.ts';
import * as shell from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from '../shell/extras.css.ts';
import { StatusCell, TrustCell, reasonText, refusalText } from './shared.tsx';

const AREA = 'plugins';

export interface PluginDetailProps {
  plugin: PluginInfo;
  slice: PluginsSlice;
  onNotice: (text: string) => void;
  onUninstalled: () => void;
}

function LearnMore(props: { id: string; label: string }): JSX.Element {
  return (
    <a class={ux.learnMore} href={adminDocsHref(AREA, props.id)} target="_blank" rel="noopener noreferrer">
      {t('admin-ux-learn-more')}
      <span class={shell.srOnly}> {t('admin-ux-learn-more-field', { label: props.label })}</span>
    </a>
  );
}

export function PluginDetail(props: PluginDetailProps): JSX.Element {
  const id = createUniqueId();
  const p = (): PluginInfo => props.plugin;
  const slice = props.slice;

  // The grant being edited; starts from, and follows, what the server has stored.
  const [draft, setDraft] = createSignal<PluginCapability[]>([]);
  createEffect(() => setDraft([...p().granted]));
  const grantChanged = (): boolean => [...draft()].sort().join() !== [...p().granted].sort().join();

  const [endpoint, setEndpoint] = createSignal('');
  createEffect(() => setEndpoint(p().endpoint ?? ''));
  const [tested, setTested] = createSignal<ClassifierTest | null>(null);
  const [testError, setTestError] = createSignal<string | null>(null);
  const [confirmUnsigned, setConfirmUnsigned] = createSignal(false);
  const [confirmUninstall, setConfirmUninstall] = createSignal(false);

  async function saveGrants(): Promise<void> {
    if (await slice.grant(p().id, { accountId: null, capabilities: draft() })) props.onNotice(t('plugins-admin-grants-saved', { name: isolate(p().name) }));
  }

  async function saveEndpoint(): Promise<void> {
    const value = endpoint().trim() === '' ? null : endpoint().trim();
    if (await slice.setEndpoint(p().id, value)) props.onNotice(t('plugins-admin-endpoint-saved', { name: isolate(p().name) }));
  }

  async function runTest(): Promise<void> {
    setTested(null);
    setTestError(null);
    try {
      setTested(await slice.api.testClassifier(p().id));
    } catch (e) {
      if (!(e instanceof PluginsApiError)) throw e;
      setTestError(refusalText(e));
    }
  }

  async function setUnsigned(allow: boolean): Promise<void> {
    if (await slice.setAllowUnsigned(p().id, allow)) {
      props.onNotice(allow ? t('plugins-admin-unsigned-allowed', { name: isolate(p().name) }) : t('plugins-admin-unsigned-withdrawn', { name: isolate(p().name) }));
    }
  }

  return (
    <div class={css.listEditor} data-testid="plugin-card" data-plugin-id={p().id}>
      <section class={css.panel} aria-labelledby={`${id}-status`}>
        <h4 id={`${id}-status`} class={css.panelTitle}>
          {t('plugins-admin-panel-status')}
        </h4>
        <span class={css.badgeRow}>
          <TrustCell plugin={p()} />
          <StatusCell plugin={p()} />
        </span>
        <Show when={p().restartRequired}>
          <p class={css.panelText} role="status" data-testid="status-line">
            {p().loaded ? t('plugins-admin-restart-to-apply') : t('plugins-admin-restart-to-load')}
          </p>
        </Show>
        <Show when={!p().restartRequired ? p().notLoadedReason : null}>
          {(reason) => (
            <p class={css.panelText} role="status" data-testid="status-line">
              {reasonText(reason())}
            </p>
          )}
        </Show>
        <Show when={p().loaded}>
          <p class={`${css.panelText} ${css.mono}`} data-testid="running-with">
            {t('plugins-admin-running-with', { capabilities: p().loadedCapabilities.join(', ') })}
          </p>
        </Show>
      </section>

      <section class={css.panel} aria-labelledby={`${id}-grants`}>
        <h4 id={`${id}-grants`} class={css.panelTitle}>
          {t('plugins-admin-grants-heading')}
        </h4>
        <HelpNote id={`${id}-grants-help`}>
          <>
            {t('plugins-admin-grants-hint')} <LearnMore id="grants" label={t('plugins-admin-grants-heading')} />
          </>
        </HelpNote>
        <fieldset class={css.checks} aria-describedby={`${id}-grants-help`}>
          <legend class={shell.srOnly}>{t('plugins-admin-grants-heading')}</legend>
          <For each={p().capabilities}>
            {(cap) => (
              <div class={css.checkRow} data-testid="grant-check">
                <input
                  id={`${id}-cap-${cap}`}
                  type="checkbox"
                  checked={draft().includes(cap)}
                  onChange={(e) => setDraft(e.currentTarget.checked ? [...draft(), cap] : draft().filter((c) => c !== cap))}
                />
                <label for={`${id}-cap-${cap}`} class={css.mono}>
                  {cap}
                </label>
              </div>
            )}
          </For>
        </fieldset>
        <div class={css.actions}>
          <button type="button" class="btn btn--primary" disabled={!grantChanged()} onClick={() => void saveGrants()}>
            {t('plugins-admin-grants-save')}
          </button>
        </div>
      </section>

      <section class={css.panel} aria-labelledby={`${id}-net`}>
        <h4 id={`${id}-net`} class={css.panelTitle}>
          {t('plugins-admin-panel-network')}
        </h4>
        <dl class={css.summary}>
          <dt class={css.summaryTerm}>{t('plugins-admin-hosts-label')}</dt>
          <dd class={`${css.summaryValue} ${css.mono}`}>{p().netAllowlist.length > 0 ? p().netAllowlist.join(', ') : t('plugins-admin-hosts-empty')}</dd>
          <dt class={css.summaryTerm}>{t('plugins-admin-limit-memory')}</dt>
          <dd class={css.summaryValue}>{t('plugins-admin-limit-memory-value', { mb: p().limits.memoryMb })}</dd>
          <dt class={css.summaryTerm}>{t('plugins-admin-limit-deadline')}</dt>
          <dd class={css.summaryValue}>{t('plugins-admin-limit-deadline-value', { ms: p().limits.deadlineMs })}</dd>
          <Show when={p().limits.fuel !== null}>
            <dt class={css.summaryTerm}>{t('plugins-admin-limit-fuel')}</dt>
            <dd class={css.summaryValue}>{p().limits.fuel ?? 0}</dd>
          </Show>
        </dl>
        <p class={css.panelText}>
          {t('plugins-admin-network-note')} <LearnMore id="network" label={t('plugins-admin-panel-network')} />
        </p>
      </section>

      <Show when={p().role === 'spam-classifier'}>
        <section class={css.panel} aria-labelledby={`${id}-cls`}>
          <h4 id={`${id}-cls`} class={css.panelTitle}>
            {t('plugins-admin-panel-classifier')}
          </h4>
          <div class={shell.field}>
            <label class={shell.fieldLabel} for={`${id}-endpoint`}>
              {t('plugins-admin-endpoint')}
            </label>
            <input
              id={`${id}-endpoint`}
              class={`${ux.control} ${css.mono}`}
              type="text"
              spellcheck={false}
              value={endpoint()}
              aria-describedby={`${id}-endpoint-help`}
              onInput={(e) => setEndpoint(e.currentTarget.value)}
            />
            <HelpNote id={`${id}-endpoint-help`}>
              <>
                {t('plugins-admin-endpoint-hint')} <LearnMore id="endpoint" label={t('plugins-admin-endpoint')} />
              </>
            </HelpNote>
          </div>
          <div class={css.actions}>
            <button type="button" class="btn" disabled={endpoint().trim() === (p().endpoint ?? '')} onClick={() => void saveEndpoint()}>
              {t('plugins-admin-endpoint-save')}
            </button>
            <button type="button" class="btn btn--ghost" disabled={!p().loaded} aria-describedby={!p().loaded ? `${id}-test-why` : undefined} onClick={() => void runTest()}>
              {t('plugins-admin-test')}
            </button>
            <Show when={!p().loaded}>
              <span id={`${id}-test-why`} class={shell.help}>
                {t('plugins-admin-error-not-loaded')}
              </span>
            </Show>
          </div>
          <Show when={tested()}>
            {(result) => (
              <div data-testid="test-result" role="status">
                <p class={css.panelText}>{t('plugins-admin-test-verdict', { verdict: result().verdict })}</p>
                <pre class={css.pre}>{JSON.stringify(result().detail, null, 2)}</pre>
              </div>
            )}
          </Show>
          <Feedback kind="error" message={testError()} />
        </section>
      </Show>

      <Show when={!p().firstParty && !p().signed}>
        <section class={css.panel} aria-labelledby={`${id}-unsigned`}>
          <h4 id={`${id}-unsigned`} class={css.panelTitle}>
            {t('plugins-admin-panel-unsigned')}
          </h4>
          <div class={css.checkRow} data-testid="allow-unsigned">
            <input
              id={`${id}-allow`}
              type="checkbox"
              checked={p().allowUnsigned}
              aria-describedby={`${id}-allow-help`}
              onChange={(e) => {
                const on = e.currentTarget.checked;
                // The box shows the stored state until the server answers.
                e.currentTarget.checked = p().allowUnsigned;
                if (on) setConfirmUnsigned(true);
                else void setUnsigned(false);
              }}
            />
            <label for={`${id}-allow`}>{t('admin-plugins-allow-unsigned')}</label>
          </div>
          <HelpNote id={`${id}-allow-help`}>
            <>
              {t('plugins-admin-unsigned-help')} <LearnMore id="unsigned" label={t('admin-plugins-allow-unsigned')} />
            </>
          </HelpNote>
        </section>
      </Show>

      <section class={css.panel} aria-labelledby={`${id}-remove`}>
        <h4 id={`${id}-remove`} class={css.panelTitle}>
          {t('plugins-admin-panel-remove')}
        </h4>
        <p class={css.panelText}>{t('plugins-admin-uninstall-note')}</p>
        <div class={css.actions}>
          <button type="button" class="btn btn--danger" onClick={() => setConfirmUninstall(true)}>
            {t('plugins-admin-uninstall')}
            <span class={shell.srOnly}> {p().name}</span>
          </button>
        </div>
      </section>

      <ConfirmDialog
        open={confirmUnsigned()}
        title={t('plugins-admin-unsigned-title', { name: isolate(p().name) })}
        body={t('plugins-admin-unsigned-body')}
        confirmLabel={t('plugins-admin-unsigned-confirm')}
        onConfirm={async () => {
          await setUnsigned(true);
          setConfirmUnsigned(false);
        }}
        onCancel={() => setConfirmUnsigned(false)}
      />
      <ConfirmDialog
        open={confirmUninstall()}
        title={t('plugins-admin-uninstall-title', { name: isolate(p().name) })}
        body={t('plugins-admin-uninstall-detail')}
        confirmLabel={t('plugins-admin-uninstall')}
        typedConfirmation={p().id}
        onConfirm={async () => {
          await slice.uninstall(p().id);
          setConfirmUninstall(false);
          props.onUninstalled();
        }}
        onCancel={() => setConfirmUninstall(false)}
      />
    </div>
  );
}
