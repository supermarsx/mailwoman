// Tasks module (plan §2.5, §3 e5) — mounted into the app shell via the frozen
// `AppModule` registry (`shell/modules.ts`). Task lists (CalDAV VTODO
// collections), a create form, root tasks with nested subtasks, the My Day /
// Today view, complete/reopen, drag-reorder of My Day, and the mail→task /
// event→task convert entry points — all over `state/slices/tasks.ts` and the
// frozen `Task/*` surface (mock-backed until e10 swaps in the real engine).
//
// Shared primitives (ribbon / command-palette / tokens) are reused read-only;
// this module owns only its own view. Styling is token-native (`tasks.css.ts`).

import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { useApp } from '../../state/context.ts';
import { isDone } from '../../state/slices/tasks.ts';
import type { Id } from '../../api/jmap-types.ts';
import type { Task } from '../../api/pim-types.ts';
import * as css from './tasks.css.ts';

/** Which working set the main pane shows. */
type View = 'list' | 'myDay';

/** Short human label for a task's due/start date, or empty when unscheduled. */
function scheduleLabel(task: Task): string {
  const when = task.due ?? task.start;
  return when === null ? '' : when.slice(0, 10);
}

export function TasksModule(): JSX.Element {
  const app = useApp();
  const [view, setView] = createSignal<View>('list');
  const [newTitle, setNewTitle] = createSignal('');
  const [mailId, setMailId] = createSignal('');
  const [eventId, setEventId] = createSignal('');
  const [dragId, setDragId] = createSignal<Id | null>(null);

  onMount(() => {
    void loadCatalog('tasks');
    void app.loadTasks();
  });

  /** The list a new/converted task lands in: the focused list, else the first. */
  function targetList(): Id {
    return app.selectedListId() ?? app.taskLists()[0]?.id ?? '';
  }

  async function addTask(e: Event): Promise<void> {
    e.preventDefault();
    const title = newTitle().trim();
    if (title === '') return;
    setNewTitle('');
    await app.createTask({ listId: targetList(), title });
  }

  async function convertMail(): Promise<void> {
    const id = mailId().trim();
    if (id === '') return;
    setMailId('');
    await app.convertMailToTask(id, targetList(), { title: t('tasks-followup-mail', { id }) });
  }

  async function convertEvent(): Promise<void> {
    const id = eventId().trim();
    if (id === '') return;
    setEventId('');
    await app.convertEventToTask(id, targetList(), { title: t('tasks-prep-event', { id }) });
  }

  function selectAll(): void {
    setView('list');
    void app.selectList(null);
  }

  function selectList(id: Id): void {
    setView('list');
    void app.selectList(id);
  }

  /** Drop `dragId` before `targetId` in the My Day order (client-side reorder). */
  function dropOn(targetId: Id): void {
    const src = dragId();
    setDragId(null);
    if (src === null || src === targetId) return;
    const ids = app.myDayTasks().map((t) => t.id);
    const from = ids.indexOf(src);
    const to = ids.indexOf(targetId);
    if (from < 0 || to < 0) return;
    ids.splice(from, 1);
    ids.splice(to, 0, src);
    app.reorderMyDay(ids);
  }

  const listActive = (id: Id | null): boolean =>
    view() === 'list' && app.selectedListId() === id;

  return (
    <section class={css.layout} aria-label={t('tasks-title')} data-module="tasks">
      <nav class={css.sidebar} aria-label={t('tasks-lists')}>
        <button
          type="button"
          class={css.navButton}
          aria-current={listActive(null)}
          onClick={selectAll}
        >
          {t('tasks-all')}
        </button>
        <button
          type="button"
          class={css.navButton}
          aria-current={view() === 'myDay'}
          onClick={() => setView('myDay')}
        >
          {t('tasks-my-day')}
        </button>

        <h2 class={css.sidebarHeading}>{t('tasks-lists-heading')}</h2>
        <For each={app.taskLists()} fallback={<p class={css.meta}>{t('tasks-no-lists')}</p>}>
          {(list) => (
            <button
              type="button"
              class={css.navButton}
              aria-current={listActive(list.id)}
              onClick={() => selectList(list.id)}
            >
              <span class={css.colorDot} style={{ background: list.color }} aria-hidden="true" />
              <span class={css.title}><bdi>{list.name}</bdi></span>
            </button>
          )}
        </For>
      </nav>

      <div class={css.main}>
        <form class={css.addForm} onSubmit={addTask}>
          <input
            class={css.input}
            type="text"
            aria-label={t('tasks-new-title')}
            placeholder={t('tasks-add-placeholder')}
            value={newTitle()}
            onInput={(e) => setNewTitle(e.currentTarget.value)}
          />
          <button class={css.button} type="submit">{t('common-add')}</button>
        </form>

        <Show
          when={view() === 'myDay'}
          fallback={<TaskTree roots={app.rootTasks()} />}
        >
          <ol class={css.taskList} aria-label={t('tasks-my-day')}>
            <For
              each={app.myDayTasks()}
              fallback={<li class={css.empty}>{t('tasks-my-day-empty')}</li>}
            >
              {(task) => (
                <li
                  data-task-id={task.id}
                  draggable="true"
                  onDragStart={() => setDragId(task.id)}
                  onDragOver={(e) => e.preventDefault()}
                  onDrop={() => dropOn(task.id)}
                >
                  <TaskItem task={task} />
                </li>
              )}
            </For>
          </ol>
        </Show>

        <div class={css.convert}>
          <span class={css.meta}>{t('tasks-convert')}</span>
          <input
            class={css.input}
            type="text"
            aria-label={t('tasks-mail-id')}
            placeholder={t('tasks-mail-id-placeholder')}
            value={mailId()}
            onInput={(e) => setMailId(e.currentTarget.value)}
          />
          <button class={css.button} type="button" onClick={convertMail}>
            {t('tasks-mail-to-task')}
          </button>
          <input
            class={css.input}
            type="text"
            aria-label={t('tasks-event-id')}
            placeholder={t('tasks-event-id-placeholder')}
            value={eventId()}
            onInput={(e) => setEventId(e.currentTarget.value)}
          />
          <button class={css.button} type="button" onClick={convertEvent}>
            {t('tasks-event-to-task')}
          </button>
        </div>
      </div>
    </section>
  );
}

/** The root task list of the focused list, each with its subtasks nested. */
function TaskTree(props: { roots: Task[] }): JSX.Element {
  return (
    <ul class={css.taskList} aria-label={t('tasks-list-label')}>
      <For each={props.roots} fallback={<li class={css.empty}>{t('tasks-empty-list')}</li>}>
        {(task) => (
          <li data-task-id={task.id}>
            <TaskItem task={task} />
            <Subtasks parentId={task.id} />
          </li>
        )}
      </For>
    </ul>
  );
}

/** Subtasks of `parentId` (RELATED-TO children), rendered indented. */
function Subtasks(props: { parentId: Id }): JSX.Element {
  const app = useApp();
  const kids = (): Task[] => app.subtasksOf(props.parentId);
  return (
    <Show when={kids().length > 0}>
      <ul class={css.subtasks} aria-label={t('tasks-subtasks')}>
        <For each={kids()}>
          {(task) => (
            <li data-task-id={task.id}>
              <TaskItem task={task} />
            </li>
          )}
        </For>
      </ul>
    </Show>
  );
}

/** One task row: complete toggle + title + schedule/priority meta. */
function TaskItem(props: { task: Task }): JSX.Element {
  const app = useApp();
  const done = (): boolean => isDone(props.task);
  return (
    <div class={`${css.row} ${done() ? css.rowDone : ''}`}>
      <input
        type="checkbox"
        class={css.checkbox}
        checked={done()}
        aria-label={done() ? t('tasks-reopen', { title: props.task.title }) : t('tasks-complete', { title: props.task.title })}
        onChange={() => void app.toggleComplete(props.task.id)}
      />
      <span class={css.title}><bdi>{props.task.title}</bdi></span>
      <Show when={props.task.priority >= 1 && props.task.priority <= 4}>
        <span class={css.priorityHigh} aria-label={t('tasks-high-priority')}>!</span>
      </Show>
      <Show when={scheduleLabel(props.task) !== ''}>
        <span class={css.meta}>{scheduleLabel(props.task)}</span>
      </Show>
    </div>
  );
}
