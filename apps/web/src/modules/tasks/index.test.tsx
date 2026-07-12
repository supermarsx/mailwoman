import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { TasksModule } from './index.tsx';
import { AppContext } from '../../state/context.ts';
import { createAppState, type AppState } from '../../state/store.ts';
import { todayDate } from '../../state/slices/tasks.ts';
import type { Client } from '../../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../../api/jmap-types.ts';
import type { Task } from '../../api/pim-types.ts';
import type { TaskList } from '../../state/slices/tasks.ts';

// ── fixtures ────────────────────────────────────────────────────────────────

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

const LISTS: TaskList[] = [{ id: 'l1', name: 'Work', color: '#3b82f6', order: 0 }];

/** A mock JMAP client that answers `Calendar/get`, `Task/query`+`Task/get` and
 *  `Task/set` from an in-memory seed, recording every request it is handed. */
function mockClient(tasks: Task[], onJmap?: (body: JmapRequest) => void): Client {
  const jmap = async (body: JmapRequest): Promise<JmapResponse> => {
    onJmap?.(body);
    const method = body.methodCalls[0]?.[0];
    if (method === 'Calendar/get') {
      return {
        methodResponses: [['Calendar/get', { accountId: 'acct1', state: 's', list: LISTS, notFound: [] }, 'lists']],
      } as unknown as JmapResponse;
    }
    if (method === 'Task/query') {
      return {
        methodResponses: [
          ['Task/query', { accountId: 'acct1', queryState: 'q', ids: tasks.map((t) => t.id), position: 0 }, 'q'],
          ['Task/get', { accountId: 'acct1', state: 's', list: tasks, notFound: [] }, 'g'],
        ],
      } as unknown as JmapResponse;
    }
    // Task/set — echo a created row for the `new` create key.
    return {
      methodResponses: [
        [
          'Task/set',
          {
            accountId: 'acct1',
            oldState: 's',
            newState: 's2',
            created: { new: { id: 'created-1', uid: 'created-1' } },
            updated: null,
            destroyed: null,
            notCreated: null,
            notUpdated: null,
            notDestroyed: null,
          },
          'set',
        ],
      ],
    } as unknown as JmapResponse;
  };
  const session = async (): Promise<JmapSession> =>
    ({ primaryAccounts: { 'urn:mailwoman:tasks': 'acct1' }, accounts: { acct1: {} } }) as unknown as JmapSession;
  return { jmap, session, onNetwork: () => () => undefined } as unknown as Client;
}

function renderTasks(tasks: Task[], onJmap?: (body: JmapRequest) => void): AppState {
  const app = createAppState(mockClient(tasks, onJmap));
  render(() => (
    <AppContext.Provider value={app}>
      <TasksModule />
    </AppContext.Provider>
  ));
  return app;
}

// ── tests ───────────────────────────────────────────────────────────────────

describe('TasksModule', () => {
  it('renders the loaded task list and its lists in the sidebar', async () => {
    renderTasks([mkTask({ id: 't1', title: 'Write the report' }), mkTask({ id: 't2', title: 'Book the room' })]);
    expect(await screen.findByText('Write the report')).toBeInTheDocument();
    expect(screen.getByText('Book the room')).toBeInTheDocument();
    // The list from Calendar/get shows in the sidebar.
    expect(screen.getByRole('button', { name: /Work/ })).toBeInTheDocument();
  });

  it('nests subtasks under their parent (RELATED-TO via parentId)', async () => {
    renderTasks([
      mkTask({ id: 'p1', title: 'Ship V3' }),
      mkTask({ id: 's1', title: 'Draft the plan', parentId: 'p1' }),
    ]);
    await screen.findByText('Ship V3');
    const subtasks = screen.getByRole('list', { name: 'Subtasks' });
    expect(within(subtasks).getByText('Draft the plan')).toBeInTheDocument();
    // The subtask is nested, not a second root row: the root list has exactly
    // one direct <li> (which in turn contains the subtask list).
    const rootList = screen.getByRole('list', { name: 'Tasks' });
    expect(rootList.querySelectorAll(':scope > li')).toHaveLength(1);
  });

  it('My Day shows only due-today / overdue / pinned tasks, hiding the rest', async () => {
    const today = todayDate();
    renderTasks([
      mkTask({ id: 'due', title: 'Due today', due: `${today}T09:00:00` }),
      mkTask({ id: 'pin', title: 'Pinned today', myDayDate: today }),
      mkTask({ id: 'later', title: 'Way later', due: '2099-01-01T09:00:00' }),
      mkTask({ id: 'done', title: 'Already done', due: `${today}T08:00:00`, status: 'completed' }),
    ]);
    await screen.findByText('Way later');
    fireEvent.click(screen.getByRole('button', { name: 'My Day' }));
    const myDay = screen.getByRole('list', { name: 'My Day' });
    expect(within(myDay).getByText('Due today')).toBeInTheDocument();
    expect(within(myDay).getByText('Pinned today')).toBeInTheDocument();
    expect(within(myDay).queryByText('Way later')).toBeNull();
    expect(within(myDay).queryByText('Already done')).toBeNull();
  });

  it('completing a task flips it done and swaps the toggle label to Reopen', async () => {
    const app = renderTasks([mkTask({ id: 't1', title: 'Close the ticket' })]);
    const box = (await screen.findByRole('checkbox', { name: 'Complete Close the ticket' })) as HTMLInputElement;
    fireEvent.click(box);
    await waitFor(() => expect(app.tasks().find((t) => t.id === 't1')?.status).toBe('completed'));
    expect(screen.getByRole('checkbox', { name: 'Reopen Close the ticket' })).toBeInTheDocument();
  });

  it('mail→task sends a Task/set create carrying fromEmail (convert stub)', async () => {
    const sent: JmapRequest[] = [];
    renderTasks([mkTask({ id: 't1', title: 'Existing' })], (body) => sent.push(body));
    await screen.findByText('Existing');
    fireEvent.input(screen.getByRole('textbox', { name: 'Message id to convert' }), { target: { value: 'm-42' } });
    fireEvent.click(screen.getByRole('button', { name: 'Mail → task' }));
    await waitFor(() => {
      const setReq = sent.find((b) => b.methodCalls[0]?.[0] === 'Task/set');
      expect(setReq).toBeDefined();
      const create = setReq?.methodCalls[0]?.[1]?.['create'] as Record<string, Record<string, unknown>>;
      expect(create['new']?.['fromEmail']).toEqual({ emailId: 'm-42' });
    });
    // The optimistic converted row appears without a reload.
    expect(await screen.findByText('Follow up: mail m-42')).toBeInTheDocument();
  });
});
