import { createSignal } from 'solid-js';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { ResizeHandle } from './ResizeHandle.tsx';

it('exposes pane bounds and supports fine/coarse keyboard sizing and bounds', () => {
  const commit = vi.fn();
  render(() => {
    const [value, setValue] = createSignal(220);
    return <ResizeHandle label="Folder width" controls="folders" value={value()} min={180} max={400} onChange={setValue} onCommit={commit} />;
  });
  const handle = screen.getByRole('separator', { name: 'Folder width' });
  expect(handle).toHaveAttribute('aria-controls', 'folders');
  fireEvent.keyDown(handle, { key: 'ArrowRight' });
  expect(handle).toHaveAttribute('aria-valuenow', '230');
  fireEvent.keyDown(handle, { key: 'ArrowLeft', shiftKey: true });
  expect(handle).toHaveAttribute('aria-valuenow', '190');
  fireEvent.keyDown(handle, { key: 'Home' });
  fireEvent.keyDown(handle, { key: 'ArrowLeft' });
  expect(handle).toHaveAttribute('aria-valuenow', '180');
  fireEvent.keyDown(handle, { key: 'End' });
  expect(commit).toHaveBeenLastCalledWith(400);
});

it('uses the correct axis and supports trailing panes', () => {
  const change = vi.fn();
  render(() => <ResizeHandle label="Reader height" orientation="horizontal" value={300} min={200} max={600} direction={-1} onChange={change} />);
  const handle = screen.getByRole('separator');
  fireEvent.keyDown(handle, { key: 'ArrowRight' });
  expect(change).not.toHaveBeenCalled();
  fireEvent.keyDown(handle, { key: 'ArrowDown' });
  expect(change).toHaveBeenLastCalledWith(290);
  expect(handle).toHaveAttribute('aria-orientation', 'horizontal');
});

it('clamps dragging, commits on release, and restores the starting width on cancellation', () => {
  const change = vi.fn();
  const commit = vi.fn();
  render(() => <ResizeHandle label="Folder width" value={220} min={180} max={400} onChange={change} onCommit={commit} />);
  const handle = screen.getByRole('separator');
  Object.assign(handle, { setPointerCapture: vi.fn(), hasPointerCapture: () => true, releasePointerCapture: vi.fn() });
  const pointer = (type: string, x: number) => {
    const event = new Event(type, { bubbles: true, cancelable: true });
    Object.assign(event, { pointerId: 1, button: 0, clientX: x, clientY: 0 });
    fireEvent(handle, event);
  };
  pointer('pointerdown', 220);
  pointer('pointermove', 900);
  expect(change).toHaveBeenLastCalledWith(400);
  pointer('pointerup', 900);
  expect(commit).toHaveBeenLastCalledWith(400);
  commit.mockClear();
  pointer('pointerdown', 220);
  pointer('pointermove', 100);
  expect(change).toHaveBeenLastCalledWith(180);
  pointer('pointercancel', 100);
  expect(change).toHaveBeenLastCalledWith(220);
  expect(commit).not.toHaveBeenCalled();
});
