import { beforeAll, expect, it } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { loadCatalog } from '../../../i18n';
import { ImageProxyReadout } from './ImageProxyReadout.tsx';

beforeAll(async () => {
  await loadCatalog('admin-settings');
});

it('shows configured against effective values and the per-tier counts, and invents nothing when absent', async () => {
  render(() => (
    <ImageProxyReadout
      load={async () => ({
        imageCache: {
          tiers: {
            configured: { ramBytes: 33554432, disk: { enabled: true, mode: 'auto', floorBytes: 67108864, ceilingBytes: 4294967296, manualBytes: 536870912 } },
            ram: { usedBytes: 1048576, budgetBytes: 33554432, entries: 12, hot: 3, cold: 4, hitPercent: 71 },
            disk: { enabled: true, active: true, bytes: 2097152, budgetBytes: 268435456, entries: 40, hot: 5, cold: 6, hitPercent: 9 },
            index: { entries: 52 },
            promotionsPerMin: 2,
            promotions: 20,
            demotions: 11,
            drops: 3,
          },
        },
      })}
    />
  ));
  expect(await screen.findByTestId('image-proxy-ram')).toHaveTextContent('12 images, 3 hot, 4 cold, hit rate 71 %');
  expect(screen.getByTestId('image-proxy-disk')).toHaveTextContent('40 images, 5 hot, 6 cold, hit rate 9 %');
  expect(screen.getByTestId('image-proxy-disk')).toHaveTextContent('(auto;');
  expect(screen.getByTestId('image-proxy-moves')).toHaveTextContent('Promotions per minute: 2');
});

it('says the readout is unavailable when the server reports no tiers, and shows why a disk tier is inactive', async () => {
  const { unmount } = render(() => <ImageProxyReadout load={async () => ({})} />);
  expect(await screen.findByText('The image cache readout is not available.')).toBeVisible();
  unmount();
  render(() => (
    <ImageProxyReadout
      load={async () => ({
        imageCache: { tiers: { ram: {}, disk: { enabled: true, active: false, lastError: 'disk write failed: no space left on device' } } },
      })}
    />
  ));
  expect(await screen.findByTestId('image-proxy-disk')).toHaveTextContent('on but not active. disk write failed');
});
