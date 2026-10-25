import { tagIconName } from '../../../components/Icon.tsx';
import { describe, it, expect } from 'vitest';
import { render } from '@solidjs/testing-library';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { DIRECTIONAL, ICON_NAMES, Icon } from './icons.tsx';
import { SECTIONS } from './sections.ts';

describe('admin icons', () => {
  it('every icon is a 16x16 decorative SVG in currentColor', () => {
    for (const name of ICON_NAMES) {
      const { container, unmount } = render(() => <Icon name={name} />);
      const svg = container.querySelector('svg')!;
      expect(svg.getAttribute('viewBox')).toBe('0 0 16 16');
      expect(svg.getAttribute('width')).toBe('16');
      expect(svg.getAttribute('aria-hidden')).toBe('true');
      const shapes = svg.querySelectorAll('rect, circle, path, polyline, line');
      expect(shapes.length, name).toBeGreaterThan(0);
      for (const s of shapes) {
        const paint = [s.getAttribute('fill'), s.getAttribute('stroke')].filter((v) => v !== null && v !== 'none');
        // No hard-coded colour: only currentColor paints the shape.
        expect(paint.every((p) => p === 'currentColor'), name).toBe(true);
        const sw = s.getAttribute('stroke-width');
        if (sw !== null) expect(sw, name).toBe('2');
      }
      unmount();
    }
  });

  it('directional glyphs carry the mirroring class and the others do not', () => {
    for (const name of ICON_NAMES) {
      const { container, unmount } = render(() => <Icon name={name} />);
      const cls = container.querySelector('svg')!.getAttribute('class') ?? '';
      expect(cls.includes('icon--directional'), name).toBe(DIRECTIONAL.has(name));
      unmount();
    }
  });

  it('every section has its own icon', () => {
    const icons = SECTIONS.map((s) => s.icon);
    expect(new Set(icons).size).toBe(icons.length);
    for (const i of icons) expect(ICON_NAMES).toContain(i);
  });

  it('no icon library is imported: the icons are this one file', () => {
    const src = readFileSync(join(process.cwd(), 'src/components/Icon.tsx'), 'utf8');
    const imports = [...src.matchAll(/^import[^\n]*from '([^']+)'/gm)].map((m) => m[1]);
    expect(imports).toEqual(['solid-js']);
  });
});

it('legacy saved tag glyphs resolve to native SVG names without rendering arbitrary text', () => {
  expect(tagIconName('\u{1f4bc}')).toBe('archive');
  expect(tagIconName('clock')).toBe('clock');
  expect(tagIconName('<script>')).toBe('tag');
});
