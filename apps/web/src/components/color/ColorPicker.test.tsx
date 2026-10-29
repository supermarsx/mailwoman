import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { ColorField, resolveAgainst, type ColorFieldProps } from './ColorPicker.tsx';

function harness(extra: Partial<ColorFieldProps> = {}, initial: string | null = '#336699') {
  const changes: (string | null)[] = [];
  const previews: (string | null)[] = [];
  const [value, setValue] = createSignal<string | null>(initial);
  render(() => (
    <div>
      <button type="button">before</button>
      <ColorField label="Text colour" value={value()} onChange={(v) => { changes.push(v); setValue(v); }} onPreview={(v) => previews.push(v)} recentOwner="t81-test" {...extra} />
    </div>
  ));
  const trigger = () => screen.getByRole('button', { name: /^Text colour:/ });
  const dialog = () => screen.getByRole('dialog', { name: 'Text colour' });
  const hex = () => screen.getByRole('textbox', { name: 'Hex' }) as HTMLInputElement;
  return { changes, previews, value, trigger, dialog, hex };
}

beforeEach(() => localStorage.clear());
afterEach(() => cleanup());

describe('ColorField (t81)', () => {
  it('names the trigger with its value and opens a labelled dialog', () => {
    const h = harness();
    expect(h.trigger()).toHaveAttribute('aria-label', 'Text colour: #336699');
    expect(h.trigger()).toHaveAttribute('aria-expanded', 'false');
    fireEvent.click(h.trigger());
    expect(h.trigger()).toHaveAttribute('aria-expanded', 'true');
    expect(h.dialog()).toBeVisible();
    expect(h.hex().value).toBe('#336699');
    expect(screen.getByRole('slider', { name: 'Saturation and brightness' })).toBeInTheDocument();
    expect(screen.getByRole('slider', { name: 'Hue' })).toHaveAttribute('aria-valuenow', '210');
    expect(screen.queryByRole('slider', { name: 'Opacity' })).toBeNull();
  });

  it('moves saturation and brightness with arrow keys, live-previews, and commits on Enter', () => {
    const h = harness({}, '#808080');
    fireEvent.click(h.trigger());
    const area = screen.getByRole('slider', { name: 'Saturation and brightness' });
    expect(area).toHaveAttribute('aria-valuetext', 'Saturation 0%, brightness 50%');
    fireEvent.keyDown(area, { key: 'ArrowUp', shiftKey: true });
    expect(area).toHaveAttribute('aria-valuetext', 'Saturation 0%, brightness 60%');
    fireEvent.keyDown(area, { key: 'ArrowRight' });
    expect(area).toHaveAttribute('aria-valuenow', '1');
    fireEvent.keyDown(area, { key: 'End' });
    expect(area).toHaveAttribute('aria-valuenow', '100');
    expect(h.previews.length).toBe(3);
    expect(h.changes).toEqual([]);
    fireEvent.keyDown(area, { key: 'Enter' });
    expect(h.changes).toHaveLength(1);
    expect(h.changes[0]).toMatch(/^#[0-9a-f]{6}$/);
    expect(h.trigger()).toHaveAttribute('aria-expanded', 'false');
    expect(document.activeElement).toBe(h.trigger());
  });

  it('steps the hue and clamps it at the ends', () => {
    const h = harness({}, '#ff0000');
    fireEvent.click(h.trigger());
    const hue = screen.getByRole('slider', { name: 'Hue' });
    fireEvent.keyDown(hue, { key: 'ArrowLeft' });
    expect(hue).toHaveAttribute('aria-valuenow', '0');
    fireEvent.keyDown(hue, { key: 'PageUp' });
    expect(hue).toHaveAttribute('aria-valuenow', '10');
    fireEvent.keyDown(hue, { key: 'End' });
    expect(hue).toHaveAttribute('aria-valuenow', '360');
    fireEvent.keyDown(hue, { key: 'Home' });
    fireEvent.keyDown(hue, { key: 'ArrowRight', shiftKey: true });
    expect(hue).toHaveAttribute('aria-valuetext', '10 degrees');
  });

  it('Escape restores the opening value, closes and returns focus', () => {
    const h = harness();
    fireEvent.click(h.trigger());
    fireEvent.input(h.hex(), { target: { value: '#ff0000' } });
    expect(h.previews.at(-1)).toBe('#ff0000');
    fireEvent.keyDown(h.hex(), { key: 'Escape' });
    expect(h.previews.at(-1)).toBe('#336699');
    expect(h.changes).toEqual([]);
    expect(h.trigger()).toHaveAttribute('aria-expanded', 'false');
    expect(document.activeElement).toBe(h.trigger());
  });

  it('validates the hex field and refuses to commit an invalid one', () => {
    const h = harness();
    fireEvent.click(h.trigger());
    fireEvent.input(h.hex(), { target: { value: '#12' } });
    expect(h.hex()).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByRole('alert')).toHaveTextContent('#RGB or #RRGGBB');
    expect(screen.getByRole('button', { name: 'Done' })).toBeDisabled();
    fireEvent.keyDown(h.hex(), { key: 'Enter' });
    expect(h.trigger()).toHaveAttribute('aria-expanded', 'true');
    fireEvent.input(h.hex(), { target: { value: 'ABC' } });
    expect(h.hex()).toHaveAttribute('aria-invalid', 'false');
    fireEvent.keyDown(h.hex(), { key: 'Enter' });
    expect(h.changes).toEqual(['#aabbcc']);
  });

  it('takes RGB channels', () => {
    const h = harness({}, '#000000');
    fireEvent.click(h.trigger());
    fireEvent.input(screen.getByRole('spinbutton', { name: 'Red' }), { target: { value: '255' } });
    expect(h.hex().value).toBe('#ff0000');
    fireEvent.input(screen.getByRole('spinbutton', { name: 'Blue' }), { target: { value: '999' } });
    expect(h.hex().value).toBe('#ff0000');
  });

  it('traps Tab inside the popover', () => {
    const h = harness();
    fireEvent.click(h.trigger());
    const done = screen.getByRole('button', { name: 'Done' });
    done.focus();
    fireEvent.keyDown(done, { key: 'Tab' });
    expect(document.activeElement).toBe(screen.getByRole('slider', { name: 'Saturation and brightness' }));
    fireEvent.keyDown(document.activeElement!, { key: 'Tab', shiftKey: true });
    expect(document.activeElement).toBe(done);
  });

  it('offers the theme accents and the none choice, and commits none', () => {
    const h = harness({ emptyLabel: 'Automatic' });
    fireEvent.click(h.trigger());
    const theme = screen.getByRole('group', { name: 'Theme colours' });
    expect(theme.querySelectorAll('[data-color-swatch]').length).toBeGreaterThanOrEqual(10);
    expect(screen.getByRole('button', { name: /^Blue \(#/ })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Automatic' }));
    expect(screen.getByRole('button', { name: 'Automatic' })).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: 'Done' }));
    expect(h.changes).toEqual([null]);
    expect(h.trigger()).toHaveAttribute('aria-label', 'Text colour: Automatic');
  });

  it('remembers committed colours as recents for the owner', () => {
    const h = harness();
    fireEvent.click(h.trigger());
    fireEvent.input(h.hex(), { target: { value: '#123456' } });
    fireEvent.click(screen.getByRole('button', { name: 'Done' }));
    fireEvent.click(h.trigger());
    const recent = screen.getByRole('group', { name: 'Recent colours' });
    expect(recent.querySelector('[data-color-swatch="#123456"]')).not.toBeNull();
  });

  it('commits on a click outside', () => {
    const h = harness();
    fireEvent.click(h.trigger());
    fireEvent.input(h.hex(), { target: { value: '#00ff00' } });
    fireEvent.pointerDown(screen.getByRole('button', { name: 'before' }));
    expect(h.changes).toEqual(['#00ff00']);
  });

  it('shows a WCAG hint for text against the theme background', () => {
    const h = harness({ contrast: { against: '#ffffff', use: 'text' } }, '#bbbbbb');
    fireEvent.click(h.trigger());
    const hint = h.dialog().querySelector('[data-color-contrast]');
    expect(hint).toHaveAttribute('data-color-contrast', 'fail');
    expect(hint).toHaveTextContent('Below AA');
    fireEvent.input(h.hex(), { target: { value: '#000000' } });
    expect(h.dialog().querySelector('[data-color-contrast]')).toHaveAttribute('data-color-contrast', 'aa');
    expect(h.dialog().querySelector('[data-color-contrast]')).toHaveTextContent('21.00:1');
  });

  it('adds an opacity strip only when alpha is supported', () => {
    const h = harness({ alpha: true }, '#33669980');
    fireEvent.click(h.trigger());
    const strip = screen.getByRole('slider', { name: 'Opacity' });
    expect(strip).toHaveAttribute('aria-valuenow', '50');
    fireEvent.keyDown(strip, { key: 'End' });
    expect(h.hex().value).toBe('#336699');
  });

  it('shows the eyedropper only where the EyeDropper API exists', () => {
    const h = harness();
    fireEvent.click(h.trigger());
    expect(screen.queryByRole('button', { name: 'Pick from screen' })).toBeNull();
    fireEvent.keyDown(h.hex(), { key: 'Escape' });
    const open = vi.fn().mockResolvedValue({ sRGBHex: '#abcdef' });
    (window as unknown as { EyeDropper: unknown }).EyeDropper = class { open = open; };
    try {
      fireEvent.click(h.trigger());
      expect(screen.getByRole('button', { name: 'Pick from screen' })).toBeInTheDocument();
    } finally {
      delete (window as unknown as { EyeDropper?: unknown }).EyeDropper;
    }
  });

  it('sets colours through the CSSOM, never through a style attribute', () => {
    const set = vi.spyOn(Element.prototype, 'setAttribute');
    try {
      const h = harness();
      fireEvent.click(h.trigger());
      fireEvent.input(h.hex(), { target: { value: '#00ff00' } });
      const swatch = h.dialog().querySelector<HTMLElement>('[data-color-swatch]')!;
      expect(swatch.style.backgroundColor).not.toBe('');
      expect(h.dialog().querySelector<HTMLElement>('[data-color-preview]')!.style.backgroundColor).toBe('rgb(0, 255, 0)');
      expect(set.mock.calls.filter(([name]) => name.toLowerCase() === 'style')).toEqual([]);
    } finally {
      set.mockRestore();
    }
  });

  it('resolves theme keys against the active palette', () => {
    expect(resolveAgainst('#123456', 'light')).toBe('#123456');
    expect(resolveAgainst('bg', 'light')).toMatch(/^#/);
  });
});
