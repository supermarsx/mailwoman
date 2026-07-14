import { createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import type { Email } from '../api/jmap-types.ts';

// The per-row hover/action cluster (plan §1.5): pin, snooze (with presets),
// label (tag registry), follow-up flag, archive, delete. Each action goes
// through the mail slice, so every one is reversible via the shared undo toast.

/** Snooze presets → absolute ISO times, computed at click. The `labelId` is a
 *  mail catalog id resolved with `t()` at render (kept out of the pure time math
 *  so this stays trivially testable). */
export function snoozePresets(now = new Date()): { labelId: string; at: string }[] {
  const laterToday = new Date(now.getTime() + 3 * 3_600_000);
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  tomorrow.setHours(9, 0, 0, 0);
  const nextWeek = new Date(now);
  nextWeek.setDate(nextWeek.getDate() + 7);
  nextWeek.setHours(9, 0, 0, 0);
  return [
    { labelId: 'mail-snooze-later', at: laterToday.toISOString() },
    { labelId: 'mail-snooze-tomorrow', at: tomorrow.toISOString() },
    { labelId: 'mail-snooze-next-week', at: nextWeek.toISOString() },
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
        class={`msg-actions__btn ${a11y.iconButton}`}
        classList={{ 'msg-actions__btn--on': pinned() }}
        aria-label={pinned() ? t('mail-unpin') : t('mail-pin')}
        aria-pressed={pinned()}
        onClick={() => void app.pinMessage(id(), !pinned())}
      >
        📌
      </button>

      <div class="msg-actions__wrap">
        <button
          type="button"
          class={`msg-actions__btn ${a11y.iconButton}`}
          aria-label={t('mail-snooze')}
          aria-haspopup="menu"
          onClick={() => toggleMenu('snooze')}
        >
          🕒
        </button>
        <Show when={menu() === 'snooze'}>
          <div class="msg-menu" role="menu" aria-label={t('mail-snooze-menu')}>
            <For each={snoozePresets()}>
              {(p) => (
                <button
                  type="button"
                  role="menuitem"
                  class={`msg-menu__item ${a11y.focusable}`}
                  onClick={() => {
                    setMenu('none');
                    void app.snoozeMessage(id(), p.at);
                  }}
                >
                  {t(p.labelId)}
                </button>
              )}
            </For>
            <Show when={props.email.snoozedUntil != null}>
              <button
                type="button"
                role="menuitem"
                class={`msg-menu__item ${a11y.focusable}`}
                onClick={() => {
                  setMenu('none');
                  void app.unsnoozeMessage(id());
                }}
              >
                {t('mail-unsnooze')}
              </button>
            </Show>
          </div>
        </Show>
      </div>

      <div class="msg-actions__wrap">
        <button
          type="button"
          class={`msg-actions__btn ${a11y.iconButton}`}
          aria-label={t('mail-label')}
          aria-haspopup="menu"
          onClick={() => toggleMenu('tag')}
        >
          🏷️
        </button>
        <Show when={menu() === 'tag'}>
          <div class="msg-menu" role="menu" aria-label={t('mail-labels-menu')}>
            <For each={app.tags()}>
              {(tag) => {
                const on = () => hasKeyword(tag.id);
                return (
                  <button
                    type="button"
                    role="menuitemcheckbox"
                    aria-checked={on()}
                    class={`msg-menu__item ${a11y.focusable}`}
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
        class={`msg-actions__btn ${a11y.iconButton}`}
        classList={{ 'msg-actions__btn--on': hasFollowUp() }}
        aria-label={hasFollowUp() ? t('mail-clear-flag') : t('mail-flag')}
        aria-pressed={hasFollowUp()}
        onClick={() =>
          void app.setFollowUp(id(), hasFollowUp() ? null : new Date(Date.now() + 86_400_000).toISOString())
        }
      >
        🚩
      </button>

      <button
        type="button"
        class={`msg-actions__btn ${a11y.iconButton}`}
        aria-label={t('mail-archive')}
        onClick={() => void app.archiveMessage(id())}
      >
        🗄️
      </button>
      <button
        type="button"
        class={`msg-actions__btn ${a11y.iconButton}`}
        aria-label={t('mail-delete')}
        onClick={() => void app.trashMessage(id())}
      >
        🗑️
      </button>
    </div>
  );
}
