import { describe, it, expect, beforeEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createAssistSlice, type AssistSlice } from './assist.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';
import { AssistService } from '../../modules/assist/service.ts';
import type { AssistConfig } from '../../modules/assist/types.ts';

const ctx: SliceContext = { client: {} as Client, showToast: () => undefined };

/** A service double returning a fixed config. */
function serviceWith(config: AssistConfig): AssistService {
  const s = new AssistService();
  (s as unknown as { getConfig: () => Promise<AssistConfig> }).getConfig = async () => config;
  return s;
}

function withSlice(service: AssistService, run: (slice: AssistSlice) => void | Promise<void>): Promise<void> {
  return new Promise((resolve, reject) => {
    createRoot((dispose) => {
      void Promise.resolve(run(createAssistSlice(ctx, service)))
        .then(() => {
          dispose();
          resolve();
        })
        .catch((e) => {
          dispose();
          reject(e instanceof Error ? e : new Error(String(e)));
        });
    });
  });
}

const enabled: AssistConfig = {
  availability: 'enabled',
  capabilities: ['assistant', 'grammar'],
  endpointHost: 'ai.example.com',
  includeE2ee: false,
  includeAttachments: false,
};

describe('assist slice', () => {
  beforeEach(() => localStorage.clear());

  it('defaults to disabled (hide-all) before config loads', async () => {
    await withSlice(serviceWith(enabled), (slice) => {
      expect(slice.enabled()).toBe(false);
      expect(slice.config().availability).toBe('disabled');
      expect(slice.can('assistant')).toBe(false);
    });
  });

  it('reflects the gateway config after loadConfig', async () => {
    await withSlice(serviceWith(enabled), async (slice) => {
      await slice.loadConfig();
      expect(slice.enabled()).toBe(true);
      expect(slice.can('assistant')).toBe(true);
      expect(slice.can('draft')).toBe(false); // not granted
    });
  });

  it('stays disabled when getConfig throws', async () => {
    const bad = new AssistService();
    (bad as unknown as { getConfig: () => Promise<AssistConfig> }).getConfig = async () => {
      throw new Error('network');
    };
    await withSlice(bad, async (slice) => {
      await slice.loadConfig();
      expect(slice.enabled()).toBe(false);
    });
  });

  it('records the per-message disclosure log', async () => {
    await withSlice(serviceWith(enabled), (slice) => {
      slice.recordDisclosure('grammar', { endpointHost: 'ai.example.com', sent: ['message text'], withheld: ['attachments'] });
      expect(slice.disclosureLog().length).toBe(1);
      expect(slice.disclosureLog()[0]?.capability).toBe('grammar');
      expect(slice.disclosureLog()[0]?.disclosure.endpointHost).toBe('ai.example.com');
    });
  });

  it('records the auto-tag audit trail', async () => {
    await withSlice(serviceWith(enabled), (slice) => {
      slice.recordTagAudit({ id: 't1', messageId: 'm1', keyword: 'work', action: 'suggested', actor: 'assist', ts: 'now' });
      slice.recordTagAudit({ id: 't2', messageId: 'm1', keyword: 'work', action: 'applied', actor: 'user', ts: 'now' });
      expect(slice.tagAudit().length).toBe(2);
      expect(slice.tagAudit()[1]?.actor).toBe('user');
    });
  });

  it('persists the opt-in auto-tag mode (default suggest)', async () => {
    await withSlice(serviceWith(enabled), (slice) => {
      expect(slice.autoTagMode()).toBe('suggest');
      slice.setAutoTagMode('auto');
      expect(slice.autoTagMode()).toBe('auto');
      expect(localStorage.getItem('mw.assist.autotag.v1')).toBe('auto');
    });
  });
});
