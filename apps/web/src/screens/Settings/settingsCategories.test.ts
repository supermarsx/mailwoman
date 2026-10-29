import { describe, expect, it } from 'vitest';
import { ICON_NAMES } from '../../components/Icon.tsx';
import { SETTINGS_CATEGORIES } from './settingsCategories.ts';

describe('Settings section icons (t86)', () => {
  const sections = SETTINGS_CATEGORIES.flatMap((category) => category.sections);
  it('every section has its own icon from the icon module', () => {
    const icons = sections.map((section) => section.icon);
    expect(new Set(icons).size).toBe(icons.length);
    for (const icon of icons) expect(ICON_NAMES).toContain(icon);
  });
  it('the sections show the icons chosen for them', () => {
    expect(Object.fromEntries(sections.map((section) => [section.id, section.icon]))).toEqual({
      'linked-accounts': 'mailboxes', identities: 'identityProvider', signatures: 'signature', security: 'shield', privacy: 'eyeOff',
      reader: 'mailRead', compose: 'compose', folders: 'folder', labels: 'tag', rules: 'filter', searches: 'search', receipts: 'mailCheck', 'mail-transfer': 'transfer',
      appearance: 'palette', shortcuts: 'keyboard', notifications: 'bell', time: 'clock',
      nextcloud: 'nextcloud', packages: 'package', access: 'key',
      metadata: 'serverMetadata',
    });
  });
});
