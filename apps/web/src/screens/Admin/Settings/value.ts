// Value helpers of the admin Settings section (worker f01): client-side checks
// from the registry's declared type and bounds (UX5), before any request, and
// the short display of a value in the list.

import { formatLanguageName, formatNumber, t } from '../../../i18n';
import type { SettingKind } from '../../../api/admin/settings.ts';

/** What an editor holds: text for most kinds, a boolean for `bool`. */
export type Draft = string | boolean;

export type Parsed = { ok: true; value: unknown } | { ok: false; error: string };

const SLUG = /^[a-z0-9-]*$/;
const EMAIL = /^[^\s@]{1,64}@[a-z0-9.-]+\.[a-z0-9-]+$/i;
const DOMAIN = /^(?=.{1,253}$)([a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9-]{2,63}$/i;

function isUrl(s: string, schemes: readonly string[]): boolean {
  try {
    const u = new URL(s);
    return schemes.includes(u.protocol.replace(':', '')) && u.hostname !== '';
  } catch {
    return false;
  }
}

/** The editor text of a stored value. */
export function toDraft(kind: SettingKind, value: unknown): Draft {
  if (kind.type === 'bool') return value === true;
  if (value === null || value === undefined) return '';
  if (typeof value === 'string') return value;
  if (typeof value === 'number') return String(value);
  return JSON.stringify(value, null, 2);
}

/** Whether a kind is edited as JSON text. */
export function isJsonKind(kind: SettingKind): boolean {
  return ['localeMap', 'intList', 'quietHours', 'links'].includes(kind.type);
}

/** Check a draft against the declared type and bounds; the server checks again. */
export function parseDraft(kind: SettingKind, draft: Draft, nullable: boolean): Parsed {
  if (kind.type === 'bool') return { ok: true, value: draft === true };
  const text = typeof draft === 'string' ? draft.trim() : '';
  if (text === '') {
    return nullable ? { ok: true, value: null } : { ok: false, error: t('admin-ux-err-required') };
  }
  switch (kind.type) {
    case 'int':
    case 'float': {
      const n = Number(text);
      if (!Number.isFinite(n)) return { ok: false, error: t('admin-ux-err-number') };
      if (kind.type === 'int' && !Number.isInteger(n)) return { ok: false, error: t('admin-ux-err-whole') };
      if (kind.min !== undefined && n < kind.min) return { ok: false, error: t('admin-ux-err-min', { min: kind.min }) };
      if (kind.max !== undefined && n > kind.max) return { ok: false, error: t('admin-ux-err-max', { max: kind.max }) };
      return { ok: true, value: n };
    }
    case 'enum':
    case 'locale': {
      const allowed = kind.values ?? [];
      return allowed.length === 0 || allowed.includes(text)
        ? { ok: true, value: text }
        : { ok: false, error: t('admin-ux-err-option') };
    }
    case 'text': {
      if (kind.maxLength !== undefined && [...text].length > kind.maxLength) {
        return { ok: false, error: t('admin-ux-err-max-length', { max: kind.maxLength }) };
      }
      const ok =
        kind.format === 'slug'
          ? SLUG.test(text)
          : kind.format === 'httpsUrl'
            ? isUrl(text, ['https'])
            : kind.format === 'httpUrl'
              ? isUrl(text, ['http', 'https'])
              : kind.format === 'email'
                ? EMAIL.test(text)
                : kind.format === 'domain'
                  ? DOMAIN.test(text)
                  : kind.format === 'contact'
                    ? text.startsWith('mailto:')
                      ? EMAIL.test(text.slice(7))
                      : isUrl(text, ['https'])
                    : true;
      if (ok) return { ok: true, value: text };
      const isLink = kind.format === 'httpsUrl' || kind.format === 'httpUrl' || kind.format === 'contact';
      return {
        ok: false,
        error: t(isLink ? 'admin-ux-err-url' : kind.format === 'email' ? 'admin-ux-err-email' : 'admin-ux-err-pattern'),
      };
    }
    default: {
      try {
        return { ok: true, value: JSON.parse(text) as unknown };
      } catch {
        return { ok: false, error: t('admin-settings-json-invalid') };
      }
    }
  }
}

/** A short, plain display of a value for the list. */
export function displayValue(kind: SettingKind, value: unknown): string {
  if (value === null || value === undefined || value === '') return t('admin-settings-value-empty');
  if (typeof value === 'boolean') return value ? t('admin-settings-value-on') : t('admin-settings-value-off');
  if (typeof value === 'number') return formatNumber(value);
  if (kind.type === 'locale' && typeof value === 'string' && value !== 'browser') return formatLanguageName(value);
  if (typeof value === 'string') return value;
  if (Array.isArray(value)) return t('admin-settings-value-list', { count: value.length });
  if (typeof value === 'object') return t('admin-settings-value-list', { count: Object.keys(value).length });
  return String(value);
}
