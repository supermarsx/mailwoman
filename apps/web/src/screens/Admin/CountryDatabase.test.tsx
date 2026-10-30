import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { CountryDatabase } from './CountryDatabase.tsx';
import { renderInShell, holding } from './shell/testkit.tsx';
import type { GeoipApi, GeoipStatus } from '../../api/admin/geoip.ts';

const providers: GeoipStatus['providers'] = [
  { id: 'dbip', name: 'DB-IP IP to Country Lite', needsKey: false, homepage: 'https://db-ip.com/db/download/ip-to-country-lite', licence: 'CC BY 4.0', licenceUrl: 'https://creativecommons.org/licenses/by/4.0/', attribution: 'IP Geolocation by DB-IP', attributionUrl: 'https://db-ip.com', updates: 'monthly', checksum: 'structural' },
  { id: 'maxmind', name: 'MaxMind GeoLite2 Country', needsKey: true, homepage: null, licence: 'GeoLite2 End User License Agreement', licenceUrl: 'https://www.maxmind.com/en/geolite2/eula', attribution: 'This product includes GeoLite2 data created by MaxMind, available from https://www.maxmind.com', attributionUrl: 'https://www.maxmind.com', updates: 'twice weekly', checksum: 'sha256' },
  { id: 'ipinfo', name: 'IPinfo Lite', needsKey: true, homepage: null, licence: 'CC BY-SA 4.0', licenceUrl: 'https://creativecommons.org/licenses/by-sa/4.0/', attribution: 'IP address data powered by IPinfo', attributionUrl: 'https://ipinfo.io', updates: 'daily', checksum: 'structural' },
  { id: 'custom', name: 'Custom URL', needsKey: false, homepage: null, licence: null, licenceUrl: null, attribution: null, attributionUrl: null, updates: null, checksum: 'optional sha256' },
];
const config: GeoipStatus['config'] = { provider: 'dbip', licenceKeyConfigured: false, customUrl: '', customChecksumUrl: '', customRedirectHosts: [], customCaConfigured: false, autoUpdate: true, intervalDays: 7 };
const schedule: GeoipStatus['schedule'] = { autoUpdate: true, lastCheckAt: null, lastSuccessAt: null, nextCheckAt: null, lastError: null, failures: 0, waitingForFirstDownload: true };
const missing: GeoipStatus = { managedBy: 'console', state: 'missing', error: null, dataDirectory: '/data/geoip', writable: true, database: null, previous: null, policyActive: true, busy: false, config, providers, schedule, staleAfterDays: 45 };
const loaded: GeoipStatus = { ...missing, state: 'loaded', database: { databaseType: 'DBIP-Country-Lite', buildEpoch: 1793750400, sizeBytes: 7_900_000, sha256: 'a'.repeat(64), networks: 512_345, ipVersion: 6, description: null, source: 'dbip', release: '2026-10', installedAt: 1793800000, installedBy: 'root', checksumVerified: false, stale: false }, schedule: { ...schedule, lastCheckAt: 1793800000, nextCheckAt: 1794404800, waitingForFirstDownload: false } };

function api(initial: GeoipStatus = missing): GeoipApi & { current: GeoipStatus } {
  const t: GeoipApi & { current: GeoipStatus } = {
    current: initial,
    read: vi.fn(async () => ({ data: t.current, etag: '"e1"' })),
    save: vi.fn(async (c) => { t.current = { ...t.current, config: { ...t.current.config, provider: c.provider, licenceKeyConfigured: !!c.licenceKey || (t.current.config.licenceKeyConfigured && !c.clearLicenceKey), autoUpdate: c.autoUpdate, intervalDays: c.intervalDays } }; return { data: t.current, etag: '"e2"' }; }),
    download: vi.fn(async () => { t.current = loaded; return { ...loaded, outcome: { result: 'installed' } }; }),
    check: vi.fn(async () => ({ ...t.current, outcome: { result: 'up-to-date' } })),
    rollback: vi.fn(async () => t.current),
    upload: vi.fn(async () => t.current),
    lookup: vi.fn(async (ip: string) => ({ ip, database: t.current.state === 'loaded' ? 'ready' as const : 'unavailable' as const, country: t.current.state === 'loaded' ? 'PT' : null })),
  };
  return t;
}

describe('country database page', () => {
  it('shows the fail-closed warning while missing, downloads in place, and looks up an address', async () => {
    const transport = api();
    renderInShell(() => <CountryDatabase api={transport} />);
    expect(await screen.findByText(/An active country policy refuses sign-in in this state/)).toBeInTheDocument();
    const table = screen.getByRole('table', { name: 'Country database status' });
    expect(table).toHaveTextContent('Missing');
    const section = screen.getByRole('region', { name: 'Country database' });
    fireEvent.click(screen.getByRole('button', { name: 'Download now' }));
    await waitFor(() => expect(transport.download).toHaveBeenCalledTimes(1));
    expect(await screen.findByText('The new database is installed and in use.')).toBeInTheDocument();
    // The page stayed mounted: same section node, now showing the loaded state.
    expect(screen.getByRole('region', { name: 'Country database' })).toBe(section);
    expect(table).toHaveTextContent('Loaded');
    expect(table).toHaveTextContent('DB-IP IP to Country Lite (2026-10)');
    expect(table).toHaveTextContent('512,345');
    expect(screen.queryByText(/An active country policy refuses sign-in/)).not.toBeInTheDocument();
    // CC BY 4.0 attribution for the data in use.
    const attribution = screen.getByTestId('geoip-attribution');
    expect(attribution).toHaveTextContent('IP Geolocation by DB-IP');
    expect(within(attribution).getByRole('link', { name: 'IP Geolocation by DB-IP' })).toHaveAttribute('href', 'https://db-ip.com');
    expect(within(attribution).getByRole('link', { name: 'CC BY 4.0' })).toHaveAttribute('href', 'https://creativecommons.org/licenses/by/4.0/');
    fireEvent.input(screen.getByLabelText('IP address'), { target: { value: '89.114.7.1' } });
    fireEvent.click(screen.getByRole('button', { name: 'Look up' }));
    expect(await screen.findByTestId('geoip-lookup-result')).toHaveTextContent('89.114.7.1 is in PT.');
  });

  it('sends the licence key write-only and never shows it back', async () => {
    const transport = api(loaded);
    renderInShell(() => <CountryDatabase api={transport} />);
    const provider = await screen.findByLabelText('Provider');
    await waitFor(() => expect(provider).toBeEnabled());
    fireEvent.change(provider, { target: { value: 'maxmind' } });
    expect(screen.getByTestId('geoip-provider-licence')).toHaveTextContent('GeoLite2 End User License Agreement');
    const key = screen.getByLabelText(/^MaxMind licence key/);
    expect(key).toHaveAttribute('type', 'password');
    fireEvent.input(key, { target: { value: 'secret-key-123' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save source' }));
    await waitFor(() => expect(transport.save).toHaveBeenCalledWith(expect.objectContaining({ provider: 'maxmind', licenceKey: 'secret-key-123', autoUpdate: true, intervalDays: 7 }), '"e1"'));
    await waitFor(() => expect(key).toHaveValue(''));
    expect(document.body.textContent).not.toContain('secret-key-123');
    expect(screen.getByText(/A key is stored. It is never shown/)).toBeInTheDocument();
  });

  it('is read-only when MW_GEOIP_DB manages the database', async () => {
    const transport = api({ ...loaded, managedBy: 'environment', writable: false, database: { ...loaded.database!, source: 'environment' } });
    renderInShell(() => <CountryDatabase api={transport} />);
    expect(await screen.findByText(/MW_GEOIP_DB is set on the server/)).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Download now' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Upload .mmdb' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Look up' })).toBeInTheDocument();
  });

  it('disables actions without security.write', async () => {
    renderInShell(() => <CountryDatabase api={api(loaded)} />, { loadMe: holding(['security.read']) });
    const download = await screen.findByRole('button', { name: 'Download now' });
    await waitFor(() => expect(screen.getByRole('table', { name: 'Country database status' })).toHaveTextContent('Loaded'));
    expect(download).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Roll back' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Look up' })).toBeInTheDocument();
  });

  it('reports an up-to-date check', async () => {
    const transport = api(loaded);
    renderInShell(() => <CountryDatabase api={transport} />);
    const check = await screen.findByRole('button', { name: 'Check for updates' });
    await waitFor(() => expect(check).toBeEnabled());
    fireEvent.click(check);
    expect(await screen.findByText("The installed database is the provider's current release.")).toBeInTheDocument();
    expect(transport.check).toHaveBeenCalledTimes(1);
  });

  it('shows the stale warning and the previous copy', async () => {
    renderInShell(() => <CountryDatabase api={api({ ...loaded, database: { ...loaded.database!, stale: true }, previous: { source: 'dbip', buildEpoch: 1791158400 } })} />);
    expect(await screen.findByText(/built more than 45 days ago/)).toBeInTheDocument();
    expect(screen.getByText(/Previous copy kept for rollback: DB-IP IP to Country Lite/)).toBeInTheDocument();
  });
});
