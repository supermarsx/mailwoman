import { fireEvent, render, screen } from '@solidjs/testing-library';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { TooltipLayer, tooltipDescription } from './TooltipLayer.tsx';

beforeEach(() => { vi.useFakeTimers(); });
afterEach(() => { vi.useRealTimers(); });
const settle = async (ms = 0) => { await vi.advanceTimersByTimeAsync(ms); };

it('adopts titles on focus, preserves existing descriptions and accessible names, and dismisses on Escape', async () => {
  render(() => <><button title="Restore the inherited value" aria-label="Reset" aria-describedby="existing" /><TooltipLayer /></>);
  const button = screen.getByRole('button', { name: 'Reset' });
  fireEvent.focusIn(button);
  await settle();
  const tooltip = screen.getByRole('tooltip');
  expect(tooltip).toHaveTextContent('Restore the inherited value');
  expect(button.getAttribute('aria-describedby')).toContain(tooltip.id);
  expect(button.getAttribute('aria-describedby')).toContain('existing');
  expect(button).not.toHaveAttribute('title');
  fireEvent.keyDown(button, { key: 'Escape' });
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  expect(button).toHaveAttribute('aria-describedby', 'existing');
  expect(button).toHaveAccessibleName('Reset');
  expect(button).not.toHaveAttribute('title');
  fireEvent.pointerOut(button, { relatedTarget: document.body });
  expect(button).toHaveAttribute('title', 'Restore the inherited value');
});

it('waits for hover, prefers informative explicit descriptions, and cleans up removed targets', async () => {
  render(() => <><button title="Refresh" data-tooltip="Fetch the latest messages from this account">Refresh</button><TooltipLayer /></>);
  const button = screen.getByRole('button', { name: 'Refresh' });
  fireEvent.pointerOver(button, { pointerType: 'mouse' });
  await settle(200);
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  await settle(150);
  expect(screen.getByRole('tooltip')).toHaveTextContent('Fetch the latest messages from this account');
  button.remove();
  await settle(200);
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
});

it('keeps touch activation unobstructed and ignores editable user content', async () => {
  const click = vi.fn();
  render(() => <><button aria-label="Archive" onClick={click} /><div contenteditable="true"><button title="User supplied title">Content</button></div><TooltipLayer /></>);
  const button = screen.getByRole('button', { name: 'Archive' });
  const touch = new Event('pointerdown', { bubbles: true });
  Object.defineProperty(touch, 'pointerType', { value: 'touch' });
  button.dispatchEvent(touch);
  fireEvent.focusIn(button); fireEvent.click(button);
  await settle(400);
  expect(click).toHaveBeenCalledOnce();
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  expect(tooltipDescription(screen.getByRole('button', { name: 'Content' }))).toBeNull();
});

it('clamps placement and dismisses an anchor scrolled out of the viewport', async () => {
  render(() => <><button aria-label="Close" /><TooltipLayer /></>);
  const button = screen.getByRole('button', { name: 'Close' });
  const rect = vi.spyOn(button, 'getBoundingClientRect').mockReturnValue({ left: 0, right: 20, top: 0, bottom: 20, width: 20, height: 20 } as DOMRect);
  fireEvent.focusIn(button); await settle();
  expect(screen.getByRole('tooltip').style.left).toBe('10px');
  expect(Number.parseFloat(screen.getByRole('tooltip').style.top)).toBeGreaterThanOrEqual(8);
  rect.mockReturnValue({ left: 0, right: 20, top: -200, bottom: -180, width: 20, height: 20 } as DOMRect);
  fireEvent.scroll(document); await settle(20);
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
});
