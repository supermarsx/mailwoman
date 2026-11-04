// A titled group of settings, each edited in place (InlineSettingEditor).

import { For, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import { InlineSettingEditor } from '../Settings/InlineSettingEditor.tsx';
import type { PrivacyTabContext } from './model.ts';

/** The rows for `keys`, in the order given, that the server reports. */
export function rowsFor(all: readonly AdminSetting[], keys: readonly string[]): AdminSetting[] {
  return keys.flatMap((key) => all.filter((row) => row.key === key));
}

export function InlineGroup(props: { ctx: PrivacyTabContext; title: string; description?: string; rows: AdminSetting[]; format?: (s: AdminSetting) => string }): JSX.Element {
  return (
    <Show when={props.rows.length > 0}>
      <AdminSubsection title={props.title} {...(props.description ? { description: props.description } : {})}>
        <For each={props.rows}>{(row) => (
          <InlineSettingEditor
            settingKey={row.key}
            setting={row}
            format={props.format}
            focus={props.ctx.focusKey() === row.key}
            onDirty={props.ctx.onDirty}
            onChanged={props.ctx.onSaved}
          />
        )}</For>
      </AdminSubsection>
    </Show>
  );
}

/** What a tab of editors shows instead of rows: no permission, an error, or nothing yet. */
export function SettingsGate(props: { ctx: PrivacyTabContext; children: JSX.Element }): JSX.Element {
  return (
    <Show when={props.ctx.canReadSettings()} fallback={<p role="status">{t('privacy-settings-denied')}</p>}>
      <Show when={props.ctx.settingsError()}><p role="alert">{describeAdminError(props.ctx.settingsError())}</p></Show>
      {props.children}
    </Show>
  );
}
