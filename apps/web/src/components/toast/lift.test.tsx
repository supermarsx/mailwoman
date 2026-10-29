// t67: the toast column stays clear of a floating bar (the bulk selection bar).
import { afterEach, expect, it } from 'vitest';
import { cleanup, render, screen } from '@solidjs/testing-library';
import { ToastRegion, resetToasts, toast } from './index.ts';
import { registerToastObstacle } from './toast.ts';

const rect = (left: number, top: number, width: number, height: number): DOMRect =>
  ({ left, top, width, height, right: left + width, bottom: top + height, x: left, y: top, toJSON: () => ({}) }) as DOMRect;

afterEach(() => { cleanup(); resetToasts(); });

it('lifts the toasts above a bar they would cover, and drops the lift when the bar goes', async () => {
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: 900 });
  render(() => <ToastRegion />);
  toast.info('Loading images…');
  const item = document.querySelector<HTMLElement>('[data-toast-id]')!;
  item.getBoundingClientRect = () => rect(1100, 840, 320, 36);
  const region = screen.getByTestId('toast-region');
  // A bar at the bottom whose right end reaches under the toast column.
  const bar = document.createElement('div'); document.body.append(bar);
  bar.getBoundingClientRect = () => rect(700, 836, 500, 44);
  const stop = registerToastObstacle(bar);
  await Promise.resolve(); await Promise.resolve();
  expect(region.style.getPropertyValue('--mw-toast-lift')).toBe(`${900 - 836 + 12}px`);
  stop();
  await Promise.resolve(); await Promise.resolve();
  expect(region.style.getPropertyValue('--mw-toast-lift')).toBe('');
  bar.remove();
});

it('does not lift for a bar beside the toast column', async () => {
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: 900 });
  render(() => <ToastRegion />);
  toast.info('Saved');
  document.querySelector<HTMLElement>('[data-toast-id]')!.getBoundingClientRect = () => rect(1100, 840, 320, 36);
  const bar = document.createElement('div'); document.body.append(bar);
  bar.getBoundingClientRect = () => rect(300, 836, 500, 44);
  const stop = registerToastObstacle(bar);
  await Promise.resolve(); await Promise.resolve();
  expect(screen.getByTestId('toast-region').style.getPropertyValue('--mw-toast-lift')).toBe('');
  stop(); bar.remove();
});
