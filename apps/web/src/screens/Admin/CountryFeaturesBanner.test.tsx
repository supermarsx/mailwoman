import { beforeAll, describe, expect, it } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { loadCatalog } from '../../i18n';
import { CountryFeaturesBanner } from './CountryFeaturesBanner.tsx';
import type { CountryFeatures, GeoipApi, GeoipStatus } from '../../api/admin/geoip.ts';

beforeAll(async () => { await loadCatalog('admin-geoip'); });

function api(features: Partial<CountryFeatures>): GeoipApi {
  const countryFeatures: CountryFeatures = {
    state: 'missing', error: null, loaded: false, buildEpoch: null, ageDays: null, stale: false,
    features: [], warning: false, dependentFeatures: [], ...features,
  };
  return { read: async () => ({ data: { countryFeatures } as unknown as GeoipStatus, etag: '' }) } as unknown as GeoipApi;
}

describe('CountryFeaturesBanner', () => {
  it('warns, names the features and links to the database when one is on and none is loaded', async () => {
    render(() => <CountryFeaturesBanner api={api({ warning: true, dependentFeatures: ['accountGeography', 'adminDiscovery'] })} showStatus />);
    const warning = await screen.findByTestId('country-features-warning');
    expect(warning).toHaveTextContent('no country database is loaded');
    expect(warning).toHaveTextContent('Per-account sign-in country policy');
    expect(warning).toHaveTextContent('Unknown-country admin console block');
    expect(screen.getByRole('link', { name: 'Go to Country database' })).toBeInTheDocument();
    expect(screen.getByTestId('country-database-status')).toHaveTextContent('No country database is loaded.');
  });
  it('stays silent when the database is loaded, and shows its build date and age', async () => {
    render(() => <CountryFeaturesBanner api={api({ loaded: true, state: 'loaded', buildEpoch: 1_700_000_000, ageDays: 12 })} showStatus />);
    const status = await screen.findByTestId('country-database-status');
    expect(status).toHaveTextContent('12 days ago');
    expect(screen.queryByTestId('country-features-warning')).toBeNull();
  });
});
