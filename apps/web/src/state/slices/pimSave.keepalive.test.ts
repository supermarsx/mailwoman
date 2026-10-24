// Note and task saves ask the transport to outlive the page (t32-h11, 26.20).
//
// Each create / update / destroy a user makes in Notes or Tasks is sent with
// `keepalive`, so a save started just before the tab closes or reloads still
// reaches the server (see `api/client.keepalive.test.ts` for the transport side).
// Loads are not: nothing is lost if a read is cut off.

import { describe, it, expect } from 'vitest';
import { createRoot } from 'solid-js';
import type { Client } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import { CAP_NOTES } from '../../api/pim-types.ts';
import type { SliceContext } from './context.ts';
import { createNotesSlice } from './notes.ts';
import { createTasksSlice } from './tasks.ts';

interface Call {
  method: string;
  keepalive: boolean;
}

/** A client that records each call's first method and whether it asked for keepalive. */
function recordingClient(calls: Call[]): Client {
  const jmap = async (body: JmapRequest, opts?: { keepalive?: boolean }): Promise<JmapResponse> => {
    const method = body.methodCalls[0]?.[0] ?? '';
    calls.push({ method, keepalive: opts?.keepalive === true });
    const tag = body.methodCalls[0]?.[2] ?? '';
    if (method === 'Note/set') {
      return { methodResponses: [['Note/set', { accountId: 'a', created: { new: { id: 'n-new' } } }, tag]], sessionState: 's' };
    }
    if (method === 'Task/set') {
      return {
        methodResponses: [['Task/set', { accountId: 'a', created: { new: { id: 't-new', uid: 't-new' } }, notCreated: null }, tag]],
        sessionState: 's',
      };
    }
    if (method === 'Calendar/get') {
      return { methodResponses: [['Calendar/get', { accountId: 'a', state: 's', list: [], notFound: [] }, 'lists']], sessionState: 's' };
    }
    if (method === 'Task/query') {
      return {
        methodResponses: [
          ['Task/query', { accountId: 'a', ids: [] }, 'q'],
          ['Task/get', { accountId: 'a', state: 's', list: [], notFound: [] }, 'g'],
        ],
        sessionState: 's',
      };
    }
    return {
      methodResponses: [
        ['Note/query', { accountId: 'a', ids: [] }, 'q'],
        ['Note/get', { accountId: 'a', state: 's', list: [], notFound: [] }, 'g'],
      ],
      sessionState: 's',
    };
  };
  const session = async (): Promise<JmapSession> =>
    ({ primaryAccounts: { [CAP_NOTES]: 'a', 'urn:mailwoman:tasks': 'a' }, accounts: { a: {} } }) as unknown as JmapSession;
  return { jmap, session, onNetwork: () => () => undefined } as unknown as Client;
}

function ctxWith(calls: Call[]): SliceContext {
  return { client: recordingClient(calls), showToast: () => undefined };
}

describe('note and task saves use keepalive; loads do not', () => {
  it('notes: create, update and delete ask for keepalive; loadNotes does not', async () => {
    const calls: Call[] = [];
    await new Promise<void>((resolve, reject) => {
      createRoot(async (dispose) => {
        try {
          const s = createNotesSlice(ctxWith(calls));
          await s.loadNotes();
          const note = await s.createNote({ title: 'A' });
          await s.updateNote(note.id, { title: 'B' });
          await s.deleteNote(note.id);
          resolve();
        } catch (e) {
          reject(e);
        } finally {
          dispose();
        }
      });
    });
    expect(calls.filter((c) => c.method === 'Note/set').map((c) => c.keepalive)).toEqual([true, true, true]);
    expect(calls.filter((c) => c.method === 'Note/query').every((c) => !c.keepalive)).toBe(true);
  });

  it('tasks: create, update and delete ask for keepalive; loadTasks does not', async () => {
    const calls: Call[] = [];
    await new Promise<void>((resolve, reject) => {
      createRoot(async (dispose) => {
        try {
          const s = createTasksSlice(ctxWith(calls));
          await s.loadTasks();
          const id = await s.createTask({ listId: 'l1', title: 'A' });
          await s.updateTask(id!, { title: 'B' });
          await s.deleteTask(id!);
          resolve();
        } catch (e) {
          reject(e);
        } finally {
          dispose();
        }
      });
    });
    expect(calls.filter((c) => c.method === 'Task/set').map((c) => c.keepalive)).toEqual([true, true, true]);
    expect(calls.filter((c) => c.method !== 'Task/set').every((c) => !c.keepalive)).toBe(true);
  });
});
