import { describe, it, expect, beforeEach } from 'vitest';
import { screen, fireEvent } from '@solidjs/testing-library';
import { InboxTabs } from './InboxTabs.tsx';
import { renderWithApp } from './appHarness.tsx';

describe('InboxTabs', () => {
  beforeEach(() => localStorage.clear());

  it('is opt-in: shows an enable button, not tabs, by default', () => {
    renderWithApp(() => <InboxTabs />);
    expect(screen.getByRole('button', { name: 'Focused inbox' })).toBeInTheDocument();
    expect(screen.queryByRole('tab')).toBeNull();
  });

  it('reveals Focused/Other tabs when enabled and switches the active tab', () => {
    const { app } = renderWithApp(() => <InboxTabs />);
    fireEvent.click(screen.getByRole('button', { name: 'Focused inbox' }));

    const focused = screen.getByRole('tab', { name: /Focused/ });
    const other = screen.getByRole('tab', { name: /Other/ });
    expect(focused).toHaveAttribute('aria-selected', 'true');

    fireEvent.click(other);
    expect(app.inboxTab()).toBe('other');
    expect(other).toHaveAttribute('aria-selected', 'true');
  });

  it('offers no unified-inbox control, in either mode, and the store has no such flag', () => {
    const { app } = renderWithApp(() => <InboxTabs />);
    expect(screen.queryByRole('checkbox')).toBeNull();
    expect(screen.queryByText(/unified/i)).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Focused inbox' }));
    // Precondition for the second half: the focused tabs really are showing.
    expect(screen.getAllByRole('tab')).toHaveLength(2);
    expect(screen.queryByRole('checkbox')).toBeNull();
    expect(screen.queryByText(/unified/i)).toBeNull();
    expect('unifiedInbox' in app).toBe(false);
    expect('setUnifiedInbox' in app).toBe(false);
  });
});
