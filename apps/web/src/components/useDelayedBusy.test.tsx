import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createRoot, createSignal } from 'solid-js';
import { render, screen } from '@solidjs/testing-library';
import { useDelayedBusy } from './useDelayedBusy.ts';
import { InlineLoading } from './InlineLoading.tsx';

beforeEach(() => { vi.useFakeTimers(); });
afterEach(() => { vi.useRealTimers(); });

function harness() {
  let set!: (v: boolean) => void; let shown!: () => boolean; let dispose!: () => void;
  createRoot((d) => { dispose = d; const [a, setA] = createSignal(false); set = setA; shown = useDelayedBusy(a, 150, 300); });
  return { set, shown, dispose };
}

describe('useDelayedBusy', () => {
  it('shows nothing for work that ends within the delay', () => {
    const h = harness();
    h.set(true); vi.advanceTimersByTime(100); expect(h.shown()).toBe(false);
    h.set(false); vi.advanceTimersByTime(1000); expect(h.shown()).toBe(false);
    h.dispose();
  });
  it('turns on after the delay and stays on for the minimum display time', () => {
    const h = harness();
    h.set(true); vi.advanceTimersByTime(151); expect(h.shown()).toBe(true);
    vi.advanceTimersByTime(50); h.set(false); expect(h.shown()).toBe(true);
    vi.advanceTimersByTime(240); expect(h.shown()).toBe(true);
    vi.advanceTimersByTime(20); expect(h.shown()).toBe(false);
    h.dispose();
  });
  it('turns off at once when the minimum has already elapsed', () => {
    const h = harness();
    h.set(true); vi.advanceTimersByTime(600); expect(h.shown()).toBe(true);
    h.set(false); expect(h.shown()).toBe(false);
    h.dispose();
  });
  it('keeps showing when work restarts during the minimum window', () => {
    const h = harness();
    h.set(true); vi.advanceTimersByTime(200); h.set(false); h.set(true); vi.advanceTimersByTime(1000);
    expect(h.shown()).toBe(true);
    h.dispose();
  });
});

describe('InlineLoading', () => {
  it('is a polite status that announces at once and draws the spinner after the delay', () => {
    render(() => <InlineLoading message="Sanitizing…" />);
    const status = screen.getByRole('status');
    expect(status).toHaveAttribute('aria-live', 'polite');
    expect(status).toHaveTextContent('Sanitizing…');
    expect(status.querySelector('[data-mw-spinner]')).toBeNull();
    vi.advanceTimersByTime(160);
    expect(status.querySelector('[data-mw-spinner]')).not.toBeNull();
    expect(status.getAttribute('data-visible')).toBe('true');
  });
  it('is never a cover: no overlay element, no pointer blocking', () => {
    render(() => <InlineLoading />);
    const status = screen.getByTestId('inline-loading');
    expect(status.style.position).toBe('');
    expect(status.tagName).toBe('SPAN');
  });
});

describe('MessageLoading (Loading…)', () => {
  it('draws the default skeleton after the configured delay and honors indicator/none choices', async () => {
    const { MessageLoading } = await import('./MessageLoading.tsx');
    const { container,unmount } = render(() => <MessageLoading variant="skeleton" delayMs={150} />);
    expect(container.querySelectorAll('[role="status"]')).toHaveLength(1);
    expect(container.textContent).toContain('Loading');expect(container.querySelector('[data-skeleton="viewer"]')).toBeNull();
    vi.advanceTimersByTime(160);expect(container.querySelector('[data-skeleton="viewer"]')).not.toBeNull();expect(container.querySelector('[data-mw-spinner]')).toBeNull();unmount();
    const indicator=render(()=><MessageLoading variant="indicator" delayMs={0}/>);expect(indicator.container.querySelector('[data-mw-spinner]')).not.toBeNull();indicator.unmount();
    const quiet=render(()=><MessageLoading variant="none" delayMs={0}/>);vi.advanceTimersByTime(1000);expect(quiet.container.querySelector('[data-skeleton]')).toBeNull();expect(quiet.container.querySelector('[data-mw-spinner]')).toBeNull();
  });
});
