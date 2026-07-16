import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createRulesSlice, emptyRuleDraft, type RulesSlice } from './rules.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';
import { CAP_SECURITY, type MailRule } from '../../api/crypto-types.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_SECURITY]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/jmap/api',
  downloadUrl: '/d',
  uploadUrl: '/u',
  eventSourceUrl: '/e',
  state: 's0',
};

function ruleFixture(id: string, name: string): MailRule {
  return {
    id,
    name,
    matchAll: true,
    conditions: [{ type: 'from', op: 'contains', value: 'x@y' }],
    actions: [{ type: 'move', value: 'A' }],
    enabled: true,
    runsAt: 'engine',
  };
}

/** A fake JMAP client backed by a mutable in-memory rule list. */
function makeClient(seed: MailRule[]): { client: Client; jmap: ReturnType<typeof vi.fn>; store: MailRule[] } {
  const store = [...seed];
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const call = body.methodCalls[0]!;
    const [name, args, callId] = call;
    const a = args as Record<string, unknown>;
    if (name === 'MailRule/get') {
      return { methodResponses: [['MailRule/get', { accountId: 'acct1', state: 's', list: store, notFound: [] }, callId]], sessionState: 's' };
    }
    if (name === 'MailRule/set') {
      const created: Record<string, { id: string }> = {};
      if (a.create) {
        for (const [cid, spec] of Object.entries(a.create as Record<string, MailRule>)) {
          const id = `srv-${store.length + 1}`;
          store.push({ ...spec, id });
          created[cid] = { id };
        }
      }
      if (a.update) {
        for (const [id, patch] of Object.entries(a.update as Record<string, Partial<MailRule>>)) {
          const idx = store.findIndex((r) => r.id === id);
          if (idx >= 0) store[idx] = { ...store[idx]!, ...patch };
        }
      }
      if (a.destroy) {
        for (const id of a.destroy as string[]) {
          const idx = store.findIndex((r) => r.id === id);
          if (idx >= 0) store.splice(idx, 1);
        }
      }
      return { methodResponses: [['MailRule/set', { accountId: 'acct1', created, updated: {}, destroyed: [] }, callId]], sessionState: 's' };
    }
    return { methodResponses: [], sessionState: 's' };
  });
  const client = { session: async () => SESSION, jmap } as unknown as Client;
  return { client, jmap, store };
}

function ctxFor(client: Client): SliceContext {
  return { client, showToast: vi.fn() };
}

describe('rules slice — MailRule JMAP round-trip', () => {
  it('loads rules from MailRule/get', async () => {
    await createRoot(async (dispose) => {
      const { client } = makeClient([ruleFixture('1', 'a'), ruleFixture('2', 'b')]);
      const slice: RulesSlice = createRulesSlice(ctxFor(client));
      await slice.loadRules();
      expect(slice.rules().map((r) => r.name)).toEqual(['a', 'b']);
      dispose();
    });
  });

  it('creates a rule (no id) then reloads the list', async () => {
    await createRoot(async (dispose) => {
      const { client, jmap } = makeClient([]);
      const slice = createRulesSlice(ctxFor(client));
      await slice.saveRule({ ...emptyRuleDraft(), name: 'new one' });
      const setArgs = jmap.mock.calls.map((c) => c[0].methodCalls[0]!).find((m) => m[0] === 'MailRule/set');
      expect(setArgs?.[1]).toHaveProperty('create');
      expect(slice.rules().map((r) => r.name)).toContain('new one');
      dispose();
    });
  });

  it('updates a rule when a draft carries an id', async () => {
    await createRoot(async (dispose) => {
      const { client, jmap } = makeClient([ruleFixture('1', 'old')]);
      const slice = createRulesSlice(ctxFor(client));
      await slice.loadRules();
      await slice.saveRule({ ...ruleFixture('1', 'renamed'), id: '1' });
      const setArgs = jmap.mock.calls.map((c) => c[0].methodCalls[0]!).find((m) => m[0] === 'MailRule/set');
      expect(setArgs?.[1]).toHaveProperty('update');
      expect(slice.rules()[0]!.name).toBe('renamed');
      dispose();
    });
  });

  it('deletes a rule and drops it locally', async () => {
    await createRoot(async (dispose) => {
      const { client } = makeClient([ruleFixture('1', 'a'), ruleFixture('2', 'b')]);
      const slice = createRulesSlice(ctxFor(client));
      await slice.loadRules();
      await slice.deleteRule('1');
      expect(slice.rules().map((r) => r.id)).toEqual(['2']);
      dispose();
    });
  });

  it('toggles enabled optimistically', async () => {
    await createRoot(async (dispose) => {
      const { client } = makeClient([ruleFixture('1', 'a')]);
      const slice = createRulesSlice(ctxFor(client));
      await slice.loadRules();
      await slice.toggleRule('1', false);
      expect(slice.rules()[0]!.enabled).toBe(false);
      dispose();
    });
  });
});
