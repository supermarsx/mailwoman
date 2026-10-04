import { describe, it, expect } from 'vitest';
import { screen } from '@solidjs/testing-library';
import { Appearance } from './Appearance.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';

// The brand / theme / accent form saved to the server's memory only and was reset
// by a restart; it was removed (26.20, t28-e8). These tests fail if it comes back.
describe('Admin › Appearance', () => {
  it('has no form control of any kind', () => {
    const { container } = renderWithAdmin(() => <Appearance />);
    expect(container.querySelectorAll('input, textarea, select, button, form')).toHaveLength(0);
    for (const label of ['Brand name', 'Default theme', 'Accent (hex, optional)', 'Save appearance']) {
      expect(screen.queryByText(label)).toBeNull();
    }
  });

  it('neither reads nor writes the deployment appearance', () => {
    const { api } = renderWithAdmin(() => <Appearance />, mockAdminApi());
    expect(api.getAppearance).not.toHaveBeenCalled();
    expect(api.setAppearance).not.toHaveBeenCalled();
  });

  it('says why, and that per-user appearance is unaffected', () => {
    renderWithAdmin(() => <Appearance />);
    const region = screen.getByRole('region', { name: 'Appearance' });
    expect(region).toHaveTextContent('reset at every restart');
    expect(region).toHaveTextContent("Each user's own appearance");
  });
});
