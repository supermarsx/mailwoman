import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { Appearance } from './Appearance.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';

describe('Admin › Appearance', () => {
  it('loads the brand + theme', async () => {
    renderWithAdmin(
      () => <Appearance />,
      mockAdminApi({ getAppearance: vi.fn(async () => ({ theme: 'grove-dark', brandName: 'Acme Mail', accent: '#123456' })) }),
    );
    const brand = (await screen.findByText('Brand name')).parentElement!.querySelector('input') as HTMLInputElement;
    expect(brand.value).toBe('Acme Mail');
  });

  it('saves an edited appearance', async () => {
    const setAppearance = vi.fn(async () => undefined);
    renderWithAdmin(() => <Appearance />, mockAdminApi({ setAppearance }));
    const brand = (await screen.findByText('Brand name')).parentElement!.querySelector('input')!;
    fireEvent.input(brand, { target: { value: 'Vogue Mail' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save appearance' }));
    await Promise.resolve();
    expect(setAppearance).toHaveBeenCalledWith(expect.objectContaining({ brandName: 'Vogue Mail' }));
  });
});
