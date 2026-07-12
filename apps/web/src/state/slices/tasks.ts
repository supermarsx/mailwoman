// Tasks store slice (plan §2.5, §3 e0 → filled by e5). Frozen seam composed into
// `AppState`; e5 fills the signals + actions over the `Task/*` surface (mock
// until e10), incl. the My Day / Today view filter.

import { createSignal, type Accessor } from 'solid-js';
import type { Task } from '../../api/pim-types.ts';
import type { SliceContext } from './context.ts';

export interface TasksSlice {
  tasks: Accessor<Task[]>;
  /** Load the account's task lists + tasks (e5 fills). */
  loadTasks(): Promise<void>;
}

export function createTasksSlice(_ctx: SliceContext): TasksSlice {
  const [tasks] = createSignal<Task[]>([]);

  return {
    tasks,
    loadTasks: () => Promise.resolve(),
  };
}
