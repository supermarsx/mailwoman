import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import axe from 'axe-core';
import {
  TOAST_DURATIONS,
  TOAST_LIMIT,
  ToastRegion,
  resetToasts,
  setToastPause,
  toast,
  toastTimeLeft,
  toasts,
} from './index.ts';

function messages(): string[] {
  return toasts().map((i) => i.message);
}

function setHidden(hidden: boolean): void {
  Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => (hidden ? 'hidden' : 'visible') });
  document.dispatchEvent(new Event('visibilitychange'));
}

/** Let the region's queued focus/hover recheck run. */
async function flush(): Promise<void> {
  await Promise.resolve();
}

beforeEach(() => {
  vi.useFakeTimers();
  resetToasts();
});
afterEach(() => {
  cleanup();
  resetToasts();
  vi.useRealTimers();
  Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => 'visible' });
});

describe('toast store', () => {
  it('closes each tone after its own time', () => {
    toast.success('saved');
    toast.info('note');
    toast.warning('careful');
    vi.advanceTimersByTime(TOAST_DURATIONS.success - 1);
    expect(messages()).toEqual(['saved', 'note', 'careful']);
    vi.advanceTimersByTime(1);
    expect(messages()).toEqual(['careful']);
    vi.advanceTimersByTime(TOAST_DURATIONS.warning - TOAST_DURATIONS.success);
    expect(messages()).toEqual([]);

    toast.error('failed');
    vi.advanceTimersByTime(TOAST_DURATIONS.error - 1);
    expect(messages()).toEqual(['failed']);
    vi.advanceTimersByTime(1);
    expect(messages()).toEqual([]);
  });

  it('gives a toast with an action the action time, and honours an explicit duration', () => {
    toast.success('moved', { action: { label: 'Undo', run: () => undefined } });
    toast.info('quick', { duration: 1000 });
    vi.advanceTimersByTime(1000);
    expect(messages()).toEqual(['moved']);
    vi.advanceTimersByTime(TOAST_DURATIONS.action - 1001);
    expect(messages()).toEqual(['moved']);
    vi.advanceTimersByTime(1);
    expect(messages()).toEqual([]);
  });

  it('keeps an Infinity toast until it is closed', () => {
    const id = toast.error('offline', { duration: Infinity });
    vi.advanceTimersByTime(10 * 60_000);
    expect(messages()).toEqual(['offline']);
    toast.dismiss(id);
    expect(messages()).toEqual([]);
  });

  it(`shows at most ${TOAST_LIMIT}; the oldest goes first and is told why`, () => {
    const closed: string[] = [];
    for (const n of [1, 2, 3, 4]) toast.info(`m${n}`, { onClose: (r) => closed.push(`m${n}:${r}`) });
    expect(messages()).toEqual(['m2', 'm3', 'm4']);
    expect(closed).toEqual(['m1:evicted']);
  });

  it('replaces a toast with the same id in place and restarts its clock', () => {
    toast.info('first');
    toast.info('Saving…', { id: 'save' });
    toast.info('other');
    vi.advanceTimersByTime(4000);
    toast.success('Saved', { id: 'save' });
    expect(toasts().map((i) => [i.message, i.tone])).toEqual([
      ['first', 'info'],
      ['Saved', 'success'],
      ['other', 'info'],
    ]);
    vi.advanceTimersByTime(1000);
    expect(messages()).toEqual(['Saved']);
    vi.advanceTimersByTime(4000);
    expect(messages()).toEqual([]);
  });

  it('counts the same tone and text without an id as one toast', () => {
    toast.error('Could not update');
    toast.error('Could not update');
    toast.error('Could not update');
    toast.info('Could not update');
    expect(messages()).toEqual(['Could not update', 'Could not update']);
    expect(toasts().map((i) => i.tone)).toEqual(['error', 'info']);
  });

  it('stops every clock while any pause reason holds, and resumes with the time left', () => {
    toast.success('saved');
    vi.advanceTimersByTime(2000);
    setToastPause('hover', true);
    setToastPause('hidden', true);
    vi.advanceTimersByTime(60_000);
    expect(messages()).toEqual(['saved']);
    expect(toastTimeLeft(toasts()[0]!.id)).toBe(3000);
    setToastPause('hover', false);
    vi.advanceTimersByTime(60_000);
    expect(messages()).toEqual(['saved']);
    setToastPause('hidden', false);
    vi.advanceTimersByTime(2999);
    expect(messages()).toEqual(['saved']);
    vi.advanceTimersByTime(1);
    expect(messages()).toEqual([]);
  });

  it('runs a pausable: false toast to its deadline through a pause', () => {
    setToastPause('focus', true);
    toast.info('Message sent', { pausable: false, duration: 3000 });
    toast.info('stays');
    vi.advanceTimersByTime(3000);
    expect(messages()).toEqual(['stays']);
  });

  it('reports timeout, dismiss and cleared to onClose', () => {
    const reasons: string[] = [];
    toast.info('a', { onClose: (r) => reasons.push(r) });
    const b = toast.info('b', { onClose: (r) => reasons.push(r) });
    toast.info('c', { duration: 100_000, onClose: (r) => reasons.push(r) });
    toast.dismiss(b);
    vi.advanceTimersByTime(TOAST_DURATIONS.info);
    toast.clear();
    expect(reasons).toEqual(['dismiss', 'timeout', 'cleared']);
  });
});

describe('ToastRegion', () => {
  it('makes success, info and warning role=status inside a polite live region, and errors role=alert', () => {
    const { container } = render(() => <ToastRegion />);
    // Nothing answers a role lookup while no toast is shown.
    expect(screen.queryByRole('status')).toBeNull();
    expect(screen.queryByRole('alert')).toBeNull();
    const polite = container.querySelector('[aria-live="polite"]')!;
    expect(polite).toHaveAttribute('aria-atomic', 'false');
    toast.info('Archived');
    toast.warning('Partly saved');
    toast.error('Not saved');
    const statuses = screen.getAllByRole('status');
    expect(statuses.map((n) => n.textContent)).toEqual(['Archived', 'Partly saved']);
    toast.success('Saved');
    expect(screen.getAllByRole('status').map((n) => n.textContent)).toEqual(['Partly saved', 'Saved']);
    for (const s of screen.getAllByRole('status')) {
      expect(polite.contains(s)).toBe(true);
      // Announced once, by the container.
      expect(s).toHaveAttribute('aria-live', 'off');
    }
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('Not saved');
    expect(alert).toHaveAttribute('data-tone', 'error');
    expect(alert.parentElement?.closest('[aria-live]')).toBeNull();
  });

  it('closes a toast from its close button', () => {
    render(() => <ToastRegion />);
    const closed = vi.fn();
    toast.success('Saved', { onClose: closed });
    fireEvent.click(screen.getByRole('button', { name: 'Close notification' }));
    expect(screen.queryByTestId('toast')).toBeNull();
    expect(closed).toHaveBeenCalledWith('dismiss');
  });

  it('runs the action once and closes the toast', async () => {
    render(() => <ToastRegion />);
    const run = vi.fn();
    const closed = vi.fn();
    toast.info('Archived', { action: { label: 'Undo', run }, onClose: closed });
    fireEvent.click(screen.getByRole('button', { name: 'Undo' }));
    await flush();
    expect(run).toHaveBeenCalledTimes(1);
    expect(closed).toHaveBeenCalledWith('action');
    expect(screen.queryByTestId('toast')).toBeNull();
  });

  it('closes the newest toast on Escape', () => {
    render(() => <ToastRegion />);
    toast.info('older');
    toast.error('newer');
    fireEvent.keyDown(document.body, { key: 'Escape' });
    expect(messages()).toEqual(['older']);
    fireEvent.keyDown(document.body, { key: 'Escape' });
    expect(messages()).toEqual([]);
  });

  it('leaves Escape to an open dialog or a text field', () => {
    render(() => (
      <>
        <input aria-label="Search" />
        <ToastRegion />
      </>
    ));
    toast.info('kept');
    fireEvent.keyDown(screen.getByRole('textbox', { name: 'Search' }), { key: 'Escape' });
    expect(messages()).toEqual(['kept']);
    const dialog = document.createElement('div');
    dialog.setAttribute('aria-modal', 'true');
    document.body.append(dialog);
    fireEvent.keyDown(document.body, { key: 'Escape' });
    expect(messages()).toEqual(['kept']);
    // Inside the region Escape is the toast's, dialog or not.
    fireEvent.keyDown(screen.getByRole('button', { name: 'Close notification' }), { key: 'Escape' });
    expect(messages()).toEqual([]);
    dialog.remove();
  });

  it('stops the clock while the pointer rests on a toast', () => {
    render(() => <ToastRegion />);
    toast.success('Saved');
    vi.advanceTimersByTime(1000);
    fireEvent.pointerOver(screen.getByTestId('toast'));
    expect(screen.getByTestId('toast')).toHaveAttribute('data-paused', 'true');
    vi.advanceTimersByTime(60_000);
    expect(messages()).toEqual(['Saved']);
    fireEvent.pointerOver(document.body);
    vi.advanceTimersByTime(3999);
    expect(messages()).toEqual(['Saved']);
    vi.advanceTimersByTime(1);
    expect(messages()).toEqual([]);
  });

  it('stops the clock while focus is in the region and hands focus back when its toast closes', async () => {
    render(() => (
      <>
        <button type="button">Before</button>
        <ToastRegion />
      </>
    ));
    const before = screen.getByRole('button', { name: 'Before' });
    before.focus();
    toast.success('Saved');
    screen.getByRole('button', { name: 'Close notification' }).focus();
    vi.advanceTimersByTime(60_000);
    expect(messages()).toEqual(['Saved']);
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(messages()).toEqual([]);
    expect(document.activeElement).toBe(before);
    await flush();
    toast.success('Next');
    vi.advanceTimersByTime(TOAST_DURATIONS.success);
    expect(messages()).toEqual([]);
  });

  it('moves focus to the next toast when the focused one closes', () => {
    render(() => <ToastRegion />);
    toast.info('one');
    toast.info('two');
    const [first] = screen.getAllByRole('button', { name: 'Close notification' });
    first!.focus();
    fireEvent.click(first!);
    expect(messages()).toEqual(['two']);
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Close notification' }));
  });

  it('stops the clock while the window is hidden', () => {
    render(() => <ToastRegion />);
    toast.error('Failed');
    setHidden(true);
    vi.advanceTimersByTime(60_000);
    expect(messages()).toEqual(['Failed']);
    setHidden(false);
    vi.advanceTimersByTime(TOAST_DURATIONS.error);
    expect(messages()).toEqual([]);
  });

  it('shows the seconds left of a countdown toast', () => {
    render(() => <ToastRegion />);
    toast.info('Message sent', { countdown: true, duration: 10_000, action: { label: 'Cancel', run: () => undefined } });
    vi.advanceTimersByTime(250);
    expect(screen.getByTestId('toast')).toHaveTextContent('10 s');
    vi.advanceTimersByTime(3000);
    expect(screen.getByTestId('toast')).toHaveTextContent('7 s');
  });

  it('renders no style attribute and passes axe', async () => {
    const { container } = render(() => <ToastRegion />);
    toast.success('Saved');
    toast.error('Not saved', { action: { label: 'Retry', run: () => undefined } });
    toast.info('Archived', { countdown: true, action: { label: 'Undo', run: () => undefined } });
    expect([...container.querySelectorAll('[style]')]).toEqual([]);
    vi.useRealTimers();
    const result = await axe.run(container, { rules: { 'color-contrast': { enabled: false }, region: { enabled: false } } });
    expect(result.violations.map((v) => v.id)).toEqual([]);
  });
});

it('reserves one persistent job slot without increasing the ordinary notification limit',()=>{
 const closed=vi.fn();toast.info('Job',{id:'job',persistent:true,duration:Infinity,onClose:closed});
 for(let i=0;i<6;i++)toast.info(`Ordinary ${i}`);
 expect(toasts().filter(item=>!item.persistent)).toHaveLength(TOAST_LIMIT);
 expect(toasts().find(item=>item.id==='job')).toBeDefined();expect(closed).not.toHaveBeenCalled();
 toast.info('Replacement job',{id:'replacement',persistent:true,duration:Infinity});
 expect(toasts().filter(item=>item.persistent)).toHaveLength(1);expect(closed).toHaveBeenCalledWith('evicted');
});
