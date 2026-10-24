// `loadTasks()` answering while this client is writing (t32-h11, 26.20).
//
// A change pushed from another device or session makes the open Tasks module
// reload. That reload can be answered from data read before a write this
// client has just made reached the server; applied as-is it would drop a task
// just created, put back an old title, or show a task just deleted. A reload
// also must not hand out new objects for tasks whose content did not change.
// The latest-wins guarantee of `tasks.load-race.test.ts` (t31-e4) still holds.

import { describe, it, expect } from 'vitest';
import { createRoot } from 'solid-js';
import type { Client } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import type { Task } from '../../api/pim-types.ts';
import type { SliceContext } from './context.ts';
import { createTasksSlice, type TasksSlice } from './tasks.ts';

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

function mkTask(id: string, over: Partial<Task> = {}): Task {
  return {
    id,
    listId: 'l1',
    uid: id,
    title: id,
    description: '',
    start: null,
    due: null,
    timeZone: null,
    priority: 0,
    percentComplete: 0,
    status: 'needs-action',
    progress: '',
    recurrenceRules: [],
    parentId: null,
    myDayDate: null,
    etag: null,
    ...over,
  };
}

const LISTS = [
  { id: 'l1', name: 'Work', color: '#3b82f6', order: 0 },
  { id: 'l2', name: 'Home', color: '#22c55e', order: 1 },
];

interface Server {
  client: Client;
  rows: Task[];
  holdNextLoad(): Gate;
}

function server(seed: Task[]): Server {
  const s: Server = {
    rows: seed.map((t) => ({ ...t })),
    client: undefined as unknown as Client,
    holdNextLoad: () => (loadGate = gate()),
  };
  let loadGate: Gate | null = null;
  let created = 0;
  const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
    const [name, args] = body.methodCalls[0]!;
    if (name === 'Calendar/get') {
      return {
        methodResponses: [['Calendar/get', { accountId: 'acct1', state: 's', list: LISTS, notFound: [] }, 'lists']],
      } as unknown as JmapResponse;
    }
    if (name === 'Task/query') {
      const listId = (args['filter'] as { listId?: string }).listId;
      const list = s.rows.filter((t) => listId === undefined || t.listId === listId).map((t) => structuredClone(t));
      const g = loadGate;
      loadGate = null;
      if (g !== null) {
        g.reach();
        await g.released;
      }
      return {
        methodResponses: [
          ['Task/query', { accountId: 'acct1', queryState: 'q', ids: list.map((t) => t.id), position: 0 }, 'q'],
          ['Task/get', { accountId: 'acct1', state: 's', list, notFound: [] }, 'g'],
        ],
      } as unknown as JmapResponse;
    }
    const create = (args['create'] ?? {}) as Record<string, Partial<Task>>;
    const update = (args['update'] ?? {}) as Record<string, Partial<Task>>;
    const destroy = (args['destroy'] ?? []) as string[];
    const out: Record<string, { id: string; uid: string }> = {};
    for (const [key, c] of Object.entries(create)) {
      const id = `srv-${(created += 1)}`;
      s.rows = [...s.rows, mkTask(id, { ...c, id, uid: id })];
      out[key] = { id, uid: id };
    }
    for (const [id, patch] of Object.entries(update)) {
      s.rows = s.rows.map((t) => (t.id === id ? { ...t, ...patch } : t));
    }
    s.rows = s.rows.filter((t) => !destroy.includes(t.id));
    return {
      methodResponses: [['Task/set', { accountId: 'acct1', created: out, notCreated: null }, 'set']],
    } as unknown as JmapResponse;
  };
  const session = async (): Promise<JmapSession> =>
    ({ primaryAccounts: { 'urn:mailwoman:tasks': 'acct1' }, accounts: { acct1: {} } }) as unknown as JmapSession;
  s.client = { jmap, session, onNetwork: () => () => undefined } as unknown as Client;
  return s;
}

function withSlice(client: Client, run: (s: TasksSlice) => Promise<void>): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    createRoot((dispose) => {
      const ctx: SliceContext = { client, showToast: () => undefined };
      run(createTasksSlice(ctx)).then(resolve, reject).finally(dispose);
    });
  });
}

describe('tasks slice — a reload answering while this client writes', () => {
  it('keeps a task created while a reload was out, when the reload answers without it', async () => {
    const srv = server([mkTask('t1')]);
    await withSlice(srv.client, async (s) => {
      await s.loadTasks();
      const held = srv.holdNextLoad();
      const reload = s.loadTasks();
      await held.reached;

      const id = await s.createTask({ listId: 'l1', title: 'Saved during the reload' });
      held.release();
      await reload;
      expect(s.tasks().map((t) => t.id)).toEqual(['t1', id]);
    });
  });

  it('keeps the title saved while a reload was out, when the reload answers the old title', async () => {
    const srv = server([mkTask('t1', { title: 'Old' })]);
    await withSlice(srv.client, async (s) => {
      await s.loadTasks();
      const held = srv.holdNextLoad();
      const reload = s.loadTasks();
      await held.reached;

      await s.updateTask('t1', { title: 'Saved' });
      held.release();
      await reload;
      expect(s.tasks()[0]?.title).toBe('Saved');
    });
  });

  it('does not put back a task deleted while a reload was out, and keeps its subtasks as roots', async () => {
    const srv = server([mkTask('p1'), mkTask('s1', { parentId: 'p1' })]);
    await withSlice(srv.client, async (s) => {
      await s.loadTasks();
      const held = srv.holdNextLoad();
      const reload = s.loadTasks();
      await held.reached;

      await s.deleteTask('p1');
      held.release();
      await reload;
      expect(s.tasks().map((t) => t.id)).toEqual(['s1']);
      expect(s.rootTasks().map((t) => t.id)).toEqual(['s1']);
      expect(s.taskDeletedElsewhere('p1')).toBe(false);
    });
  });

  it('marks a task removed elsewhere, but not one that left the view because another list was picked', async () => {
    const srv = server([mkTask('t1'), mkTask('t2'), mkTask('h1', { listId: 'l2' })]);
    await withSlice(srv.client, async (s) => {
      await s.loadTasks();
      srv.rows = srv.rows.filter((t) => t.id !== 't2');
      await s.loadTasks();
      expect(s.taskDeletedElsewhere('t2')).toBe(true);
      expect(s.taskDeletedElsewhere('t1')).toBe(false);

      await s.selectList('l2');
      expect(s.tasks().map((t) => t.id)).toEqual(['h1']);
      expect(s.taskDeletedElsewhere('t1')).toBe(false);
    });
  });

  it('keeps the same objects for tasks a reload did not change', async () => {
    const srv = server([mkTask('t1'), mkTask('t2')]);
    await withSlice(srv.client, async (s) => {
      await s.loadTasks();
      const [a, b] = s.tasks();

      srv.rows = srv.rows.map((t) => (t.id === 't2' ? { ...t, title: 'Changed' } : t));
      await s.loadTasks();
      expect(s.tasks()[0]).toBe(a);
      expect(s.tasks()[1]).not.toBe(b);
      expect(s.tasks()[1]?.title).toBe('Changed');
    });
  });

  it('does not carry a task created in one list into another list picked meanwhile', async () => {
    const srv = server([mkTask('t1'), mkTask('h1', { listId: 'l2' })]);
    await withSlice(srv.client, async (s) => {
      await s.loadTasks();
      const held = srv.holdNextLoad();
      const picking = s.selectList('l2');
      await held.reached;

      await s.createTask({ listId: 'l1', title: 'Work item' });
      held.release();
      await picking;
      expect(s.tasks().map((t) => t.id)).toEqual(['h1']);
    });
  });
});
