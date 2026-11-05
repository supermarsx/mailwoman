import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { ComposeStatusBar } from './ComposeStatusBar.tsx';
import type { StatusItem } from './statusBar.ts';

const make = (over: Partial<StatusItem> & Pick<StatusItem, 'id'>): StatusItem => ({ severity: 'neutral', icon: 'info', label: over.id, ...over });

describe('ComposeStatusBar', () => {
  it('is a polite status region that is present with no items', () => {
    render(() => <ComposeStatusBar items={() => []} />);
    const bar = screen.getByRole('status', { name: 'Composer status' });
    expect(bar).toHaveAttribute('aria-live', 'polite');
    expect(bar.querySelectorAll('[data-status-id]')).toHaveLength(0);
  });

  it('shows the most urgent first and carries severity as text and icon, not only colour', () => {
    render(() => <ComposeStatusBar items={() => [
      make({ id: 'saved', severity: 'success', label: 'Saved to account Drafts · 14:32', icon: 'check' }),
      make({ id: 'bad', severity: 'danger', label: 'Invalid address: x', icon: 'alert' }),
    ]} />);
    const rows = Array.from(document.querySelectorAll('[data-status-id]'));
    expect(rows.map(r => r.getAttribute('data-status-id'))).toEqual(['bad', 'saved']);
    expect(rows[0]).toHaveAttribute('data-severity', 'danger');
    expect(rows[0]!.querySelector('svg')).not.toBeNull();
    expect(rows[0]).toHaveTextContent('Invalid address: x');
  });

  it('announces only errors assertively and exposes the detail text to assistive technology', () => {
    render(() => <ComposeStatusBar items={() => [
      make({ id: 'err', severity: 'danger', label: 'Could not send', assertive: true }),
      make({ id: 'id', label: 'From default identity', detail: 'none matched the original recipients' }),
    ]} />);
    expect(screen.getAllByRole('alert')).toHaveLength(1);
    expect(screen.getByRole('alert')).toHaveTextContent('Could not send');
    expect(screen.getByTestId('compose-status-id')).toHaveTextContent('none matched the original recipients');
  });

  it('makes an item with onClick a button and an action a second button', () => {
    const jump = vi.fn();
    const retry = vi.fn();
    render(() => <ComposeStatusBar items={() => [make({ id: 'subject', severity: 'warning', label: 'No subject', onClick: jump, actionLabel: 'Retry', onAction: retry })]} />);
    fireEvent.click(screen.getByRole('button', { name: 'No subject' }));
    fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
    expect(jump).toHaveBeenCalledOnce();
    expect(retry).toHaveBeenCalledOnce();
  });

  it('folds overflow behind a count chip that opens a popover, and closes it on Esc', () => {
    render(() => <ComposeStatusBar width={300} items={() => [
      make({ id: 'a', severity: 'danger', label: 'A' }),
      make({ id: 'b', severity: 'warning', label: 'B' }),
      make({ id: 'c', severity: 'neutral', label: 'C' }),
    ]} />);
    const chip = screen.getByTestId('compose-status-more');
    expect(chip).toHaveTextContent('+2');
    expect(chip).toHaveAttribute('aria-expanded', 'false');
    const pop = screen.getByTestId('compose-status-pop');
    expect(pop).toHaveAttribute('hidden');
    fireEvent.click(chip);
    expect(chip).toHaveAttribute('aria-expanded', 'true');
    expect(pop).not.toHaveAttribute('hidden');
    expect(within(pop).getByTestId('compose-status-b')).toBeInTheDocument();
    fireEvent.keyDown(pop, { key: 'Escape' });
    expect(pop).toHaveAttribute('hidden');
  });

  it('keeps the same bar element as items come and go, so nothing remounts and nothing moves', () => {
    const [items, setItems] = createSignal<StatusItem[]>([]);
    render(() => <ComposeStatusBar items={items} />);
    const bar = screen.getByTestId('compose-status');
    setItems([make({ id: 'x', severity: 'danger', label: 'X' })]);
    expect(screen.getByTestId('compose-status')).toBe(bar);
    setItems([]);
    expect(screen.getByTestId('compose-status')).toBe(bar);
  });

  it('spins the icon of a busy item', () => {
    render(() => <ComposeStatusBar items={() => [make({ id: 'saving', severity: 'busy', label: 'Saving…', icon: 'refresh' })]} />);
    expect(document.querySelector('.compose-status__icon--spin')).not.toBeNull();
  });
});
