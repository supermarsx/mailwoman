// t66: the user Settings navigation, grouped into categories. Each category
// lists its sections in order of how often they are opened. Section ids are the
// workspace routes (`navigate({section})`) and stay unchanged.
import type { IconName } from '../../components/Icon.tsx';

export interface SettingsSectionEntry { id: string; label: string; icon: IconName; terms?: string }
export interface SettingsCategory { id: string; label: string; sections: readonly SettingsSectionEntry[] }

export const SETTINGS_CATEGORIES: readonly SettingsCategory[] = [
  { id: 'account', label: 'settings-category-account', sections: [
    { id: 'linked-accounts', label: 'linked-accounts-title', icon: 'users', terms: 'mailboxes accounts switch link imap pop3 ews' },
    { id: 'identities', label: 'settings-ident-title', icon: 'users', terms: 'from address sender name' },
    { id: 'signatures', label: 'settings-sig-title', icon: 'file' },
    { id: 'security', label: 'security-center-title', icon: 'shield', terms: 'password passkey totp two factor authentication sessions encryption' },
    { id: 'privacy', label: 'privacy-title', icon: 'privacy', terms: 'remote images tracking telemetry analytics' },
  ] },
  { id: 'mail', label: 'settings-category-mail', sections: [
    { id: 'reader', label: 'reader-settings-title', icon: 'eye', terms: 'message reading images layout dark mode' },
    { id: 'compose', label: 'settings-compose-title', icon: 'edit', terms: 'editor reply quote font signature recipients cc bcc draft' },
    { id: 'folders', label: 'settings-folders-title', icon: 'folder', terms: 'sidebar mappings count pin order icons spam trash focused inbox other tabs' },
    { id: 'labels', label: 'labels-title', icon: 'tag', terms: 'tags colour color' },
    { id: 'rules', label: 'settings-nav-rules', icon: 'policy', terms: 'filter automation sieve' },
    { id: 'searches', label: 'settings-search-title', icon: 'search', terms: 'search folder' },
    { id: 'receipts', label: 'receipts-title', icon: 'check', terms: 'read receipt delivery dsn' },
    { id: 'mail-transfer', label: 'mail-transfer-title', icon: 'download', terms: 'import export download eml mbox' },
  ] },
  { id: 'interface', label: 'settings-category-interface', sections: [
    { id: 'appearance', label: 'settings-appearance', icon: 'branding', terms: 'theme dark light color accent badge unread counter css javascript font density branding toolbar language' },
    { id: 'shortcuts', label: 'settings-nav-keyboard-display', icon: 'configuration', terms: 'keyboard display key hotkey drag selection' },
    { id: 'notifications', label: 'settings-notif-title', icon: 'alert', terms: 'push alerts sound browser' },
    { id: 'time', label: 'settings-time-title', icon: 'clock', terms: 'timezone date clock iana' },
  ] },
  { id: 'integrations', label: 'settings-category-integrations', sections: [
    { id: 'nextcloud', label: 'nextcloud-account-title', icon: 'nextcloud', terms: 'files contacts calendar cloud' },
    { id: 'packages', label: 'packages-title', icon: 'gift', terms: 'parcel delivery tracking' },
    { id: 'access', label: 'settings-nav-access', icon: 'lock', terms: 'api keys mcp tokens sharing permissions' },
  ] },
  { id: 'about', label: 'settings-category-about', sections: [
    { id: 'metadata', label: 'settings-nav-metadata', icon: 'info', terms: 'server metadata version' },
  ] },
];

/** The category a section belongs to, or undefined for a section outside the nav. */
export function categoryOf(section: string): SettingsCategory | undefined {
  return SETTINGS_CATEGORIES.find(category => category.sections.some(entry => entry.id === section));
}

/**
 * The categories to show: sections the caller does not support are dropped,
 * sections that do not match every query term are dropped, and a category left
 * with no section is dropped. A query term may match the category name too.
 */
export function visibleCategories(
  supported: (section: string) => boolean,
  query: string,
  translate: (id: string) => string,
): Array<{ category: SettingsCategory; sections: SettingsSectionEntry[] }> {
  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  return SETTINGS_CATEGORIES.map(category => {
    const name = translate(category.label);
    const sections = category.sections.filter(entry => supported(entry.id)
      && terms.every(term => `${translate(entry.label)} ${name} ${entry.terms ?? ''}`.toLocaleLowerCase().includes(term)));
    return { category, sections };
  }).filter(group => group.sections.length > 0);
}
