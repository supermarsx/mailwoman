import { describe, it, expect, beforeEach } from 'vitest';
import { screen, fireEvent } from '@solidjs/testing-library';
import { Ribbon } from './Ribbon.tsx';
import { renderWithApp } from './appHarness.tsx';

// The ribbon is the named WCAG 2.2 target (t8-e1): a WAI-ARIA tablist with
// roving tabindex over Home / View / Folder, each controlling the panel.
describe('Ribbon', () => {
  beforeEach(() => localStorage.clear());

  it('exposes a tablist with the three ribbon tabs', () => {
    renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} />);
    const tablist = screen.getByRole('tablist', { name: 'Ribbon' });
    expect(tablist).toBeInTheDocument();
    expect(screen.getAllByRole('tab')).toHaveLength(3);
    expect(screen.getByRole('tab', { name: 'Home' })).toHaveAttribute('aria-selected', 'true');
  });

  it('uses roving tabindex: only the selected tab is in the tab order', () => {
    renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} />);
    const home = screen.getByRole('tab', { name: 'Home' });
    const view = screen.getByRole('tab', { name: 'View' });
    expect(home).toHaveAttribute('tabindex', '0');
    expect(view).toHaveAttribute('tabindex', '-1');
  });

  it('ArrowRight moves selection (activation follows focus) and shows that panel', () => {
    renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} />);
    const tablist = screen.getByRole('tablist', { name: 'Ribbon' });
    fireEvent.keyDown(tablist, { key: 'ArrowRight' });

    const view = screen.getByRole('tab', { name: 'View' });
    expect(view).toHaveAttribute('aria-selected', 'true');
    expect(view).toHaveAttribute('tabindex', '0');
    // The View panel (theme/density/settings groups) is now shown.
    expect(screen.getByRole('tabpanel')).toHaveAttribute('aria-labelledby', 'ribbon-tab-view');
  });

  it('the collapse toggle reflects its expanded state', () => {
    const { app } = renderWithApp(() => <Ribbon onCompose={() => undefined} onOpenSettings={() => undefined} />);
    const toggle = screen.getByRole('button', { name: /Collapse|Expand/ });
    expect(toggle).toHaveAttribute('aria-expanded', String(!app.ribbonCollapsed()));
    fireEvent.click(toggle);
    expect(app.ribbonCollapsed()).toBe(true);
  });
});
