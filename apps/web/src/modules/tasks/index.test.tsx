import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { TasksModule, dueFromDateInput, priorityChoice } from './index.tsx';
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

// ── editing + delete (26.20 t28-e5) ─────────────────────────────────────────
//
// The engine's `Task/set` update shallow-merges the patch into the stored task
// (`task_update`, crates/mw-engine/src/pim/tasks.rs:145-150) and destroy takes a
// list of ids (`task_destroy`, :173-189), so the assertions below are on the
// exact `update` / `destroy` arguments the module sends.

/** The `Task/set` calls carrying `key`, in the order they were sent. */
function setCalls(sent: JmapRequest[], key: 'update' | 'destroy'): unknown[] {
  return sent
    .filter((b) => b.methodCalls[0]?.[0] === 'Task/set')
    .map((b) => (b.methodCalls[0]?.[1] as Record<string, unknown>)[key])
    .filter((v) => v !== undefined);
}

describe('TasksModule — edit and delete', () => {
  it('edits title, due date and priority, sending one Task/set update with exactly those fields', async () => {
    const sent: JmapRequest[] = [];
    const app = renderTasks([mkTask({ id: 't1', title: 'Write the report' })], (body) => sent.push(body));
    fireEvent.click(await screen.findByRole('button', { name: 'Edit Write the report' }));
    const form = screen.getByRole('form', { name: 'Edit Write the report' });
    // Precondition: opening the editor sends nothing.
    expect(setCalls(sent, 'update')).toHaveLength(0);

    fireEvent.input(within(form).getByLabelText('Title'), { target: { value: 'Write the Q3 report' } });
    fireEvent.input(within(form).getByLabelText('Due date'), { target: { value: '2026-10-09' } });
    fireEvent.change(within(form).getByLabelText('Priority'), { target: { value: '1' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Save' }));

    await waitFor(() => expect(setCalls(sent, 'update')).toHaveLength(1));
    expect(setCalls(sent, 'update')[0]).toEqual({
      t1: { title: 'Write the Q3 report', due: '2026-10-09', priority: 1 },
    });
    // The row shows the new values and the form is gone.
    expect(await screen.findByText('Write the Q3 report')).toBeInTheDocument();
    expect(screen.getByText('2026-10-09')).toBeInTheDocument();
    expect(screen.getByLabelText('High priority')).toBeInTheDocument();
    expect(screen.queryByRole('form')).toBeNull();
    expect(app.tasks().find((t) => t.id === 't1')?.title).toBe('Write the Q3 report');
  });

  it('sends only the fields that changed — an untouched priority is not rewritten', async () => {
    const sent: JmapRequest[] = [];
    renderTasks(
      [mkTask({ id: 't1', title: 'Renew passport', priority: 3, due: '2026-10-09T17:00:00' })],
      (body) => sent.push(body),
    );
    fireEvent.click(await screen.findByRole('button', { name: 'Edit Renew passport' }));
    const form = screen.getByRole('form', { name: 'Edit Renew passport' });
    // The stored values are what the form opens with.
    expect((within(form).getByLabelText('Due date') as HTMLInputElement).value).toBe('2026-10-09');
    expect((within(form).getByLabelText('Priority') as HTMLSelectElement).value).toBe('1');
    fireEvent.input(within(form).getByLabelText('Due date'), { target: { value: '2026-10-12' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(setCalls(sent, 'update')).toHaveLength(1));
    // The time of day is kept; title and priority (3) are not in the patch.
    expect(setCalls(sent, 'update')[0]).toEqual({ t1: { due: '2026-10-12T17:00:00' } });
  });

  it('clearing the due date sends due: null', async () => {
    const sent: JmapRequest[] = [];
    renderTasks([mkTask({ id: 't1', title: 'Someday', due: '2026-10-09' })], (body) => sent.push(body));
    fireEvent.click(await screen.findByRole('button', { name: 'Edit Someday' }));
    fireEvent.input(screen.getByLabelText('Due date'), { target: { value: '' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(setCalls(sent, 'update')).toHaveLength(1));
    expect(setCalls(sent, 'update')[0]).toEqual({ t1: { due: null } });
  });

  it('Cancel, an unchanged Save, and an emptied title send nothing', async () => {
    const sent: JmapRequest[] = [];
    renderTasks([mkTask({ id: 't1', title: 'Keep me' })], (body) => sent.push(body));

    fireEvent.click(await screen.findByRole('button', { name: 'Edit Keep me' }));
    fireEvent.input(screen.getByLabelText('Title'), { target: { value: 'Changed my mind' } });
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.getByText('Keep me')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Edit Keep me' }));
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(screen.queryByRole('form')).toBeNull());

    fireEvent.click(screen.getByRole('button', { name: 'Edit Keep me' }));
    fireEvent.input(screen.getByLabelText('Title'), { target: { value: '   ' } });
    fireEvent.submit(screen.getByRole('form'));
    // The edit stays open on an empty title.
    expect(screen.getByRole('form')).toBeInTheDocument();

    expect(setCalls(sent, 'update')).toHaveLength(0);
    expect(setCalls(sent, 'destroy')).toHaveLength(0);
  });

  it('delete asks first, then sends Task/set destroy and removes the row', async () => {
    const sent: JmapRequest[] = [];
    const app = renderTasks(
      [mkTask({ id: 't1', title: 'Old chore' }), mkTask({ id: 't2', title: 'Stays' })],
      (body) => sent.push(body),
    );
    fireEvent.click(await screen.findByRole('button', { name: 'Edit Old chore' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    // First click only asks: nothing is sent and the task is still there.
    expect(screen.getByText('Delete this task? This cannot be undone.')).toBeInTheDocument();
    expect(setCalls(sent, 'destroy')).toHaveLength(0);
    expect(app.tasks().some((t) => t.id === 't1')).toBe(true);

    // Declining keeps it.
    fireEvent.click(screen.getByRole('button', { name: 'Keep task' }));
    expect(setCalls(sent, 'destroy')).toHaveLength(0);

    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete task' }));
    await waitFor(() => expect(setCalls(sent, 'destroy')).toEqual([['t1']]));
    await waitFor(() => expect(app.tasks().some((t) => t.id === 't1')).toBe(false));
    expect(screen.queryByText('Old chore')).toBeNull();
    expect(screen.getByText('Stays')).toBeInTheDocument();
  });
});

describe('task editor helpers', () => {
  it('priorityChoice maps stored iCalendar priorities onto the offered choices', () => {
    expect([0, 1, 4, 5, 6, 9, 12].map(priorityChoice)).toEqual([0, 1, 1, 5, 9, 9, 0]);
  });

  it('dueFromDateInput keeps an existing time of day and clears on empty', () => {
    expect(dueFromDateInput('2026-10-12', null)).toBe('2026-10-12');
    expect(dueFromDateInput('2026-10-12', '2026-10-09')).toBe('2026-10-12');
    expect(dueFromDateInput('2026-10-12', '2026-10-09T17:00:00')).toBe('2026-10-12T17:00:00');
    expect(dueFromDateInput('', '2026-10-09T17:00:00')).toBeNull();
  });
});
