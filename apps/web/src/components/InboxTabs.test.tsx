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

  it('toggles the unified inbox', () => {
    const { app } = renderWithApp(() => <InboxTabs />);
    const checkbox = screen.getByLabelText('Unified inbox');
    fireEvent.click(checkbox);
    expect(app.unifiedInbox()).toBe(true);
  });
});
