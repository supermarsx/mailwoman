import { createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import * as css from './messageRow.css.ts';
import type { Email } from '../api/jmap-types.ts';

// The per-row action cluster (plan §1.5): pin, snooze (with presets), label
// (tag registry), follow-up, archive, delete. Each action goes through the mail
// slice. Pin, snooze, label, follow-up, archive and delete each raise the
// shared undo toast; Unsnooze does not.
//
// Layout is in messageRow.css.ts. With a hovering pointer the cluster appears
// over the row on hover or focus; without one (touch, the phone layout) it is
// opened and closed by the "More actions" button, which is why that button is
// a sibling of `.msg-actions` and not inside it. The "follow-up" button sets a
// reminder for 24 hours from now (`followUpAt`); it does not set `$flagged`.

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
  // Whether the "More actions" toggle has the cluster open (touch / phone).
  const [open, setOpen] = createSignal(false);

  const id = () => props.email.id;
  const pinned = () => props.email.pinned === true;
  const hasFollowUp = () => props.email.followUpAt != null;
  const hasKeyword = (kw: string) => props.email.keywords?.[kw] === true;

  function toggleMenu(which: 'snooze' | 'tag'): void {
    setMenu((m) => (m === which ? 'none' : which));
  }

  function closeAll(): void {
    setMenu('none');
    setOpen(false);
  }

  let wrapper: HTMLDivElement | undefined;
  let toggleEl: HTMLButtonElement | undefined;

  /** Escape closes an open menu first, then the cluster, and puts focus back on
   *  the control that opened it. Not handled (so not swallowed) when nothing here
   *  is open. */
  function onKeyDown(e: KeyboardEvent): void {
    if (e.key !== 'Escape') return;
    if (menu() !== 'none') {
      const opener = wrapper?.querySelector<HTMLElement>('[aria-haspopup="menu"][aria-expanded="true"]');
      setMenu('none');
      opener?.focus();
    } else if (open()) {
      setOpen(false);
      toggleEl?.focus();
    } else {
      return;
    }
    e.preventDefault();
    e.stopPropagation();
  }

  /** Focus leaving the row's actions for somewhere else closes what was open. */
  function onFocusOut(e: FocusEvent): void {
    const next = e.relatedTarget;
    if (next instanceof Node && wrapper?.contains(next) === true) return;
    closeAll();
  }

  return (
    <div
      class={`msg-more ${css.more}`}
      ref={wrapper}
      onClick={(e) => e.stopPropagation()}
      onKeyDown={onKeyDown}
      onFocusOut={onFocusOut}
    >
      <button
        type="button"
        ref={toggleEl}
        class={`msg-more__toggle ${css.moreToggle} ${a11y.focusable}`}
        aria-label={t('mail-more-actions')}
        aria-expanded={open()}
        data-testid="msg-more-toggle"
        onClick={() => (open() ? closeAll() : setOpen(true))}
      >
        {open() ? '✕' : '⋯'}
      </button>

      <div
        class={`msg-actions ${css.actions}`}
        classList={{ [css.actionsOpen]: open() || menu() !== 'none' }}
        role="group"
        aria-label={t('mail-more-actions')}
      >
        <button
          type="button"
          class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
          aria-label={pinned() ? t('mail-unpin') : t('mail-pin')}
          aria-pressed={pinned()}
          onClick={() => void app.pinMessage(id(), !pinned())}
        >
          📌
        </button>

        <div class={`msg-actions__wrap ${css.menuWrap}`}>
          <button
            type="button"
            class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
            aria-label={t('mail-snooze')}
            aria-haspopup="menu"
            aria-expanded={menu() === 'snooze'}
            onClick={() => toggleMenu('snooze')}
          >
            🕒
          </button>
          <Show when={menu() === 'snooze'}>
            <div class={`msg-menu ${css.menu}`} role="menu" aria-label={t('mail-snooze-menu')}>
              <For each={snoozePresets()}>
                {(p) => (
                  <button
                    type="button"
                    role="menuitem"
                    class={`msg-menu__item ${css.menuItem} ${a11y.focusable}`}
                    onClick={() => {
                      closeAll();
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
                  class={`msg-menu__item ${css.menuItem} ${a11y.focusable}`}
                  onClick={() => {
                    closeAll();
                    void app.unsnoozeMessage(id());
                  }}
                >
                  {t('mail-unsnooze')}
                </button>
              </Show>
            </div>
          </Show>
        </div>

        <div class={`msg-actions__wrap ${css.menuWrap}`}>
          <button
            type="button"
            class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
            aria-label={t('mail-label')}
            aria-haspopup="menu"
            aria-expanded={menu() === 'tag'}
            onClick={() => toggleMenu('tag')}
          >
            🏷️
          </button>
          <Show when={menu() === 'tag'}>
            <div class={`msg-menu ${css.menu}`} role="menu" aria-label={t('mail-labels-menu')}>
              <For each={app.tags()}>
                {(tag) => {
                  const on = () => hasKeyword(tag.id);
                  return (
                    <button
                      type="button"
                      role="menuitemcheckbox"
                      aria-checked={on()}
                      class={`msg-menu__item ${css.menuItem} ${a11y.focusable}`}
                      onClick={() => {
                        closeAll();
                        if (on()) void app.removeTag(id(), tag.id);
                        else void app.applyTag(id(), tag.id);
                      }}
                    >
                      <span class={`msg-menu__swatch ${css.swatch}`} style={{ 'background-color': tag.color }} />
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
          class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
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
          class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
          aria-label={t('mail-archive')}
          onClick={() => void app.archiveMessage(id())}
        >
          🗄️
        </button>
        <button
          type="button"
          class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
          aria-label={t('mail-delete')}
          onClick={() => void app.trashMessage(id())}
        >
          🗑️
        </button>
      </div>
    </div>
  );
}
