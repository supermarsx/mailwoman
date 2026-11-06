import { describe, expect, it } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { RestrictionBanner } from './RestrictionBanner.tsx';

describe('RestrictionBanner', () => {
  it('renders nothing without a restriction', async () => {
    render(() => <RestrictionBanner load={async () => ({ active: false, scopes: [], since: null, expiresAt: null })} />);
    await new Promise((r) => setTimeout(r, 30));
    expect(screen.queryByTestId('restriction-banner')).toBeNull();
  });
  it('names what is paused and says mail can still be read', async () => {
    render(() => <RestrictionBanner load={async () => ({ active: true, scopes: ['assistant', 'indexing'], since: 1, expiresAt: null })} />);
    const banner = await screen.findByTestId('restriction-banner');
    await waitFor(() => expect(banner.textContent).toContain('assistant, search indexing'));
    expect(banner.textContent).toContain('still read and delete');
    expect(banner.getAttribute('role')).toBe('status');
  });
  it('stays hidden when the status cannot be loaded', async () => {
    render(() => <RestrictionBanner load={async () => { throw new Error('x'); }} />);
    await new Promise((r) => setTimeout(r, 30));
    expect(screen.queryByTestId('restriction-banner')).toBeNull();
  });
});
