// The shared busy indicator: marked so the global reduced-motion rule skips it,
// decorative to assistive tech, and the same node survives a toast's text update
// (a replaced node would restart the animation on every tick).
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { cleanup, render, screen } from '@solidjs/testing-library';
import { ToastRegion, resetToasts, toast } from '../toast/index.ts';
import { Spinner } from './Spinner.tsx';
import * as css from './spinner.css.ts';

beforeEach(() => resetToasts());
afterEach(() => { cleanup(); resetToasts(); });

describe('Spinner', () => {
  it('is a hidden, marked ring carrying the shared class', () => {
    const { container } = render(() => <Spinner />);
    const el = container.querySelector('[data-mw-spinner]');
    expect(el).not.toBeNull();
    expect(el?.classList.contains(css.spinner)).toBe(true);
    expect(el?.getAttribute('aria-hidden')).toBe('true');
  });

  it('with a label exposes a status named for the work', () => {
    render(() => <Spinner label="Loading" />);
    expect(screen.getByRole('status').textContent).toBe('Loading');
  });

  it('keeps the same node while a busy toast updates its text', () => {
    render(() => <ToastRegion />);
    toast.info('Exporting 1 of 3', { id: 'exp', testId: 'exp', busy: true, duration: Infinity });
    const first = screen.getByTestId('exp').querySelector('[data-mw-spinner]');
    expect(first).not.toBeNull();
    toast.info('Exporting 2 of 3', { id: 'exp', testId: 'exp', busy: true, duration: Infinity });
    expect(screen.getByTestId('exp').textContent).toContain('2 of 3');
    expect(screen.getByTestId('exp').querySelector('[data-mw-spinner]')).toBe(first);
  });
});
