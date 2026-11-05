// Request protection > Protocol guard: every `security.protocol.*` policy edited
// in place with InlineSettingEditor, grouped by class, with the guard's live
// counters beside each class and an overall mode switch (off / detect / enforce;
// moving to enforce asks first and says what detection has seen so far).

import { For, Show, createResource, createSignal, type JSX } from 'solid-js';
import { listAdminSettings, putGlobal, type AdminSetting } from '../../api/admin/settings.ts';
import { protectionApi, type ProtectionApi, type ProtocolGuardRow } from '../../api/admin/requestProtection.ts';
import { formatNumber, loadCatalog, t } from '../../i18n';
import { resourceValue } from '../../components/resourceValue.ts';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { AdminSubsection } from './AdminSubsection.tsx';
import { useShell } from './shell/context.ts';
import { ConfirmDialog } from './shell/ConfirmDialog.tsx';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { adminDocsHref } from './shell/docsLink.ts';
import { InlineSettingEditor } from './Settings/InlineSettingEditor.tsx';
import { GUARD_GROUPS, GUARD_MODES, MODE_KEYS, classOfKey, overallMode, wouldRefuse, type GuardMode } from './protocolGuard.ts';
import { ModeChip } from './ProtocolGuardStats.tsx';
import * as css from './protocolGuard.css.ts';
import * as admin from './admin.css.ts';

export function ProtocolGuardPanel(props: { load?: typeof listAdminSettings; api?: ProtectionApi }): JSX.Element {
  void loadCatalog('admin-settings');
  const shell = useShell();
  const canRead = (): boolean => shell.can.can('settings.read');
  const canWrite = (): boolean => shell.can.can('settings.write');
  const [settings, { mutate }] = createResource(canRead, async () => await (props.load ?? listAdminSettings)());
  const [stats, { refetch: refetchStats }] = createResource(async (): Promise<ProtocolGuardRow[]> => {
    try {
      return (await (props.api ?? protectionApi).status()).protocolGuard?.classes ?? [];
    } catch {
      return [];
    }
  });
  const current = () => resourceValue(settings);
  const rows = (): AdminSetting[] => {
    const r = current();
    return r?.kind === 'ok' ? r.data.filter((row) => row.key.startsWith('security.protocol.')) : [];
  };
  const rowOf = (key: string): AdminSetting | undefined => rows().find((r) => r.key === key);
  const statOf = (cls: string): ProtocolGuardRow | undefined => stats()?.find((r) => r.class === cls);
  const [toast, setToast] = createSignal('');
  const [error, setError] = createSignal('');
  const [pending, setPending] = createSignal<GuardMode | null>(null);
  const [busy, setBusy] = createSignal(false);
  const onSaved = (row: AdminSetting): void => {
    mutate((value) => (value?.kind === 'ok' ? { ...value, data: value.data.map((c) => (c.key === row.key ? row : c)) } : value));
    void refetchStats();
  };
  const modes = (): (string | undefined)[] => MODE_KEYS.map((k) => rowOf(k)?.effective.value as string | undefined);
  const overall = () => overallMode(modes());
  const writable = (): AdminSetting[] => MODE_KEYS.map(rowOf).filter((r): r is AdminSetting => r !== undefined && !r.globalLocked && r.env === null);

  async function apply(mode: GuardMode): Promise<void> {
    setBusy(true);
    setError('');
    try {
      let changed = 0;
      for (const row of writable()) {
        if (row.effective.value === mode) continue;
        const saved = await putGlobal(row.key, { globalValue: mode }, row.etag);
        onSaved(saved.setting);
        changed += 1;
      }
      setToast(t('admin-protocol-mode-applied', { mode: t(`admin-protocol-mode-${mode}`), count: changed }));
    } catch (cause) {
      setError(describeAdminError(cause));
    } finally {
      setBusy(false);
      setPending(null);
    }
  }
  const choose = (mode: GuardMode): void => {
    if (mode === overall()) return;
    if (mode === 'enforce') setPending(mode);
    else void apply(mode);
  };

  return (
    <section aria-label={t('admin-protocol-panel-title')} data-testid="protocol-guard-panel">
      <p class={admin.note}>{t('admin-protocol-panel-help')} <a href={adminDocsHref('protocol-guard')} target="_blank" rel="noopener noreferrer">{t('admin-ux-learn-more')}</a></p>
      <Feedback kind="success" as="toast" message={toast()} />
      <Show when={!canRead()}><p role="alert">{t('admin-protocol-panel-denied')}</p></Show>
      <Show when={canRead()}>
        <Show when={settings.loading && current() === undefined}><AsyncPending /></Show>
        <Show when={settings.error}><p role="alert">{describeAdminError(settings.error)}</p></Show>
        <Show when={current() !== undefined && current()?.kind !== 'ok'}><p role="alert">{t('admin-protocol-panel-unavailable')}</p></Show>
        <Show when={current()?.kind === 'ok'}>
          <div class={admin.card}>
            <h3 class={admin.heading}>{t('admin-protocol-overall')}</h3>
            <p class={admin.note}>{t('admin-protocol-overall-help')}</p>
            <div class={css.modeSwitch} role="group" aria-label={t('admin-protocol-overall')}>
              <For each={GUARD_MODES}>
                {(mode) => (
                  <button class="btn" type="button" aria-pressed={overall() === mode} disabled={!canWrite() || busy()} onClick={() => choose(mode)} data-testid={`protocol-overall-${mode}`}>
                    {t(`admin-protocol-mode-${mode}`)}
                  </button>
                )}
              </For>
              <span>{t('admin-protocol-overall-now')} {overall() === 'mixed' ? <span>{t('admin-protocol-mode-mixed')}</span> : <ModeChip mode={overall()} />}</span>
            </div>
            <p class={admin.note} data-testid="protocol-would-refuse">{t('admin-protocol-would-hint', { count: formatNumber(wouldRefuse(stats() ?? [])) })}</p>
            <Show when={error()}><p role="alert">{error()}</p></Show>
          </div>
          <For each={GUARD_GROUPS}>
            {(group) => (
              <AdminSubsection title={t(`admin-protocol-group-${group.id}`)} description={t(`admin-protocol-group-help-${group.id}`)}>
                <For each={group.keys.filter((k) => rowOf(k) !== undefined)}>
                  {(key) => {
                    const cls = classOf(key);
                    return (
                      <div class={css.editRow} data-testid={`protocol-row-${key}`}>
                        <InlineSettingEditor settingKey={key} setting={rowOf(key)} onChanged={onSaved} />
                        <Show when={cls !== null && statOf(cls)}>
                          {(s) => (
                            <p class={css.statLine}>
                              {t('admin-protocol-stat-line', { detected: formatNumber(s().detected), refused: formatNumber(s().enforced) })}
                            </p>
                          )}
                        </Show>
                      </div>
                    );
                  }}
                </For>
              </AdminSubsection>
            )}
          </For>
        </Show>
      </Show>
      <ConfirmDialog
        open={pending() === 'enforce'}
        title={t('admin-protocol-enforce-title')}
        body={t('admin-protocol-enforce-body', { count: formatNumber(wouldRefuse(stats() ?? [])) })}
        confirmLabel={t('admin-protocol-enforce-confirm')}
        danger={false}
        testId="protocol-enforce-dialog"
        onCancel={() => setPending(null)}
        onConfirm={() => apply('enforce')}
      />
    </section>
  );
}

const classOf = (key: string): string | null => classOfKey(key);
