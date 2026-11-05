// The settings rows of a few keys, loaded once, for InlineSettingEditor groups.

import { createResource } from 'solid-js';
import { listAdminSettings, type AdminSetting } from '../../../api/admin/settings.ts';

export function useSettingRows(keys: readonly string[]): { rows: () => AdminSetting[]; reload: () => void } {
  const [all, { refetch }] = createResource(async () => {
    const r = await listAdminSettings();
    return r.kind === 'ok' ? r.data : [];
  });
  return {
    rows: () => (all.error ? [] : (all() ?? []).filter((row) => keys.includes(row.key))),
    reload: () => void refetch(),
  };
}
