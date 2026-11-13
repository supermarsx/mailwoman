import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { ACTION_ICONS, ACTION_MENUS, snoozePresetIcon, type ActionId } from './actionIcons.ts';
import { ICON_NAMES, Icon, type IconName } from './components/Icon.tsx';
import { snoozePresets } from './components/MessageActions.tsx';

/** The icons that were already drawn the same before the action set: kept, but never used for two actions. */
const KNOWN_TWINS: readonly [IconName, IconName][] = [['copy', 'clone']];

function svgOf(name: IconName): SVGElement {
  return Icon({ name }) as unknown as SVGElement;
}

const used = [...new Set(Object.values(ACTION_ICONS))] as IconName[];

describe('action icons', () => {
  it('every action names an icon that exists in the icon module', () => {
    for (const [action, icon] of Object.entries(ACTION_ICONS)) expect(ICON_NAMES, `${action} -> ${icon}`).toContain(icon);
  });

  it('every icon an action uses is a 16x16, 2 px, currentColor glyph like the rest of the set', () => {
    for (const name of used) {
      const svg = svgOf(name);
      expect(svg.getAttribute('viewBox'), name).toBe('0 0 16 16');
      expect(svg.getAttribute('aria-hidden'), name).toBe('true');
      const shapes = [...svg.querySelectorAll('rect, circle, path, polyline, line')];
      expect(shapes.length, name).toBeGreaterThan(0);
      for (const shape of shapes) {
        const fill = shape.getAttribute('fill');
        const stroke = shape.getAttribute('stroke');
        expect(fill !== null || stroke !== null, `${name} paints something`).toBe(true);
        for (const paint of [fill, stroke]) if (paint !== null && paint !== 'none') expect(paint, name).toBe('currentColor');
        const width = shape.getAttribute('stroke-width');
        if (width !== null) expect(width, name).toBe('2');
      }
    }
  });

  it('no two actions shown together in one menu share an icon', () => {
    for (const [menu, actions] of Object.entries(ACTION_MENUS)) {
      const seen = new Map<IconName, ActionId>();
      for (const action of actions) {
        expect(ACTION_ICONS, `${menu} lists the unknown action ${action}`).toHaveProperty(action);
        const icon = ACTION_ICONS[action];
        const twin = seen.get(icon);
        if (twin !== undefined && twin !== action) throw new Error(`${menu}: ${twin} and ${action} both use "${icon}"`);
        seen.set(icon, action);
      }
    }
  });

  it('different icons used by actions are drawn differently', () => {
    const drawn = new Map<string, IconName>();
    for (const name of used) {
      const markup = svgOf(name).innerHTML;
      const other = drawn.get(markup);
      if (other !== undefined && !KNOWN_TWINS.some(([a, b]) => (a === name && b === other) || (b === name && a === other))) throw new Error(`${name} and ${other} are the same drawing`);
      drawn.set(markup, name);
    }
  });

  it('snooze presets have schedule icons distinct from the snooze button', () => {
    const icons = snoozePresets().map((preset) => snoozePresetIcon(preset.labelId));
    expect(new Set(icons).size).toBeGreaterThanOrEqual(3);
    expect(icons).not.toContain(ACTION_ICONS.snooze);
  });

  it('the menus read their icons from ACTION_ICONS, not from inline names', () => {
    const root = join(process.cwd(), 'src', 'components');
    const files = ['MessageContextMenu.tsx', 'FolderContextMenu.tsx', 'FolderPane.tsx', 'ReaderToolbar.tsx', 'ReaderMoveMenu.tsx', 'MailSelectionBar.tsx', 'MessageActions.tsx', 'Outbox.tsx', 'EmptyTrash.tsx'];
    for (const file of files) expect(readFileSync(join(root, file), 'utf8'), file).toContain("from '../actionIcons.ts'");
  });
});
