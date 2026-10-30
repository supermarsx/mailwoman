// t111: the delayed busy toast for background loading (delay threshold, dedupe,
// completion and failure transitions), against the real toast store and region.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@solidjs/testing-library';
import { ToastRegion, resetToasts, toast, toasts } from './toast/index.ts';
import { BACKGROUND_TOAST_DELAY_MS, createBackgroundLoadToast } from './backgroundLoadToast.ts';

beforeEach(() => { vi.useFakeTimers(); resetToasts(); });
afterEach(() => { cleanup(); resetToasts(); vi.useRealTimers(); });

const make = () => createBackgroundLoadToast({ id: 'list-load', testId: 'mail-loading-toast' });

describe('background load toast', () => {
  it('waits 400 ms: a load that ends sooner shows nothing', () => {
    expect(BACKGROUND_TOAST_DELAY_MS).toBe(400);
    const c = make();
    c.start('Loading messages…');
    vi.advanceTimersByTime(399);
    expect(toasts()).toHaveLength(0);
    c.finish();
    vi.advanceTimersByTime(5_000);
    expect(toasts()).toHaveLength(0);
  });

  it('shows one busy info toast after the delay and closes it on completion', () => {
    const c = make();
    c.start('Loading messages…');
    vi.advanceTimersByTime(400);
    expect(toasts()).toHaveLength(1);
    expect(toasts()[0]).toMatchObject({ id: 'list-load', tone: 'info', busy: true, message: 'Loading messages…', duration: Infinity });
    expect(c.showing()).toBe(true);
    c.finish();
    expect(toasts()).toHaveLength(0);
    expect(c.showing()).toBe(false);
  });

  it('never stacks: a repeat start is a no-op and a new message replaces the text in place', () => {
    const c = make();
    c.start('Loading messages…');
    vi.advanceTimersByTime(200);
    c.start('Loading messages…'); // does not restart the delay
    vi.advanceTimersByTime(200);
    expect(toasts()).toHaveLength(1);
    c.start('Loading older messages…');
    expect(toasts()).toHaveLength(1);
    expect(toasts()[0]!.message).toBe('Loading older messages…');
    // A second controller-less call with the same id (another list) also replaces it.
    toast.info('Loading messages…', { id: 'list-load', busy: true, duration: Infinity });
    expect(toasts()).toHaveLength(1);
  });

  it('a switch of text inside the delay keeps the original clock', () => {
    const c = make();
    c.start('Loading messages…');
    vi.advanceTimersByTime(300);
    c.start('Loading older messages…');
    vi.advanceTimersByTime(100);
    expect(toasts().map((i) => i.message)).toEqual(['Loading older messages…']);
  });

  it('a failure turns into a short error toast in the same place, even when it came fast', () => {
    const c = make();
    c.start('Loading messages…');
    vi.advanceTimersByTime(500);
    c.finish('Could not load more messages.');
    expect(toasts()).toHaveLength(1);
    expect(toasts()[0]).toMatchObject({ id: 'list-load', tone: 'error', busy: false });
    expect(Number.isFinite(toasts()[0]!.duration)).toBe(true);
    vi.advanceTimersByTime(10_000);
    expect(toasts()).toHaveLength(0);

    c.start('Loading messages…');
    c.finish('Failed fast');
    expect(toasts().map((i) => i.tone)).toEqual(['error']);
  });

  it('closed by the user: stays closed for that load, comes back for the next one', () => {
    const c = make();
    c.start('Loading messages…');
    vi.advanceTimersByTime(400);
    toast.dismiss('list-load');
    c.start('Loading older messages…');
    expect(toasts()).toHaveLength(0);
    c.finish();
    c.start('Loading messages…');
    vi.advanceTimersByTime(400);
    expect(toasts()).toHaveLength(1);
  });

  it('cancel drops a pending or shown toast', () => {
    const c = make();
    c.start('Loading messages…');
    c.cancel();
    vi.advanceTimersByTime(1_000);
    expect(toasts()).toHaveLength(0);
    c.start('Loading messages…');
    vi.advanceTimersByTime(400);
    c.cancel();
    expect(toasts()).toHaveLength(0);
  });

  it('renders in the polite region with a spinner in place of the tone icon', () => {
    render(() => <ToastRegion />);
    const c = make();
    c.start('Loading messages…');
    vi.advanceTimersByTime(400);
    const el = screen.getByTestId('mail-loading-toast');
    expect(el.getAttribute('data-busy')).toBe('true');
    expect(el.querySelector('[data-toast-spinner]')).not.toBeNull();
    expect(el.querySelector('svg path')?.getAttribute('d')).not.toBe('M8 7.2v4.6M8 4.4v.2M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14z');
    expect(el.closest('[aria-live="polite"]')).not.toBeNull();
    expect(el.getAttribute('role')).toBe('status');
    c.finish();
    expect(screen.queryByTestId('mail-loading-toast')).toBeNull();
  });
});
