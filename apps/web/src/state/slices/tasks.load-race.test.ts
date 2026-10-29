// `loadTasks()` / `selectList()` when two calls overlap (t31-e4, 26.20).
//
// Both make their requests and write `tasks` after the awaits. Opening Tasks
// starts two loads (the surface switch and the module mount each call
// `loadTasks()`), and a list click starts `selectList()` while a load may still
// be out. If an earlier call answers after a later one, it must not put its
// older rows back, and it must not end the loading state of the later one.
//
// The backend is an in-memory mock whose `Task/query` answers are computed when
// asked (so a held answer is the one from before any later write) and delivered
// only once the test releases them. Same shape as
// `modules/calendar/controller.load-race.test.ts` (t30-h1).

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

function mkTask(id: string, listId: string): Task {
  return {
    id,
    listId,
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
  };
}

const LISTS = [
  { id: 'l1', name: 'Work', color: '#3b82f6', order: 0 },
  { id: 'l2', name: 'Home', color: '#22c55e', order: 1 },
];

/** A stateful mock client; the `Task/query` calls numbered in `hold` (1-based)
 *  wait for their gate's release before answering, and so do the
 *  `Calendar/get` calls numbered in `holdLists`. */
function harness(
  hold: number[],
  holdLists: number[] = [],
): { client: Client; gates: Map<number, Gate>; listGates: Map<number, Gate> } {
  let store: Task[] = [mkTask('t1', 'l1'), mkTask('t2', 'l2')];
  let queries = 0;
  let listCalls = 0;
  let created = 0;
  const gates = new Map(hold.map((n) => [n, gate()] as const));
  const listGates = new Map(holdLists.map((n) => [n, gate()] as const));
  const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
    const [name, args] = body.methodCalls[0]!;
    if (name === 'Calendar/get') {
      listCalls += 1;
      const g = listGates.get(listCalls);
      if (g !== undefined) {
        g.reach();
        await g.released;
      }
      return {
        methodResponses: [['Calendar/get', { accountId: 'acct1', state: 's', list: LISTS, notFound: [] }, 'lists']],
      } as unknown as JmapResponse;
    }
    if (name === 'Task/query') {
      queries += 1;
      const listId = (args['filter'] as { listId?: string }).listId;
      const list = store.filter((t) => listId === undefined || t.listId === listId);
      const g = gates.get(queries);
      if (g !== undefined) {
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
    // Task/set — only creates are used here.
    const create = (args['create'] ?? {}) as Record<string, Partial<Task>>;
    const out: Record<string, { id: string; uid: string }> = {};
    for (const [key, c] of Object.entries(create)) {
      const id = `srv-${(created += 1)}`;
      store = [...store, { ...mkTask(id, c.listId ?? 'l1'), ...c, id, uid: id }];
      out[key] = { id, uid: id };
    }
    return {
      methodResponses: [['Task/set', { accountId: 'acct1', created: out, notCreated: null }, 'set']],
    } as unknown as JmapResponse;
  };
  const session = async (): Promise<JmapSession> =>
    ({ primaryAccounts: { 'urn:mailwoman:tasks': 'acct1' }, accounts: { acct1: {} } }) as unknown as JmapSession;
  const client = { jmap, session, onNetwork: () => () => undefined } as unknown as Client;
  return { client, gates, listGates };
}

function withSlice(client: Client, run: (s: TasksSlice) => Promise<void>): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    createRoot((dispose) => {
      const ctx: SliceContext = { client, showToast: () => undefined };
      run(createTasksSlice(ctx)).then(resolve, reject).finally(dispose);
    });
  });
}

describe('tasks slice — overlapping loads', () => {
  it('keeps the rows of the load that started last when an earlier loadTasks answers after it', async () => {
    const { client, gates } = harness([1]);
    await withSlice(client, async (s) => {
      const first = s.loadTasks();
      await gates.get(1)!.reached;

      // A task written while the first load waits, then a load that runs to the end.
      const id = await s.createTask({ listId: 'l1', title: 'Added while loading' });
      expect(id).not.toBeNull();
      await s.loadTasks();
      expect(s.tasks().some((t) => t.id === id)).toBe(true);

      gates.get(1)!.release();
      await first;

      expect(s.tasks().some((t) => t.id === id)).toBe(true);
      expect(s.tasks().map((t) => t.id).sort()).toEqual(['srv-1', 't1', 't2']);
      expect(s.tasksLoading()).toBe(false);
    });
  });

  it('shows the list picked last when an earlier selectList answers after it', async () => {
    const { client, gates } = harness([1]);
    await withSlice(client, async (s) => {
      const first = s.selectList('l1');
      await gates.get(1)!.reached;
      await s.selectList('l2');
      expect(s.tasks().map((t) => t.id)).toEqual(['t2']);

      gates.get(1)!.release();
      await first;

      expect(s.selectedListId()).toBe('l2');
      expect(s.tasks().map((t) => t.id)).toEqual(['t2']);
      expect(s.tasksLoading()).toBe(false);
    });
  });

  it('shows the list picked while loadTasks was still out, not the load\'s older answer', async () => {
    const { client, gates } = harness([1]);
    await withSlice(client, async (s) => {
      const first = s.loadTasks();
      await gates.get(1)!.reached;
      await s.selectList('l2');

      gates.get(1)!.release();
      await first;

      expect(s.selectedListId()).toBe('l2');
      expect(s.tasks().map((t) => t.id)).toEqual(['t2']);
      expect(s.taskLists().map((l) => l.id)).toEqual(['l1', 'l2']);
    });
  });

  it('still shows the task lists when a list is picked while loadTasks waits for them', async () => {
    const { client, listGates } = harness([], [1]);
    await withSlice(client, async (s) => {
      const first = s.loadTasks();
      await listGates.get(1)!.reached;
      await s.selectList('l2');

      listGates.get(1)!.release();
      await first;

      expect(s.taskLists().map((l) => l.id)).toEqual(['l1', 'l2']);
      expect(s.tasks().map((t) => t.id)).toEqual(['t2']);
      expect(s.tasksLoading()).toBe(false);
    });
  });

  it('stays loading until the latest call answers, whichever answers first', async () => {
    const { client, gates } = harness([1, 2]);
    await withSlice(client, async (s) => {
      const first = s.loadTasks();
      await gates.get(1)!.reached;
      const second = s.selectList('l2');
      await gates.get(2)!.reached;
      expect(s.tasksLoading()).toBe(true);

      gates.get(1)!.release();
      await first;
      expect(s.tasksLoading()).toBe(true);

      gates.get(2)!.release();
      await second;
      expect(s.tasksLoading()).toBe(false);
      expect(s.tasks().map((t) => t.id)).toEqual(['t2']);
    });
  });
});

// t84: a Task/get answer without `list` (a stubbed or partial backend) threw
// inside `merge()`; the rejection of a `void app.loadTasks()` went unhandled
// (coverage run on fcaef3f7). It is now ignored: the rows shown stay, none is
// taken for deleted elsewhere, and the loading state ends.
describe('tasks slice — a Task/get answer without a list', () => {
  it('keeps the shown rows and does not reject', async () => {
    const { client } = harness([]);
    let partial = false;
    const real = client.jmap.bind(client);
    (client as { jmap: Client['jmap'] }).jmap = (async (body: JmapRequest) => {
      const res = await real(body);
      if (!partial || body.methodCalls[0]![0] !== 'Task/query') return res;
      return {
        methodResponses: res.methodResponses.map((call) => (call[2] === 'g' ? [call[0], {}, 'g'] : call)),
      } as unknown as JmapResponse;
    }) as Client['jmap'];
    await withSlice(client, async (s) => {
      await s.loadTasks();
      expect(s.tasks().map((t) => t.id).sort()).toEqual(['t1', 't2']);

      partial = true;
      await expect(s.loadTasks()).resolves.toBeUndefined();
      await expect(s.selectList(null)).resolves.toBeUndefined();

      expect(s.tasks().map((t) => t.id).sort()).toEqual(['t1', 't2']);
      expect(s.taskDeletedElsewhere('t1')).toBe(false);
      expect(s.tasksLoading()).toBe(false);
    });
  });
});
