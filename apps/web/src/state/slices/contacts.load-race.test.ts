// `loadContacts()` when two calls overlap (t31-e4, 26.20).
//
// `loadContacts()` asks for address books, groups and cards together and writes
// all three after they answer. Opening Contacts starts two loads (the surface
// switch and the module mount each call it; Compose and the keys module call it
// too). If the earlier one answers after the later one, it must not put its
// older cards and groups back (one created in between would vanish until the
// next reload), and it must not end the loading state of the later one.
//
// The backend is the contacts mock client (`modules/contacts/mockClient.ts`),
// with chosen `ContactCard/query` answers held: they are computed when asked,
// so a held answer is the one from before any later write, and delivered only
// once the test releases them. Same shape as
// `modules/calendar/controller.load-race.test.ts` (t30-h1).

import { describe, it, expect } from 'vitest';
import { createRoot } from 'solid-js';
import type { Client } from '../../api/client.ts';
import type { JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import { defaultSeed, makeContactsClient } from '../../modules/contacts/mockClient.ts';
import type { SliceContext } from './context.ts';
import { createContactsSlice, type ContactsSlice } from './contacts.ts';

interface Gate {
  reached: Promise<void>;
  reach(): void;
  release(): void;
  released: Promise<void>;
}

function gate(): Gate {
  let reach: () => void = () => undefined;
  let release: () => void = () => undefined;
  const reached = new Promise<void>((r) => {
    reach = r;
  });
  const released = new Promise<void>((r) => {
    release = r;
  });
  return { reached, reach, release, released };
}

/** The seeded mock client; the `ContactCard/query` calls numbered in `hold`
 *  (1-based) wait for their gate's release before answering. */
function harness(hold: number[]): { client: Client; gates: Map<number, Gate> } {
  const inner = makeContactsClient(defaultSeed());
  let queries = 0;
  const gates = new Map(hold.map((n) => [n, gate()] as const));
  const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
    const res = await inner.jmap(body);
    if (body.methodCalls.some(([name]) => name === 'ContactCard/query')) {
      queries += 1;
      const g = gates.get(queries);
      if (g !== undefined) {
        g.reach();
        await g.released;
      }
    }
    return res;
  };
  return { client: { ...inner, jmap }, gates };
}

function withSlice(client: Client, run: (s: ContactsSlice) => Promise<void>): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    createRoot((dispose) => {
      const ctx: SliceContext = { client, showToast: () => undefined };
      run(createContactsSlice(ctx)).then(resolve, reject).finally(dispose);
    });
  });
}

describe('contacts slice — overlapping loadContacts()', () => {
  it('keeps the cards and groups of the load that started last when an earlier one answers after it', async () => {
    const { client, gates } = harness([1]);
    await withSlice(client, async (s) => {
      const first = s.loadContacts();
      await gates.get(1)!.reached;

      // A card and a group written while the first load waits, then a load
      // that runs to the end.
      const cardId = await s.createContact({ name: { full: 'Grace Hopper', given: 'Grace', surname: 'Hopper', prefix: '', suffix: '' } });
      expect(cardId).not.toBeNull();
      const groupId = await s.createGroup('Added while loading');
      expect(groupId).not.toBeNull();
      await s.loadContacts();
      expect(s.contacts().some((c) => c.id === cardId)).toBe(true);
      expect(s.contactGroups().some((g) => g.id === groupId)).toBe(true);

      gates.get(1)!.release();
      await first;

      expect(s.contacts().some((c) => c.id === cardId)).toBe(true);
      expect(s.contactGroups().some((g) => g.id === groupId)).toBe(true);
      expect(s.contacts()).toHaveLength(3);
      expect(s.contactsLoading()).toBe(false);
    });
  });

  it('stays loading until the latest call answers, whichever answers first', async () => {
    const { client, gates } = harness([1, 2]);
    await withSlice(client, async (s) => {
      const first = s.loadContacts();
      await gates.get(1)!.reached;
      const second = s.loadContacts();
      await gates.get(2)!.reached;
      expect(s.contactsLoading()).toBe(true);

      gates.get(1)!.release();
      await first;
      expect(s.contactsLoading()).toBe(true);

      gates.get(2)!.release();
      await second;
      expect(s.contactsLoading()).toBe(false);
      expect(s.contacts().map((c) => c.id).sort()).toEqual(['c1', 'c2']);
    });
  });
});
