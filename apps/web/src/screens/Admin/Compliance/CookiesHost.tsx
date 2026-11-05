// The Cookies tab inside Compliance: it supplies the settings context the tab was written against (it used to live in the Privacy screen).

import { createResource, type JSX } from 'solid-js';
import { listAdminSettings, type AdminSetting } from '../../../api/admin/settings.ts';
import { useShell } from '../shell/context.ts';
import type { PrivacyTabContext } from '../Privacy/model.ts';
import CookiesTab from '../Cookies/CookiesTab.tsx';

export default function CookiesHost(): JSX.Element {
  const shell = useShell();
  const canRead = (): boolean => shell.can.can('settings.read');
  const [settings, { mutate, refetch }] = createResource(async () => (canRead() ? await listAdminSettings() : null));
  const ctx: PrivacyTabContext = {
    rows: () => (!settings.error && settings.latest?.kind === 'ok' ? settings.latest.data : []),
    settingsError: () => settings.error,
    canReadSettings: canRead,
    stats: () => undefined,
    statsError: () => undefined,
    statsLoading: () => false,
    updatedAt: () => null,
    range: () => '1h',
    setRange: () => {},
    refresh: () => {},
    focusKey: () => { const f = shell.router.route().rest[1]; return f && f.includes('.') ? f : undefined; },
    onDirty: () => {},
    onSaved: (row: AdminSetting) => mutate((v) => (v?.kind === 'ok' ? { ...v, data: v.data.map((c) => (c.key === row.key ? row : c)) } : v)),
    reloadSettings: () => void refetch(),
  };
  return <CookiesTab ctx={ctx} />;
}
