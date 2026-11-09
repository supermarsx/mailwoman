import { expect, it } from 'vitest';
import { loadCatalog } from '../../i18n';
import { settingLabel, settingHelp } from '../Admin/Settings/labels.ts';
it('gives administrators readable original HTML capability and preference names with their dependencies', async () => {
  await loadCatalog('admin-settings');
  expect(settingLabel('cap.reading.original_html')).toBe('Allow Original HTML capability');
  expect(settingLabel('mail.reader.allowOriginalHtml')).toBe('Offer Original HTML per message');
  expect(settingHelp('cap.reading.original_html')).toContain('feature profile');
  expect(settingHelp('mail.reader.allowOriginalHtml')).toContain('plain-text-only mode');
});
