// Admin console › Settings (t32-admin §2.4, §3.4 e16; worker f01).
//
// The settings model C: every registry key with what decides its value now,
// searchable and filtered by category (list), and one key per page with its
// value for the whole server, its locks, and its domain and per-user values
// (detail, `#/settings/<key>`). Template: ListDetailPage (UX3).
//
// On a server without the settings routes the page says "Not available"
// instead of inventing values (UX11).

import { createMemo, createResource, createSignal, Show, useContext, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import { deleteGlobal, listAdminSettings, type AdminSetting } from '../../../api/admin/settings.ts';
import { RevertToDefault } from '../shell/RevertToDefault.tsx';
import type { AdminReadResult } from '../shell/adminApi.ts';
import { ShellContext } from '../shell/context.ts';
import { parseAdminPath, type AdminRoute } from '../shell/router.ts';
import { SourceBadge } from '../shell/SourceBadge.tsx';
import { AdminPage, PageState } from '../shell/templates/PageFrame.tsx';
import { ListDetailPage } from '../shell/templates/ListDetailPage.tsx';
import { CATEGORIES } from './keys.ts';
import { categoryLabel, settingLabel } from './labels.ts';
import { SettingDetail, sourceOf } from './SettingDetail.tsx';
import { displayValue } from './value.ts';
import * as css from './settings.css.ts';

export interface AdminSettingsProps {
  /** Injected for tests; default `GET /admin/settings`. */
  load?: () => Promise<AdminReadResult<AdminSetting[]>>;
  /** Injected for tests; default the shell router's route. */
  route?: AdminRoute;
  /** Injected for tests; passed to the detail. */
  loadOne?: (key: string) => Promise<{ setting: AdminSetting; etag: string | null }>;
}

function overridesText(s: AdminSetting): string {
  const domains = s.overrides.domain ?? 0;
  const users = (s.overrides.user ?? 0) + (s.overrides.admin_user ?? 0);
  return domains + users === 0 ? t('admin-settings-overrides-none') : t('admin-settings-overrides', { domains, users });
}

export function AdminSettings(props: AdminSettingsProps): JSX.Element {
  void loadCatalog('admin-settings');
  const shell = useContext(ShellContext);
  const route = (): AdminRoute =>
    props.route ?? shell?.router.route() ?? parseAdminPath(typeof location === 'undefined' ? '' : location.hash);
  const [list, { refetch }] = createResource(props.load ?? listAdminSettings);
  const [query, setQuery] = createSignal('');
  const [category, setCategory] = createSignal<string | null>(null);

  const all = (): AdminSetting[] => {
    const r = list();
    return r?.kind === 'ok' ? r.data : [];
  };
  const present = createMemo(() => {
    const cats = new Set(all().map((s) => s.category));
    return CATEGORIES.filter((c) => cats.has(c)).concat([...cats].filter((c) => !CATEGORIES.includes(c)));
  });
  const rows = createMemo(() => {
    const q = query().trim().toLowerCase();
    const order = (c: string): number => {
      const i = CATEGORIES.indexOf(c);
      return i === -1 ? CATEGORIES.length : i;
    };
    return all()
      .filter((s) => category() === null || s.category === category())
      .filter((s) => q === '' || s.key.toLowerCase().includes(q) || settingLabel(s.key).toLowerCase().includes(q))
      .sort((a, b) => order(a.category) - order(b.category));
  });

  return (
    <Show
      when={list()?.kind !== 'unavailable'}
      fallback={
        <AdminPage title={t('admin-settings-title')} description={t('admin-settings-description')}>
          <PageState state={{ kind: 'unavailable', reason: t('admin-settings-unavailable') }} />
        </AdminPage>
      }
    >
      <ListDetailPage<AdminSetting>
        title={t('admin-settings-title')}
        description={t('admin-settings-description')}
        section="settings"
        route={route()}
        testId="admin-settings"
        compactActions
        table={{
          caption: t('admin-settings-list-caption'),
          columns: [
            {
              id: 'setting',
              header: t('admin-settings-col-setting'),
              cell: (s) => (
                <span class={css.nameCell}>
                  <span>{settingLabel(s.key)}</span>
                  <span class={css.keyName}>{s.key}</span>
                </span>
              ),
            },
            { id: 'value', header: t('admin-settings-col-value'), cell: (s) => <span class={css.valueCell} title={displayValue(s.kind, s.effective.value)}>{displayValue(s.kind, s.effective.value)}</span> },
            { id: 'source', header: t('admin-settings-col-source'), cell: (s) => <SourceBadge source={sourceOf(s)} /> },
            { id: 'overrides', header: t('admin-settings-col-overrides'), cell: overridesText },
          ],
          rows: rows(),
          rowKey: (s) => s.key,
          state: list.error ? 'error' : list.loading ? 'loading' : 'ready',
          errorText: list.error ? String(list.error) : '',
          onRetry: () => void refetch(),
          emptyText: t('admin-settings-empty'),
          search: { label: t('admin-settings-search'), value: query(), onSearch: setQuery },
          filters: present().map((c) => ({ id: c, label: categoryLabel(c), active: category() === c })),
          compactFilters: true,
          onToggleFilter: (id) => setCategory(category() === id ? null : id),
          rowActions: (s) => <Show when={!s.dedicatedRoute && (s.globalValue !== null && s.globalValue !== undefined || s.globalLocked)}>
            <RevertToDefault compact label={settingLabel(s.key)} defaultDisplay={displayValue(s.kind, s.default)} isDefault={false}
              disabledReason={s.envState?.set && s.envState.precedence === 'pin' ? t('admin-settings-pinned', { name: s.envState.var }) : shell?.can.can('settings.write') === false ? shell.can.reason('settings.write') : null}
              onRevert={async () => { await deleteGlobal(s.key, s.etag); await refetch(); }} />
          </Show>,
        }}
        openLabel={() => t('admin-settings-open')}
        detailTitle={(key) => settingLabel(key)}
        renderDetail={(key) => (
          <Show
            when={list.loading || all().some((s) => s.key === key) || key.startsWith('serverText.')}
            fallback={<PageState state={{ kind: 'empty', text: t('admin-settings-not-found', { key }) }} />}
          >
            <SettingDetail
              settingKey={key}
              onChanged={() => void refetch()}
              {...(props.loadOne !== undefined ? { load: props.loadOne } : {})}
            />
          </Show>
        )}
      />
    </Show>
  );
}

export default AdminSettings;
