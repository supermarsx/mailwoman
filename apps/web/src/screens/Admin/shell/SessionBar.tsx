import {formatDateTime as regionalDateTime} from '../../../i18n/format.ts';
// The admin session bar: what is left of the admin session, along the bottom edge.
//
// It counts down to whichever of the two server deadlines comes first (the idle
// timeout, or the absolute limit set at sign-in) and shows the other in a popover.
// All timing logic is `sessionBar.ts`; this file is markup, wording and the
// polite announcements, which are made at the thresholds only, never per tick.

import { Show, createEffect, createSignal, createUniqueId, onCleanup, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { registerToastObstacle } from '../../../components/toast/toast.ts';
import { Icon } from './icons.tsx';
import {
  createSessionBar,
  formatClock,
  splitLong,
  type SessionBarApi,
  type SessionBarController,
  type SessionBarMode,
} from './sessionBar.ts';
import { srOnly } from './shell.css.ts';
import * as css from './sessionBar.css.ts';

export interface SessionBarProps {
  api: SessionBarApi;
  mode: SessionBarMode;
  /** The console has ended the session: go to the sign-in gate (the route stays in the address). */
  onExpired: () => void;
  /** Tells the frame whether the bar is on screen, so it can reserve its height. */
  onVisible?: (visible: boolean) => void;
  now?: () => number;
  isHidden?: () => boolean;
  /** Test hook: receives the controller. */
  controller?: (controller: SessionBarController) => void;
}

/** `11h 32m`, or `28:41` under an hour. */
function longTime(seconds: number): string {
  const parts = splitLong(seconds);
  return parts ? t('admin-session-bar-hm', { hours: parts.hours, minutes: parts.minutes }) : formatClock(seconds);
}

export function SessionBar(props: SessionBarProps): JSX.Element {
  const bar = createSessionBar({
    api: props.api,
    onExpired: () => props.onExpired(),
    enabled: () => props.mode !== 'hidden',
    ...(props.now ? { now: props.now } : {}),
    ...(props.isHidden ? { isHidden: props.isHidden } : {}),
  });
  props.controller?.(bar);
  const [open, setOpen] = createSignal(false);
  const popId = createUniqueId();
  let root: HTMLDivElement | undefined;

  const shown = (): boolean => {
    if (props.mode === 'hidden' || bar.level() === null) return false;
    return props.mode === 'always' || bar.level() !== 'normal';
  };
  createEffect(() => props.onVisible?.(shown()));
  onCleanup(() => props.onVisible?.(false));
  createEffect(() => root?.style.setProperty('--admin-bar-fraction', bar.fraction().toFixed(4)));
  createEffect(() => { if (!shown()) setOpen(false); });

  // Escape closes the details; a click elsewhere does too.
  createEffect(() => {
    if (!open()) return;
    const onKey = (e: KeyboardEvent): void => { if (e.key === 'Escape') setOpen(false); };
    const onDown = (e: Event): void => { if (root && !root.contains(e.target as Node)) setOpen(false); };
    document.addEventListener('keydown', onKey);
    document.addEventListener('pointerdown', onDown);
    onCleanup(() => { document.removeEventListener('keydown', onKey); document.removeEventListener('pointerdown', onDown); });
  });

  const primaryText = (): string => {
    const s = bar.seconds() ?? 0;
    return bar.primary() === 'idle' ? t('admin-session-bar-idle', { time: formatClock(s) }) : t('admin-session-bar-absolute', { time: longTime(s) });
  };
  const announcement = (): string => {
    switch (bar.announcement()) {
      case 'warning': return t('admin-session-bar-announce-warning');
      case 'critical': return t('admin-session-bar-announce-critical');
      case 'expired': return t('admin-session-bar-announce-expired');
      default: return '';
    }
  };
  const signedIn = (): string => {
    const at = bar.timing()?.signedInAt;
    if (at === undefined) return '';
    try { return regionalDateTime(at*1000); }
    catch { return regionalDateTime(at*1000); }
  };

  return (
    <>
      {/* The announcement region exists even while the bar is hidden by the preference's
          "warn only" mode, so the first warning is spoken; it changes only at thresholds. */}
      <div class={srOnly} role="status" aria-live="polite" aria-atomic="true" data-testid="admin-session-announce">
        {announcement()}
      </div>
      <Show when={shown()}>
        <div ref={(el) => { root = el; onCleanup(registerToastObstacle(el)); }} class={css.bar} role="group" aria-label={t('admin-session-bar-label')} data-testid="admin-session-bar" data-level={bar.level()} data-primary={bar.primary()}>
          <span class={css.sliver} aria-hidden="true" data-testid="admin-session-bar-sliver" />
          <Show
            when={bar.level() !== 'expired'}
            fallback={
              <>
                <Icon name="alert" />
                <span class={css.label} data-testid="admin-session-bar-text">{t('admin-session-bar-expired')}</span>
                <span class={css.spacer} />
                <button type="button" class={css.action} onClick={() => bar.expireNow()}>{t('admin-session-bar-sign-in')}</button>
              </>
            }
          >
            <button
              type="button"
              class={css.detailsButton}
              aria-expanded={open()}
              aria-controls={popId}
              title={t('admin-session-bar-details')}
              onClick={() => setOpen(!open())}
            >
              <Icon name={bar.level() === 'normal' ? 'clock' : 'alert'} />
              <span class={css.label}>
                <span class={css.longLabel} data-testid="admin-session-bar-text">{primaryText()}</span>
                <span class={css.shortLabel} aria-hidden="true">{formatClock(bar.seconds() ?? 0)}</span>
              </span>
            </button>
            <span class={css.spacer} />
            <Show when={(bar.level() === 'warning' || bar.level() === 'critical') && bar.canExtend()}>
              <button type="button" class={css.action} disabled={bar.busy()} onClick={() => void bar.extend()}>
                {t('admin-session-bar-extend')}
              </button>
            </Show>
            <Show when={open()}>
              <div id={popId} class={css.popover} data-testid="admin-session-bar-details">
                <div class={css.popoverRow}>
                  <span class={css.popoverTerm}>{t('admin-session-bar-row-idle')}</span>
                  <span>{formatClock(bar.idleSeconds() ?? 0)}</span>
                </div>
                <div class={css.popoverRow}>
                  <span class={css.popoverTerm}>{t('admin-session-bar-row-absolute')}</span>
                  <span>{longTime(bar.absoluteSeconds() ?? 0)}</span>
                </div>
                <div class={css.popoverRow}>
                  <span class={css.popoverTerm}>{t('admin-session-bar-row-signed-in')}</span>
                  <span>{signedIn()}</span>
                </div>
                <p class={css.popoverNote}>{t('admin-session-bar-note')}</p>
              </div>
            </Show>
          </Show>
        </div>
      </Show>
    </>
  );
}
