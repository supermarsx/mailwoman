import { beforeAll, expect, it } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { loadCatalog } from '../../../i18n';
import { HttpLimitsReadout } from './HttpLimitsReadout.tsx';

beforeAll(async () => {
  await loadCatalog('admin-settings');
});

it('shows what each limit resolved to, where it came from, and the readings Auto used', async () => {
  render(() => (
    <HttpLimitsReadout
      load={async () => ({
        checks: [
          {
            id: 'httpAdmission',
            value: {
              admission: {
                effective: {
                  connections: { value: 32768, source: 'auto' },
                  requests: { value: 512, source: 'console', capped: true },
                  http2Streams: { value: 64, source: 'auto' },
                  queueWaitMs: { value: 200, source: 'explicit' },
                  queueMax: { value: 512, source: 'auto' },
                  adaptive: { value: false, source: 'default' },
                  adaptiveFloor: { value: 256, source: 'auto' },
                },
                inputs: 'from cpus C 16, memory M bytes unknown (term dropped)',
              },
            },
          },
        ],
      })}
    />
  ));
  expect(await screen.findByTestId('http-limits-connections')).toHaveTextContent('Maximum connections: 32,768 (Auto)');
  expect(screen.getByTestId('http-limits-requests')).toHaveTextContent('(set here)');
  expect(screen.getByTestId('http-limits-requests')).toHaveTextContent('Held to the safety bound');
  expect(screen.getByTestId('http-limits-queueWaitMs')).toHaveTextContent('set by the environment');
  expect(screen.getByTestId('http-limits-adaptive')).toHaveTextContent('off');
  expect(screen.getByTestId('http-limits-inputs')).toHaveTextContent('term dropped');
});

it('says the values are not available when the listener reports none', async () => {
  render(() => <HttpLimitsReadout load={async () => ({ checks: [{ id: 'httpAdmission', value: { admission: null } }] })} />);
  expect(await screen.findByText(/shown when the server's own HTTP listener is running/)).toBeVisible();
});
