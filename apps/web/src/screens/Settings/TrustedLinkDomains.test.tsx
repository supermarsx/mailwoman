import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { TrustedLinkDomains } from './TrustedLinkDomains.tsx';
import { clearSettings } from '../../api/settings.ts';

const entry = (value: unknown, locked = false) => ({ value, source: 'global', locked, userValue: value });
let writes: { url: string; value: unknown }[]; let held = false; let known = true;
beforeEach(() => {
  clearSettings(); writes = []; held = false; known = true;
  vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
    const body = init?.method === 'PUT' ? JSON.parse(String(init.body)) as { value: unknown } : null;
    if (body) { writes.push({ url, value: body.value }); return new Response(JSON.stringify(entry(body.value))); }
    return new Response(JSON.stringify({ settings: known ? { 'links.trustedDomains': entry(['corp.test'], held), 'links.trustedSubdomains': entry(false, held) } : {} }));
  }));
});

describe('Trusted link domains settings', () => {
  it('saves a normalized, de-duplicated list and the subdomain switch', async () => {
    render(() => <TrustedLinkDomains />);
    const box = await screen.findByLabelText('Domains'); await waitFor(() => expect(box).toBeEnabled());
    expect((box as HTMLTextAreaElement).value).toBe('corp.test');
    fireEvent.input(box, { target: { value: 'corp.test\nPartner.Test.\npartner.test' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save trusted domains' }));
    await waitFor(() => expect(writes).toContainEqual({ url: '/api/settings/links.trustedDomains', value: ['corp.test', 'partner.test'] }));
    fireEvent.click(screen.getByLabelText(/Also trust subdomains/));
    await waitFor(() => expect(writes).toContainEqual({ url: '/api/settings/links.trustedSubdomains', value: true }));
  });

  it('refuses wildcards, single labels, schemes and IP addresses without writing', async () => {
    render(() => <TrustedLinkDomains />);
    const box = await screen.findByLabelText('Domains'); await waitFor(() => expect(box).toBeEnabled());
    fireEvent.input(box, { target: { value: '*.corp.test\ncom\nhttps://a.test\n10.0.0.1\nok.test' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save trusted domains' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('*.corp.test, com, https://a.test');
    expect(box).toHaveAttribute('aria-invalid', 'true');
    expect(writes).toEqual([]);
  });

  it('is read-only under an administrator lock and disabled against a server without the keys', async () => {
    held = true; const { unmount } = render(() => <TrustedLinkDomains />);
    await waitFor(() => expect(screen.getAllByTestId('setting-lock-note')).toHaveLength(2));
    expect(screen.getByLabelText('Domains')).toBeDisabled();
    expect(screen.getByLabelText(/Also trust subdomains/)).toBeDisabled();
    unmount(); clearSettings(); known = false;
    render(() => <TrustedLinkDomains />);
    await waitFor(() => expect(vi.mocked(fetch)).toHaveBeenCalled());
    expect(screen.getByLabelText('Domains')).toBeDisabled();
    expect(writes).toEqual([]);
  });
});
