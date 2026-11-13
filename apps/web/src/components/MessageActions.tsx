import {configuredSnoozePresets,resolveSnooze,snoozePresetLabel,snoozeZone,snoozeWeekStart,type SnoozePreset} from '../state/snooze.ts';
import {requestCustomSnooze} from './SnoozeDialog.tsx';
import { Icon, tagIconName } from './Icon.tsx';
import { createSignal, For, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import {formatDateTime,t} from '../i18n/index.ts';
import { rowActionsMode, type RowActionsMode } from '../state/mailFeatures.ts';
import * as a11y from './mailA11y.css.ts';
import * as css from './messageRow.css.ts';
import type { Email } from '../api/jmap-types.ts';
import { ACTION_ICONS, snoozePresetIcon } from '../actionIcons.ts';

// The per-row action cluster (plan §1.5): pin, snooze (with presets), label
// (tag registry), follow-up, archive, delete. Each action goes through the mail
// slice. Pin, snooze, label, follow-up, archive and delete each raise the
// shared undo toast; Unsnooze does not.
//
// Layout is in messageRow.css.ts. With a hovering pointer the cluster appears
// over the row on hover or focus; without one (touch, the phone layout) it is
// opened and closed by the "More actions" button, which is why that button is
// a sibling of `.msg-actions` and not inside it. Opening it that way moves
// focus to the first action; closing it hands focus back to the button.
//
// `mail.rowActionsMode` picks the presentation: 'open' (the above), 'hover'
// (the same reveal, but compact: flag, archive and delete plus a More button
// that expands pin, snooze and label; on touch it behaves like 'click'),
// 'click' (the toggle on every pointer type, cluster or context menu opens on
// click) and 'off' (no row controls; only the follow-up mark stays).
// The "follow-up" button sets a reminder for 24 hours from now (`followUpAt`);
// it does not set `$flagged`. A message with a follow-up carries a small mark
// on its row, outside the cluster, so the state shows while the cluster is shut.

/** Snooze presets → absolute ISO times, computed at click. The `labelId` is a
 *  mail catalog id resolved with `t()` at render (kept out of the pure time math
 *  so this stays trivially testable). */
type SnoozeChoice=SnoozePreset&{labelId:string;at:string};
let cachedChoices:{rules:SnoozePreset[];minute:number;zone:string;weekStart:number;rows:SnoozeChoice[]}|undefined;
export function snoozePresets(now=new Date()):SnoozeChoice[]{
  const rules=configuredSnoozePresets(),minute=Math.floor(now.getTime()/60000),zone=snoozeZone(),weekStart=snoozeWeekStart();
  if(cachedChoices?.rules===rules&&cachedChoices.minute===minute&&cachedChoices.zone===zone&&cachedChoices.weekStart===weekStart)return cachedChoices.rows;
  const rows=rules.flatMap(preset=>{if(!preset.enabled)return [];const at=resolveSnooze(preset,now,zone,weekStart);return at?[{...preset,labelId:preset.id==='later'?'mail-snooze-later':preset.kind==='hours'?'snooze-kind-hours':preset.kind==='days'?'snooze-kind-days':`mail-snooze-${preset.kind}`,at}]:[];});
  cachedChoices={rules,minute,zone,weekStart,rows};return rows;
}

export function MessageActions(props: { email: Email; onMore?:(event:MouseEvent)=>void; contextOpen?:boolean }): JSX.Element {
  const app = useApp();
  const [menu, setMenu] = createSignal<'none' | 'snooze' | 'tag'>('none');
  // Whether the "More actions" toggle has the cluster open (touch / phone).
  const [open, setOpen] = createSignal(false);
  // 'hover' mode: whether the More button has expanded the secondary actions.
  const [expanded, setExpanded] = createSignal(false);
  // Where hovering cannot reveal anything, 'hover' means 'click'.
  const [touch, setTouch] = createSignal(false);
  onMount(() => {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return;
    const query = window.matchMedia('(hover: none), (max-width: 760px)');
    setTouch(query.matches);
    const listener = (): void => { setTouch(query.matches); };
    query.addEventListener?.('change', listener);
    onCleanup(() => { query.removeEventListener?.('change', listener); });
  });
  const mode = (): RowActionsMode => {
    const configured = rowActionsMode();
    return configured === 'hover' && touch() ? 'click' : configured;
  };
  const compact = () => mode() === 'hover';

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
    setExpanded(false);
  }

  let wrapper: HTMLDivElement | undefined;
  let toggleEl: HTMLButtonElement | undefined;
  let clusterEl: HTMLDivElement | undefined;

  /** Open the cluster from its toggle and put focus on its first action. */
  function openFromToggle(): void {
    setOpen(true);
    clusterEl?.querySelector<HTMLElement>('button')?.focus();
  }

  /** After an action taken from the toggled cluster: close it and hand focus
   *  back to the toggle. Does nothing when the cluster was shown by hover. */
  function closeToToggle(): void {
    if (!open()) return;
    closeAll();
    toggleEl?.focus();
  }

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
    } else if (expanded()) {
      setExpanded(false);
      wrapper?.querySelector<HTMLElement>('[data-testid="msg-actions-expand"]')?.focus();
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
      <Show when={mode() !== 'off'}>
      <button
        type="button"
        ref={toggleEl}
        class={`msg-more__toggle ${css.moreToggle} ${a11y.focusable}`}
        classList={{ [css.moreToggleAlways]: mode() === 'click' }}
        aria-label={t('mail-more-actions')}
        aria-expanded={props.onMore?props.contextOpen:open()}
        aria-haspopup={props.onMore?'menu':undefined}
        data-testid="msg-more-toggle"
        onClick={event => props.onMore ? props.onMore(event) : (open() ? closeAll() : openFromToggle())}
      >
        <Icon name={open() ? 'close' : 'more'} />
      </button>
      </Show>

      <Show when={props.email.keywords?.['mw-followup-completed']===true}><span class={css.followUpMark} role="img" aria-label={t('mail-follow-up-finished')} data-testid="msg-finished-mark"><Icon name="check"/></span></Show>
      <Show when={hasFollowUp()}>
        <span
          class={`msg-followup ${css.followUpMark}`}
          role="img"
          aria-label={t('mail-flag')}
          data-testid="msg-followup-mark"
        >
          <Icon name={ACTION_ICONS.followUp} />
        </span>
      </Show>

      <Show when={mode() !== 'off'}>
      <div
        ref={clusterEl}
        class={`msg-actions ${css.actions} ${css.actionsFade}`}
        classList={{
          [css.actionsOpen]: open() || menu() !== 'none',
          [css.actionsOnClick]: mode() === 'click',
          [css.actionsOnClickOpen]: mode() === 'click' && (open() || menu() !== 'none'),
        }}
        data-mode={mode()}
        role="group"
        aria-label={t('mail-more-actions')}
      >
        <Show when={compact()}>
          <button
            type="button"
            class={`msg-actions__btn ${css.actionBtn} ${css.actionExpand} ${a11y.iconButton}`}
            aria-label={t('mail-row-actions-expand')}
            aria-expanded={expanded()}
            aria-haspopup="true"
            data-testid="msg-actions-expand"
            onClick={() => setExpanded((v) => !v)}
          >
            <Icon name="more" />
          </button>
        </Show>
        <button
          type="button"
          class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
          classList={{ [css.actionSecondary]: compact() && !expanded() }}
          aria-label={pinned() ? t('mail-unpin') : t('mail-pin')}
          aria-pressed={pinned()}
          onClick={() => void app.pinMessage(id(), !pinned())}
        >
          <Icon name={pinned() ? ACTION_ICONS.unpin : ACTION_ICONS.pin} />
        </button>

        <div class={`msg-actions__wrap ${css.menuWrap}`} classList={{ [css.actionSecondary]: compact() && !expanded() }}>
          <button
            type="button"
            class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
            aria-label={t('mail-snooze')}
            aria-haspopup="menu"
            aria-expanded={menu() === 'snooze'}
            onClick={() => toggleMenu('snooze')}
          >
            <Icon name={ACTION_ICONS.snooze} />
          </button>
          <Show when={menu() === 'snooze'}>
            <div class={`msg-menu ${css.menu}`} role="menu" aria-label={t('mail-snooze-menu')}>
              <button type="button" role="menuitem" class={`msg-menu__item ${css.menuItem} ${a11y.focusable}`} onClick={()=>{const messageId=id();closeAll();requestCustomSnooze(until=>app.snoozeMessage(messageId,until));}}><Icon name="snoozePick"/>{t('snooze-custom')}</button>
              <For each={snoozePresets()}>
                {(p) => (
                  <button
                    type="button"
                    role="menuitem"
                    title={formatDateTime(p.at)}
                    class={`msg-menu__item ${css.menuItem} ${a11y.focusable}`}
                    onClick={() => {
                      closeAll();
                      const at=resolveSnooze(p);if(at)void app.snoozeMessage(id(),at);
                    }}
                  >
                    <Icon name={snoozePresetIcon(p.labelId)} />{snoozePresetLabel(p)}
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
                  <Icon name={ACTION_ICONS.unsnooze} />{t('mail-unsnooze')}
                </button>
              </Show>
            </div>
          </Show>
        </div>

        <div class={`msg-actions__wrap ${css.menuWrap}`} classList={{ [css.actionSecondary]: compact() && !expanded() }}>
          <button
            type="button"
            class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
            aria-label={t('mail-label')}
            aria-haspopup="menu"
            aria-expanded={menu() === 'tag'}
            onClick={() => toggleMenu('tag')}
          >
            <Icon name="tag" />
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
                      <Icon name={tagIconName(tag.icon)} /> {tag.name}
                      <Show when={on()}> <Icon name="check" /></Show>
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
          onClick={() => {
            const at = hasFollowUp() ? null : new Date(Date.now() + 86_400_000).toISOString();
            closeToToggle();
            void app.setFollowUp(id(), at);
          }}
        >
          <Icon name={ACTION_ICONS.followUp} />
        </button>

        <button
          type="button"
          class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
          aria-label={t('mail-archive')}
          onClick={() => void app.archiveMessage(id())}
        >
          <Icon name={ACTION_ICONS.archive} />
        </button>
        <button
          type="button"
          class={`msg-actions__btn ${css.actionBtn} ${a11y.iconButton}`}
          aria-label={t('mail-delete')}
          onClick={() => void app.trashMessage(id())}
        >
          <Icon name="delete" />
        </button>
      </div>
      </Show>
    </div>
  );
}
