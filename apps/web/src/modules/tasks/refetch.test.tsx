// An open task edit form across a pushed reload (t32-h11, 26.20).
//
// When another device or session changes a task, the server pushes a change and
// the Tasks module reloads its list (`refetchPim` -> `loadTasks()`). The reload
// brings new objects for every task. The edit form a user has open must stay
// open with what they typed; if the task they are editing changed elsewhere the
// draft is kept and a notice says so, and if it was deleted elsewhere the form
// closes with a notice.

import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { TasksModule } from './index.tsx';
import { AppContext } from '../../state/context.ts';
import { createAppState, type AppState } from '../../state/store.ts';
import type { Client } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import type { Task } from '../../api/pim-types.ts';

function mkTask(over: Partial<Task> & Pick<Task, 'id' | 'title'>): Task {
  return {
    listId: 'l1',
    uid: over.id,
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

interface Server {
  rows: Task[];
  updates: Array<Record<string, Partial<Task>>>;
}

/** A mock client over `srv.rows`; every load answers fresh objects. */
function mockClient(srv: Server): Client {
  const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
    const [method, args] = body.methodCalls[0]!;
    if (method === 'Calendar/get') {
      return {
        methodResponses: [
          [
            'Calendar/get',
            { accountId: 'acct1', state: 's', list: [{ id: 'l1', name: 'Work', color: '#3b82f6', order: 0 }], notFound: [] },
            'lists',
          ],
        ],
      } as unknown as JmapResponse;
    }
    if (method === 'Task/query') {
      const list = srv.rows.map((t) => structuredClone(t));
      return {
        methodResponses: [
          ['Task/query', { accountId: 'acct1', queryState: 'q', ids: list.map((t) => t.id), position: 0 }, 'q'],
          ['Task/get', { accountId: 'acct1', state: 's', list, notFound: [] }, 'g'],
        ],
      } as unknown as JmapResponse;
    }
    const update = (args['update'] ?? {}) as Record<string, Partial<Task>>;
    const destroy = (args['destroy'] ?? []) as string[];
    if (Object.keys(update).length > 0) srv.updates.push(update);
    for (const [id, patch] of Object.entries(update)) {
      srv.rows = srv.rows.map((t) => (t.id === id ? { ...t, ...patch } : t));
    }
    srv.rows = srv.rows.filter((t) => !destroy.includes(t.id));
    return {
      methodResponses: [['Task/set', { accountId: 'acct1', created: null, notCreated: null }, 'set']],
    } as unknown as JmapResponse;
  };
  const session = async (): Promise<JmapSession> =>
    ({ primaryAccounts: { 'urn:mailwoman:tasks': 'acct1' }, accounts: { acct1: {} } }) as unknown as JmapSession;
  return { jmap, session, onNetwork: () => () => undefined } as unknown as Client;
}

function setup(seed: Task[]): { app: AppState; srv: Server } {
  const srv: Server = { rows: seed, updates: [] };
  const app = createAppState(mockClient(srv));
  render(() => (
    <AppContext.Provider value={app}>
      <TasksModule />
    </AppContext.Provider>
  ));
  return { app, srv };
}

const SEED = (): Task[] => [
  mkTask({ id: 't1', title: 'Write the report' }),
  mkTask({ id: 't2', title: 'Book the room' }),
  mkTask({ id: 's1', title: 'Draft the outline', parentId: 't1' }),
];

/** Open `title`'s edit form and type `draft` into its Title field. */
async function editAndType(title: string, draft: string): Promise<HTMLInputElement> {
  fireEvent.click(await screen.findByRole('button', { name: `Edit ${title}` }));
  const form = screen.getByRole('form', { name: `Edit ${title}` });
  const input = form.querySelector('input[type="text"]') as HTMLInputElement;
  fireEvent.input(input, { target: { value: draft } });
  return input;
}

function openForm(): HTMLFormElement | null {
  return document.querySelector('form[aria-label^="Edit "]');
}

function titleField(): HTMLInputElement | null {
  return openForm()?.querySelector('input[type="text"]') ?? null;
}

describe('TasksModule — an open edit form across a reload', () => {
  it('stays open with the typed title when the reload brings equal rows', async () => {
    const { app } = setup(SEED());
    await editAndType('Write the report', 'Write the final report');

    await app.loadTasks();
    expect(openForm()).not.toBeNull();
    expect(titleField()?.value).toBe('Write the final report');
    expect(screen.queryByText(/changed on another device/)).toBeNull();
  });

  it('stays open with the typed title when another task changed', async () => {
    const { app, srv } = setup(SEED());
    await editAndType('Write the report', 'Write the final report');

    srv.rows = srv.rows.map((t) => (t.id === 't2' ? { ...t, title: 'Book the big room' } : t));
    await app.loadTasks();
    expect(await screen.findByText('Book the big room')).toBeInTheDocument();
    expect(titleField()?.value).toBe('Write the final report');
    expect(screen.queryByText(/changed on another device/)).toBeNull();
  });

  it('keeps a subtask edit open when its parent changed', async () => {
    const { app, srv } = setup(SEED());
    await editAndType('Draft the outline', 'Draft the full outline');

    srv.rows = srv.rows.map((t) => (t.id === 't1' ? { ...t, priority: 1 } : t));
    await app.loadTasks();
    expect(await screen.findByLabelText('High priority')).toBeInTheDocument();
    expect(titleField()?.value).toBe('Draft the full outline');
  });

  it('keeps the draft and says so when the task being edited changed elsewhere; Save sends only what the user changed', async () => {
    const { app, srv } = setup(SEED());
    await editAndType('Write the report', 'Write the final report');

    srv.rows = srv.rows.map((t) => (t.id === 't1' ? { ...t, priority: 5 } : t));
    await app.loadTasks();
    expect(titleField()?.value).toBe('Write the final report');
    expect(await screen.findByText(/changed on another device/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(srv.updates).toEqual([{ t1: { title: 'Write the final report' } }]));
    expect(srv.rows.find((t) => t.id === 't1')?.priority).toBe(5);
  });

  it('keeps the typed title when the same field was changed elsewhere', async () => {
    const { app, srv } = setup(SEED());
    await editAndType('Write the report', 'Mine');

    srv.rows = srv.rows.map((t) => (t.id === 't1' ? { ...t, title: 'Theirs' } : t));
    await app.loadTasks();
    expect(titleField()?.value).toBe('Mine');
    expect(await screen.findByText(/changed on another device/)).toBeInTheDocument();
  });

  it('closes the form with a notice when the task being edited was deleted elsewhere', async () => {
    const { app, srv } = setup(SEED());
    await editAndType('Book the room', 'Book the big room');

    srv.rows = srv.rows.filter((t) => t.id !== 't2');
    await app.loadTasks();
    await waitFor(() => expect(openForm()).toBeNull());
    const notice = await screen.findByText(/was deleted on another device/);
    expect(notice).toHaveTextContent('Book the room');

    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByText(/was deleted on another device/)).toBeNull();
  });

  it('shows no notice when the user deletes the task themselves', async () => {
    const { app } = setup(SEED());
    fireEvent.click(await screen.findByRole('button', { name: 'Edit Book the room' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete task' }));
    await waitFor(() => expect(screen.queryByText('Book the room')).toBeNull());

    await app.loadTasks();
    expect(screen.queryByText(/was deleted on another device/)).toBeNull();
  });
});
