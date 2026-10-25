// Plain names and help sentences of the settings registry keys (worker f01). A
// key the console has no text for yet (a later lane's key) is shown under its
// own name rather than as a message id.

import { t } from '../../../i18n';
import { keySlug } from './keys.ts';

function lookup(prefix: string, key: string): string | null {
  const id = `${prefix}-${keySlug(key)}`;
  const text = t(id);
  return text === id ? null : text;
}

/** The family a key belongs to (`serverText.mdn-body` → `serverText.*`). */
function familyOf(key: string): string | null {
  const dot = key.indexOf('.');
  return key.startsWith('serverText.') && dot > 0 ? `${key.slice(0, dot)}.*` : null;
}

export function settingLabel(key: string): string {
  const own = lookup('admin-settings-key', key);
  if (own !== null) return own;
  const family = familyOf(key);
  const name = family !== null ? lookup('admin-settings-key', family) : null;
  return name !== null ? `${name}: ${key.slice(key.indexOf('.') + 1)}` : key;
}

export function settingHelp(key: string): string {
  return lookup('admin-settings-help', key) ?? lookup('admin-settings-help', familyOf(key) ?? key) ?? '';
}

/** The category's plain name, or the category id. */
export function categoryLabel(category: string): string {
  return lookup('admin-settings-cat', category) ?? category;
}
