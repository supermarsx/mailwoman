// t95: the user Settings breadcrumb levels: category › section [› New | Edit].
// Labels are the t66 category and section names. A label whose catalog has not
// loaded yet is `null` (the trail draws a placeholder) rather than its message
// id; a label missing from the catalogs falls back to words made from the
// section id, logged at debug level. The editor level is a word, never the
// edited item's id.

import { createSignal } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { humanize, logMissingCrumbLabel } from '../../components/breadcrumbs/humanize.ts';
import { categoryOf } from './settingsCategories.ts';
import type { SettingsRoute } from './navigation.tsx';

/** The catalogs the category and section names live in. */
export const SETTINGS_CRUMB_CATALOGS = [
  'settings', 'linked-accounts', 'mail-transfer', 'receipts', 'packages', 'labels', 'appearance',
  'settings-policy', 'nextcloud', 'reader', 'privacy', 'security',
] as const;

const [ready, setReady] = createSignal(false);
let load: Promise<void> | undefined;
/** Resolves once the label catalogs have loaded (or failed to). */
export function settingsCrumbCatalogsLoaded(): Promise<void> {
  load ??= Promise.allSettled(SETTINGS_CRUMB_CATALOGS.map((c) => loadCatalog(c))).then(() => { setReady(true); });
  return load;
}
function catalogsReady(): boolean {
  void settingsCrumbCatalogsLoaded();
  return ready();
}

/** A label, `null` while its catalog loads, the humanized fallback when missing. */
export function settingsLabel(id: string, fallback: string): string | null {
  const text = t(id);
  if (text !== id) return text;
  if (!catalogsReady()) return null;
  logMissingCrumbLabel('settings', fallback);
  return humanize(fallback) || fallback;
}

export interface SettingsCrumb {
  label: string | null;
  /** Set on a level that opens a page (the section, while an editor is open). */
  route?: SettingsRoute;
}

/** The trail for a Settings route, empty for a section outside the navigation. */
export function settingsCrumbs(route: SettingsRoute): SettingsCrumb[] {
  const category = categoryOf(route.section);
  if (!category) return [];
  const section = category.sections.find((s) => s.id === route.section)!;
  const items: SettingsCrumb[] = [
    { label: settingsLabel(category.label, category.id) },
    route.editor ? { label: settingsLabel(section.label, section.id), route: { section: section.id } } : { label: settingsLabel(section.label, section.id) },
  ];
  if (route.editor) items.push({ label: settingsLabel(route.editor.kind === 'new' ? 'settings-crumb-new' : 'settings-crumb-edit', route.editor.kind) });
  return items;
}
