import { describe, expect, it } from 'vitest';
import { IMAGE_WIDTH_STEPS, imageWidthHint } from './remote-content.ts';

describe('image width hint', () => {
  it('is null unless the server turned resizing on', () => {
    expect(imageWidthHint({}, 800, 1)).toBeNull();
    expect(imageWidthHint({ imageResizeMaxWidth: null }, 800, 1)).toBeNull();
  });
  it('snaps up to the smallest accepted step that covers the frame', () => {
    const p = { imageResizeMaxWidth: 1600 };
    expect(imageWidthHint(p, 300, 1)).toBe(320);
    expect(imageWidthHint(p, 321, 1)).toBe(640);
    expect(imageWidthHint(p, 600, 2)).toBe(1280);
    expect(imageWidthHint(p, 2400, 3)).toBe(1600);
  });
  it('never exceeds the configured maximum and only uses accepted steps', () => {
    expect(imageWidthHint({ imageResizeMaxWidth: 960 }, 1500, 2)).toBe(960);
    expect(imageWidthHint({ imageResizeMaxWidth: 100 }, 500, 1)).toBeNull();
    for (let w = 1; w < 3000; w += 37) expect(IMAGE_WIDTH_STEPS as readonly (number | null)[]).toContain(imageWidthHint({ imageResizeMaxWidth: 1600 }, w, 1));
  });
  it('ignores a nonsensical frame width', () => {
    expect(imageWidthHint({ imageResizeMaxWidth: 1600 }, 0, 1)).toBeNull();
    expect(imageWidthHint({ imageResizeMaxWidth: 1600 }, Number.NaN, 1)).toBeNull();
  });
});
