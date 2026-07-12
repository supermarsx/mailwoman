// Tasks module placeholder (plan §2.5, §3 e0 → filled by e5). e5 builds task
// lists, task create/edit (due/start/priority/recurrence/percent-complete/
// subtasks), the My Day / Today view, and the mail→task / event→task entry
// points over `state/slices/tasks.ts` and the frozen `Task/*` surface.

import type { JSX } from 'solid-js';

export function TasksModule(): JSX.Element {
  return (
    <section aria-label="Tasks" data-module="tasks">
      <h1>Tasks</h1>
      <p>The tasks module mounts here (e5).</p>
    </section>
  );
}
