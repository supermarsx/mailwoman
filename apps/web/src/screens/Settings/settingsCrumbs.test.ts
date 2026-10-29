// t95: the user Settings breadcrumb levels are names, never message ids or section ids.
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { SETTINGS_CATEGORIES } from './settingsCategories.ts';
import { settingsCrumbCatalogsLoaded, settingsCrumbs, settingsLabel } from './settingsCrumbs.ts';

const MESSAGE_ID = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/;

beforeAll(async () => { await settingsCrumbCatalogsLoaded(); });

describe('settings breadcrumbs', () => {
  it('walks every category and section: each level has its translated name', () => {
    const raw: string[] = [];
    for (const category of SETTINGS_CATEGORIES) {
      for (const section of category.sections) {
        const levels = settingsCrumbs({ section: section.id }).map((c) => c.label);
        expect(levels).toHaveLength(2);
        for (const label of levels) {
          if (label === null || MESSAGE_ID.test(label) || label === section.id || label === category.id) raw.push(`${section.id}: ${label}`);
        }
      }
    }
    expect(raw).toEqual([]);
  });

  it('an open editor adds New or Edit, never the edited item id, and the section becomes a link', () => {
    const edit = settingsCrumbs({ section: 'labels', editor: { kind: 'edit', id: 'f3b1c2d4e5a6' } });
    expect(edit.map((c) => c.label)).toEqual(['Mail', 'Labels', 'Edit']);
    expect(edit[1]!.route).toEqual({ section: 'labels' });
    expect(edit[2]!.route).toBeUndefined();
    expect(settingsCrumbs({ section: 'packages', editor: { kind: 'new' } }).at(-1)!.label).toBe('New');
  });

  it('a section outside the navigation has no trail', () => {
    expect(settingsCrumbs({ section: 'nowhere' })).toEqual([]);
  });

  it('a missing label falls back to words from the id, logged at debug level', () => {
    const debug = vi.spyOn(console, 'debug').mockImplementation(() => {});
    expect(settingsLabel('settings-t95-missing', 'mail-transfer')).toBe('Mail transfer');
    expect(debug).toHaveBeenCalledWith(expect.stringContaining('mail-transfer'));
    debug.mockRestore();
  });
});
