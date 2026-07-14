// Distribution-group expand-before-send (SPEC §13, plan §3 e7): "who is actually
// in this?" Before a message addressed to a distribution group is sent, the composer
// can expand the group to its real leaf recipients so the sender sees exactly who
// will receive it. The flatten is server-side (`mw-directory::expand_group`, recursive,
// leaves only); this control renders + lets the sender replace the group with its
// members. EXPORTED for e14 to wire into the composer.

import { createSignal, For, Show, createMemo, type JSX } from 'solid-js';
import { DirectoryService, type Fetcher } from './service.ts';
import type { GalEntry } from './index.ts';
import * as css from './styles.css.ts';

export interface GroupExpandProps {
  /** The distribution group the user addressed. */
  group: GalEntry;
  /** The sender expanded the group into these concrete recipients. */
  onExpand: (members: GalEntry[]) => void;
  fetcher?: Fetcher;
  service?: DirectoryService;
}

export function GroupExpand(props: GroupExpandProps): JSX.Element {
  const service = createMemo(() => props.service ?? new DirectoryService(props.fetcher));
  const [members, setMembers] = createSignal<GalEntry[] | null>(null);
  const [error, setError] = createSignal('');
  const [busy, setBusy] = createSignal(false);

  async function expand(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const found = await service().expandGroup(props.group.dn);
      setMembers(found);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'could not expand the group');
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.expandPanel} data-testid="group-expand" aria-label={`Members of ${props.group.displayName}`}>
      <p class={css.meta}>
        <strong>{props.group.displayName}</strong> is a distribution group.
      </p>
      <Show
        when={members()}
        fallback={
          <button type="button" class={css.button} disabled={busy()} onClick={() => void expand()}>
            Who is actually in this?
          </button>
        }
      >
        {(list) => (
          <>
            <p class={css.meta} data-testid="member-count">
              {list().length} {list().length === 1 ? 'recipient' : 'recipients'}
            </p>
            <ul class={css.memberList}>
              <For each={list()}>
                {(m) => (
                  <li class={css.member}>
                    <span class={css.optName}>{m.displayName}</span>
                    <span class={css.optMail}>{m.mail}</span>
                  </li>
                )}
              </For>
            </ul>
            <button type="button" class={css.button} onClick={() => props.onExpand(list())}>
              Replace group with {list().length} {list().length === 1 ? 'recipient' : 'recipients'}
            </button>
          </>
        )}
      </Show>
      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

export default GroupExpand;
