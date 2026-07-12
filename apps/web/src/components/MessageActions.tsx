import { createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import type { Email } from '../api/jmap-types.ts';

// The per-row hover/action cluster (plan §1.5): pin, snooze (with presets),
// label (tag registry), follow-up flag, archive, delete. Each action goes
// through the mail slice, so every one is reversible via the shared undo toast.

/** Snooze presets → absolute ISO times, computed at click. */
export function snoozePresets(now = new Date()): { label: string; at: string }[] {
  const laterToday = new Date(now.getTime() + 3 * 3_600_000);
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  tomorrow.setHours(9, 0, 0, 0);
  const nextWeek = new Date(now);
  nextWeek.setDate(nextWeek.getDate() + 7);
  nextWeek.setHours(9, 0, 0, 0);
  return [
    { label: 'Later today', at: laterToday.toISOString() },
    { label: 'Tomorrow', at: tomorrow.toISOString() },
    { label: 'Next week', at: nextWeek.toISOString() },
  ];
}

export function MessageActions(props: { email: Email }): JSX.Element {
  const app = useApp();
  const [menu, setMenu] = createSignal<'none' | 'snooze' | 'tag'>('none');

  const id = () => props.email.id;
  const pinned = () => props.email.pinned === true;
  const hasFollowUp = () => props.email.followUpAt != null;
  const hasKeyword = (kw: string) => props.email.keywords?.[kw] === true;

  function toggleMenu(which: 'snooze' | 'tag'): void {
    setMenu((m) => (m === which ? 'none' : which));
  }

  return (
    <div class="msg-actions" onClick={(e) => e.stopPropagation()}>
      <button
        type="button"
        class="msg-actions__btn"
        classList={{ 'msg-actions__btn--on': pinned() }}
        aria-label={pinned() ? 'Unpin' : 'Pin'}
        aria-pressed={pinned()}
        onClick={() => void app.pinMessage(id(), !pinned())}
      >
        📌
      </button>

      <div class="msg-actions__wrap">
        <button
          type="button"
          class="msg-actions__btn"
          aria-label="Snooze"
          aria-haspopup="menu"
          onClick={() => toggleMenu('snooze')}
        >
          🕒
        </button>
        <Show when={menu() === 'snooze'}>
          <div class="msg-menu" role="menu" aria-label="Snooze until">
            <For each={snoozePresets()}>
              {(p) => (
                <button
                  type="button"
                  role="menuitem"
                  class="msg-menu__item"
                  onClick={() => {
                    setMenu('none');
                    void app.snoozeMessage(id(), p.at);
                  }}
                >
                  {p.label}
                </button>
              )}
            </For>
            <Show when={props.email.snoozedUntil != null}>
              <button
                type="button"
                role="menuitem"
                class="msg-menu__item"
                onClick={() => {
                  setMenu('none');
                  void app.unsnoozeMessage(id());
                }}
              >
                Unsnooze
              </button>
            </Show>
          </div>
        </Show>
      </div>

      <div class="msg-actions__wrap">
        <button
          type="button"
          class="msg-actions__btn"
          aria-label="Label"
          aria-haspopup="menu"
          onClick={() => toggleMenu('tag')}
        >
          🏷️
        </button>
        <Show when={menu() === 'tag'}>
          <div class="msg-menu" role="menu" aria-label="Labels">
            <For each={app.tags()}>
              {(tag) => {
                const on = () => hasKeyword(tag.id);
                return (
                  <button
                    type="button"
                    role="menuitemcheckbox"
                    aria-checked={on()}
                    class="msg-menu__item"
                    onClick={() => {
                      setMenu('none');
                      if (on()) void app.removeTag(id(), tag.id);
                      else void app.applyTag(id(), tag.id);
                    }}
                  >
                    <span class="msg-menu__swatch" style={{ 'background-color': tag.color }} />
                    {tag.icon} {tag.name}
                    <Show when={on()}> ✓</Show>
                  </button>
                );
              }}
            </For>
          </div>
        </Show>
      </div>

      <button
        type="button"
        class="msg-actions__btn"
        classList={{ 'msg-actions__btn--on': hasFollowUp() }}
        aria-label={hasFollowUp() ? 'Clear follow-up' : 'Flag for follow-up'}
        aria-pressed={hasFollowUp()}
        onClick={() =>
          void app.setFollowUp(id(), hasFollowUp() ? null : new Date(Date.now() + 86_400_000).toISOString())
        }
      >
        🚩
      </button>

      <button
        type="button"
        class="msg-actions__btn"
        aria-label="Archive"
        onClick={() => void app.archiveMessage(id())}
      >
        🗄️
      </button>
      <button
        type="button"
        class="msg-actions__btn"
        aria-label="Delete"
        onClick={() => void app.trashMessage(id())}
      >
        🗑️
      </button>
    </div>
  );
}
