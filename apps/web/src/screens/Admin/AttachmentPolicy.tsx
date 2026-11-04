// Admin console > Attachment policy. Tabs (Lists, Limits, Download and preview,
// Warnings, Test a file); every `attachments.*` key is edited in place with
// InlineSettingEditor: chip lists for extensions and MIME types, a size with
// units, switches for download and preview. "Test a file" asks the server's own
// evaluator. `#/settings/attachments.<key>` and `#/attachments/attachments.<key>`
// keep opening the single-key editor / focusing the row.

import { For, Match, Show, Switch, createResource, createSignal, type JSX } from 'solid-js';
import { listAdminSettings, type AdminSetting } from '../../api/admin/settings.ts';
import { loadCatalog, t } from '../../i18n';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { resourceValue } from '../../components/resourceValue.ts';
import { AdminSubsection } from './AdminSubsection.tsx';
import { useShell } from './shell/context.ts';
import { describeAdminError } from './shell/Feedback.tsx';
import { sectionPath } from './shell/router.ts';
import { SectionTabs } from './shell/SectionTabs.tsx';
import { AdminSubtabPanel } from './shell/AdminSubtabPanel.tsx';
import { ATTACHMENT_TABS, attachmentRouteTab, canOpenAttachmentTab } from './shell/attachmentNavigation.ts';
import { InlineSettingEditor, type InlineEditorContext } from './Settings/InlineSettingEditor.tsx';
import { AttachmentListEditor } from './Attachments/AttachmentListEditor.tsx';
import { AttachmentSimulator, type simulateAttachment } from './Attachments/AttachmentSimulator.tsx';
import { ByteSizeEditor } from './Attachments/ByteSizeEditor.tsx';
import { listOf, type ListName } from './Attachments/rules.ts';
import { formatBytes } from '../../i18n';
import { displayValue } from './Settings/value.ts';
import * as css from './admin.css.ts';

export function AdminAttachmentPolicy(props: { load?: typeof listAdminSettings; simulate?: typeof simulateAttachment }): JSX.Element {
  void loadCatalog('admin-settings');
  const shell = useShell();
  const [settings, { refetch, mutate }] = createResource(() => shell.can.can('settings.read'), async () => await (props.load ?? listAdminSettings)());
  // The rows stay while a reload runs; the loading line shows only before the first answer.
  const current = () => resourceValue(settings);
  const rows = (): AdminSetting[] => {
    const result = current();
    return result?.kind === 'ok' ? result.data.filter((row) => row.key.startsWith('attachments.')) : [];
  };
  const [dirty, setDirty] = createSignal(0);
  const dirtyKeys = new Set<string>();
  const onDirty = (key: string, isDirty: boolean): void => {
    if (isDirty) dirtyKeys.add(key); else dirtyKeys.delete(key);
    setDirty(dirtyKeys.size);
  };
  const onSaved = (row: AdminSetting): void => {
    mutate((value) => (value?.kind === 'ok' ? { ...value, data: value.data.map((current) => (current.key === row.key ? row : current)) } : value));
  };

  const tabs = () => ATTACHMENT_TABS.filter((tab) => canOpenAttachmentTab(shell.can, tab));
  const route = () => attachmentRouteTab(shell.router.route());
  const selected = (): string => route().tab ?? tabs()[0]?.id ?? '';
  const allowed = (): boolean => tabs().some((tab) => tab.id === selected());
  const open = (path: string): void => void shell.router.navigate(path);

  const othersFor = (mime: boolean) => (): Record<ListName, readonly string[]> => {
    const valueOf = (list: ListName): readonly string[] => {
      const row = rows().find((r) => r.key === `attachments.${list}${mime ? 'MimeTypes' : 'Extensions'}`);
      const v = row?.globalValue ?? row?.effective.value;
      return Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : [];
    };
    return { allowed: valueOf('allowed'), blocked: valueOf('blocked'), warn: valueOf('warn') };
  };
  const editorFor = (key: string): ((ctx: InlineEditorContext) => JSX.Element) | undefined => {
    const list = listOf(key);
    if (list) return (ctx) => <AttachmentListEditor ctx={ctx} list={list.list} mime={list.mime} others={othersFor(list.mime)} />;
    if (key === 'attachments.maxBytes') return (ctx) => <ByteSizeEditor ctx={ctx} />;
    return undefined;
  };
  const format = (s: AdminSetting): string => (s.key === 'attachments.maxBytes' ? (Number(s.effective.value) > 0 ? formatBytes(Number(s.effective.value)) : t('admin-attachments-unlimited')) : displayValue(s.kind, s.effective.value));

  return (
    <section class={css.section}>
      <h2>{t('admin-attachments-title')}</h2>
      <p>{t('admin-attachments-description')}</p>
      <Show when={settings.loading && current() === undefined}><AsyncPending /></Show>
      <Show when={settings.error}><p role="alert">{describeAdminError(settings.error)}</p></Show>
      <Show when={current() !== undefined && current()?.kind !== 'ok'}><p role="alert">{t('admin-attachments-unavailable')}</p></Show>
      <Show when={current()?.kind === 'ok'}>
        <SectionTabs label={t('admin-attachments-title')} items={tabs().map((tab) => ({ id: tab.id, label: t(tab.label), path: sectionPath('attachments', [tab.id]) }))} selected={selected()} navigate={open} />
        <AdminSubtabPanel>
          <Show when={allowed()} fallback={<p role="alert">{t('admin-attachments-tab-denied')}</p>}>
            <Switch>
              <Match when={selected() === 'test'}><AttachmentSimulator {...(props.simulate ? { simulate: props.simulate } : {})} /></Match>
              <Match when={true}>
                <For each={ATTACHMENT_TABS.filter((tab) => tab.id === selected())}>{(tab) => (
                  <AdminSubsection
                    title={t(`admin-attachments-group-${tab.id}`)}
                    actions={<button class="btn" disabled={settings.loading || dirty() > 0} onClick={() => void refetch()}>{t('admin-attachments-refresh')}</button>}
                  >
                    <For each={tab.keys.flatMap((key) => rows().filter((row) => row.key === key))}>{(row) => (
                      <InlineSettingEditor settingKey={row.key} setting={row} editor={editorFor(row.key)} format={format} focus={route().focusKey === row.key} onDirty={onDirty} onChanged={onSaved} />
                    )}</For>
                  </AdminSubsection>
                )}</For>
              </Match>
            </Switch>
          </Show>
        </AdminSubtabPanel>
      </Show>
    </section>
  );
}
