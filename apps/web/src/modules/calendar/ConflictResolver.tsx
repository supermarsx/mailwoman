// Side-by-side conflict resolver (plan #7, SPEC §11.4). Lists the overlapping
// instance pairs the controller detected and, for the selected pair, compares
// the two events side by side and offers concrete resolutions:
//   • reschedule — move the later event to start when the earlier one ends;
//   • shorten    — trim the earlier event so it ends when the later one starts;
//   • tentative  — mark the later event tentative;
//   • double-book — keep both, but mark the later event's time as free so it no
//                   longer counts against free/busy;
//   • keep both  — accept the overlap as-is.
// Every resolution is a `CalendarEvent/set` update through the controller's
// `updateEvent`. The engine sends attendees nothing on an update (it emits an
// iTIP REQUEST only on create — `event_create`, `crates/mw-engine/src/pim/events.rs:145`),
// so the dialog makes no claim that anyone is notified.
//
// A free/busy grid shows busy/tentative/free time across the conflict day so
// the user can pick a clear slot: one row for the signed-in account, then one
// per other participant of the two events. `Calendar/freeBusy` is asked with
// their addresses as `principals` and tags each answer with the address
// (`calendar_free_busy`, `crates/mw-engine/src/pim/calendars.rs:361-494`). A
// participant the server could get no answer for is reported `unknown`, and
// their row says availability could not be retrieved instead of showing free.
//
// a11y (WCAG 2.2 AA): a focus-trapped `role=dialog` (Escape closes, focus is
// restored on close); the free/busy grid is a semantic table with row/column
// headers and per-cell text (status is never conveyed by color alone); every
// user-controlled title is bidi-isolated.

import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { t, isolate } from '../../i18n';
import type { CalendarEvent } from '../../api/pim-types.ts';
import type { CalendarController } from './controller.ts';
import type { FreeBusyBlock } from './api.ts';
import type { ConflictPair } from './types.ts';
import { otherParticipants, type OtherParticipant } from './participants.ts';
import { addDays, dateToLocal, formatFull, formatTime, instanceBound, localToDate, startOfDay } from './datetime.ts';
import { formatDuration, parseDuration } from './recurrence.ts';
import * as css from './calendar.css.ts';

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/** One resolved side of a conflict pair: the master + its concrete bounds. */
interface Side {
  event: CalendarEvent;
  start: Date;
  end: Date;
}

type FbStatus = 'busy' | 'tentative' | 'free';

export interface ConflictResolverProps {
  controller: CalendarController;
  onClose: () => void;
}

function endOf(ev: CalendarEvent): Date {
  return new Date(localToDate(ev.start).getTime() + parseDuration(ev.duration));
}

export function ConflictResolver(props: ConflictResolverProps): JSX.Element {
  const c = props.controller;
  const [selected, setSelected] = createSignal(0);
  const [busy, setBusy] = createSignal(false);
  const [failed, setFailed] = createSignal(false);
  // `null` = the free/busy query failed: the grid is withheld rather than drawn
  // as all-free.
  const [blocks, setBlocks] = createSignal<FreeBusyBlock[] | null>(null);
  const [loadingBusy, setLoadingBusy] = createSignal(false);

  // Keep the selection in range as pairs resolve away.
  createEffect(() => {
    const n = c.conflicts().length;
    if (n > 0 && selected() >= n) setSelected(n - 1);
  });

  /** The selected pair, resolved to its two masters ordered earliest-first. */
  const pair = createMemo<{ pair: ConflictPair; earlier: Side; later: Side } | null>(() => {
    const p = c.conflicts()[selected()];
    if (p === undefined) return null;
    const ea = c.masterById(p.a);
    const eb = c.masterById(p.b);
    if (ea === undefined || eb === undefined) return null;
    const sa: Side = { event: ea, start: localToDate(ea.start), end: endOf(ea) };
    const sb: Side = { event: eb, start: localToDate(eb.start), end: endOf(eb) };
    const [earlier, later] = sa.start <= sb.start ? [sa, sb] : [sb, sa];
    return { pair: p, earlier, later };
  });

  /** The other participants of the selected pair: one grid row each. */
  const people = createMemo<OtherParticipant[]>(() => {
    const pr = pair();
    if (pr === null) return [];
    return otherParticipants([pr.earlier.event.participants, pr.later.event.participants], c.identity());
  });

  /** The conflict day + the hour span the grid covers (union ± 1h, clamped). */
  const grid = createMemo(() => {
    const pr = pair();
    if (pr === null) return { day: startOfDay(new Date()), hours: [] as number[] };
    const day = startOfDay(pr.earlier.start);
    const startH = Math.max(0, Math.min(pr.earlier.start.getHours(), pr.later.start.getHours()) - 1);
    const endH = Math.min(24, Math.max(pr.earlier.end.getHours() + (pr.earlier.end.getMinutes() > 0 ? 1 : 0), pr.later.end.getHours() + (pr.later.end.getMinutes() > 0 ? 1 : 0)) + 1);
    const hours: number[] = [];
    for (let h = startH; h < endH; h += 1) hours.push(h);
    return { day, hours };
  });

  // Refetch busy time for the conflict day whenever the selected pair changes.
  // Only the latest request's answer is drawn: a slower answer for a pair that
  // is no longer selected must not replace it.
  let fbTicket = 0;
  createEffect(() => {
    const ticket = (fbTicket += 1);
    if (pair() === null) {
      setBlocks([]);
      setLoadingBusy(false);
      return;
    }
    setBlocks(null);
    setLoadingBusy(true);
    const day = grid().day;
    c.queryFreeBusy(
      day,
      addDays(day, 1),
      people().map((p) => p.address),
    ).then(
      (list) => { if (ticket === fbTicket) { setBlocks(list); setLoadingBusy(false); } },
      () => { if (ticket === fbTicket) { setBlocks(null); setLoadingBusy(false); } },
    );
  });

  /** Whether the server could get no availability for `address`. */
  function unknown(address: string): boolean {
    return (blocks() ?? []).some((b) => b.principal === address && b.status === 'unknown');
  }

  /**
   * One person's status for one hour of the conflict day: the account's own
   * (`address` null, the untagged blocks) or a participant's (the blocks tagged
   * with their address). Blocks are read as UTC instants. A merged block does
   * not say whether a floating or all-day event contributed to it (those are
   * stamped `Z` at their wall clock), so such an event's cells sit off by the
   * viewer's UTC offset.
   */
  function statusAt(hour: number, address: string | null = null): FbStatus {
    const g = grid();
    // The wall-clock hour, not midnight + `hour` x 3600 s: a day the clocks
    // change on has 23 or 25 hours, and the latter is an hour off after the change.
    const slotStart = new Date(g.day.getFullYear(), g.day.getMonth(), g.day.getDate(), hour);
    const slotEnd = new Date(g.day.getFullYear(), g.day.getMonth(), g.day.getDate(), hour + 1);
    let status: FbStatus = 'free';
    for (const b of blocks() ?? []) {
      if (b.status === 'unknown' || (b.principal ?? null) !== address) continue;
      const bs = instanceBound(b.start, true);
      const be = instanceBound(b.end, true);
      if (bs < slotEnd && be > slotStart) {
        if (b.status === 'busy') return 'busy';
        status = 'tentative';
      }
    }
    return status;
  }

  // ── resolutions ──
  async function run(fn: () => Promise<void>): Promise<void> {
    if (busy()) return;
    setBusy(true);
    setFailed(false);
    try {
      await fn();
    } catch {
      // The engine refused the update (`updateEvent` rejects on `notUpdated`).
      setFailed(true);
    } finally {
      setBusy(false);
    }
  }

  function reschedule(): void {
    const pr = pair();
    if (pr === null) return;
    void run(() => c.updateEvent(pr.later.event.id, { start: dateToLocal(pr.earlier.end) }));
  }

  function shorten(): void {
    const pr = pair();
    if (pr === null) return;
    const ms = pr.later.start.getTime() - pr.earlier.start.getTime();
    if (ms <= 0) return;
    void run(() => c.updateEvent(pr.earlier.event.id, { duration: formatDuration(ms) }));
  }

  function markTentative(): void {
    const pr = pair();
    if (pr === null) return;
    void run(() => c.updateEvent(pr.later.event.id, { status: 'tentative' }));
  }

  function doubleBook(): void {
    const pr = pair();
    if (pr === null) return;
    void run(() => c.updateEvent(pr.later.event.id, { freeBusyStatus: 'free' }));
  }

  function keepBoth(): void {
    // Accept the overlap unchanged. If it is the last pair, close the resolver.
    if (c.conflicts().length <= 1) props.onClose();
    else setSelected((i) => (i + 1) % c.conflicts().length);
  }

  // ── focus trap ──
  let dialogRef!: HTMLDivElement;
  let restoreEl: HTMLElement | null = null;
  onMount(() => {
    restoreEl = (document.activeElement as HTMLElement | null) ?? null;
    const first = dialogRef.querySelector<HTMLElement>(FOCUSABLE);
    (first ?? dialogRef).focus();
  });
  onCleanup(() => restoreEl?.focus?.());

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      e.preventDefault();
      props.onClose();
      return;
    }
    if (e.key !== 'Tab') return;
    const nodes = Array.from(dialogRef.querySelectorAll<HTMLElement>(FOCUSABLE));
    if (nodes.length === 0) return;
    const first = nodes[0]!;
    const last = nodes[nodes.length - 1]!;
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  function sidePanel(side: Side, which: 'earlier' | 'later'): JSX.Element {
    const cal = c.calendars().find((k) => k.id === side.event.calendarId);
    return (
      <div class={css.resolverSide} data-testid={`resolver-${which}`}>
        <h4 class={css.resolverSideTitle}><bdi>{side.event.title}</bdi></h4>
        <dl class={css.resolverMeta}>
          <dt>{t('calendar-field-start')}</dt>
          <dd>{formatFull(side.start)}</dd>
          <dt>{t('calendar-resolver-time')}</dt>
          <dd>{formatTime(side.start)} – {formatTime(side.end)}</dd>
          <dt>{t('calendar-field-calendar')}</dt>
          <dd><bdi>{cal?.name ?? ''}</bdi></dd>
          <Show when={side.event.locations[0] !== undefined}>
            <dt>{t('calendar-field-location')}</dt>
            <dd><bdi>{side.event.locations[0]!.name}</bdi></dd>
          </Show>
          <dt>{t('calendar-field-status')}</dt>
          <dd>{t(`calendar-status-${side.event.status}`)}</dd>
          <dt>{t('calendar-attendees')}</dt>
          <dd>{t('calendar-events-count-people', { count: Object.keys(side.event.participants).length })}</dd>
        </dl>
      </div>
    );
  }

  return (
    <div class={css.dialogBackdrop} onClick={(e) => e.target === e.currentTarget && props.onClose()}>
      <div
        ref={dialogRef}
        class={css.resolverDialog}
        role="dialog"
        tabindex={-1}
        aria-label={t('calendar-resolver-title')}
        aria-modal="true"
        onKeyDown={onKeyDown}
      >
        <h2 class={css.dialogTitle}>{t('calendar-resolver-title')}</h2>

        <Show
          when={pair()}
          fallback={<p class={css.dimText}>{t('calendar-resolver-none')}</p>}
        >
          {(pr) => (
            <>
              <Show when={c.conflicts().length > 1}>
                <div class={css.row}>
                  <label class={css.label} for="resolver-pair">{t('calendar-resolver-pick')}</label>
                  <select
                    id="resolver-pair"
                    class={css.input}
                    value={selected()}
                    onChange={(e) => setSelected(Number(e.currentTarget.value))}
                  >
                    <For each={c.conflicts()}>
                      {(cp, i) => (
                        <option value={i()}>
                          {t('calendar-resolver-pair-n', {
                            n: i() + 1,
                            total: c.conflicts().length,
                            a: isolate(c.masterById(cp.a)?.title ?? ''),
                            b: isolate(c.masterById(cp.b)?.title ?? ''),
                          })}
                        </option>
                      )}
                    </For>
                  </select>
                </div>
              </Show>

              <div class={css.resolverGridTwo}>
                {sidePanel(pr().earlier, 'earlier')}
                {sidePanel(pr().later, 'later')}
              </div>

              <p class={css.dimText}>
                {t('calendar-resolver-overlap', {
                  start: formatTime(localToDate(pr().pair.overlapStart)),
                  end: formatTime(localToDate(pr().pair.overlapEnd)),
                })}
              </p>

              {/* free/busy grid — the account's own busy time (queryFreeBusy) */}
              <Show when={loadingBusy()}>
                <p class={css.dimText} role="status">{t('calendar-fb-loading')}</p>
              </Show>
              <Show when={blocks() === null && !loadingBusy()}>
                <p class={css.dimText} role="note" data-testid="freebusy-unavailable">{t('calendar-fb-unavailable')}</p>
              </Show>
              <Show when={blocks() !== null && grid().hours.length > 0}>
                <div class={css.fbScroll}>
                  <table class={css.fbGrid} data-testid="freebusy-grid">
                    <caption class={css.srOnly}>{t('calendar-fb-caption-people')}</caption>
                    <thead>
                      <tr>
                        <th scope="col" class={css.fbCorner}>{t('calendar-fb-whose')}</th>
                        <For each={grid().hours}>
                          {(h) => <th scope="col" class={css.fbHead}>{String(h).padStart(2, '0')}</th>}
                        </For>
                      </tr>
                    </thead>
                    <tbody>
                      <tr>
                        <th scope="row" class={css.fbRowHead}>{t('calendar-fb-own')}</th>
                        <FbCells hours={grid().hours} label={t('calendar-fb-own')} statusAt={(h) => statusAt(h)} />
                      </tr>
                      <For each={people()}>
                        {(person) => {
                          // A name is the event's text: isolate it, and keep the
                          // address (what the server answered for) in the title.
                          const label = (): string => isolate(person.name !== '' ? person.name : person.address);
                          return (
                            <tr data-testid="freebusy-person" data-principal={person.address}>
                              <th scope="row" class={css.fbRowHead} title={person.address}>{label()}</th>
                              <Show
                                when={!unknown(person.address)}
                                fallback={
                                  <td class={css.fbUnknown} colSpan={grid().hours.length} data-testid="freebusy-unknown">
                                    {t('calendar-fb-unknown', { principal: label() })}
                                  </td>
                                }
                              >
                                <FbCells hours={grid().hours} label={label()} statusAt={(h) => statusAt(h, person.address)} />
                              </Show>
                            </tr>
                          );
                        }}
                      </For>
                    </tbody>
                  </table>
                </div>
              </Show>

              <Show when={failed()}>
                <p class={css.dangerText} role="alert">{t('calendar-save-failed')}</p>
              </Show>

              <div class={css.resolverActions}>
                <button type="button" class={css.primaryButton} disabled={busy()} onClick={reschedule}>
                  {t('calendar-resolver-reschedule')}
                </button>
                <button type="button" class={css.button} disabled={busy()} onClick={shorten}>
                  {t('calendar-resolver-shorten')}
                </button>
                <button type="button" class={css.button} disabled={busy()} onClick={markTentative}>
                  {t('calendar-resolver-tentative')}
                </button>
                <button type="button" class={css.button} disabled={busy()} onClick={doubleBook}>
                  {t('calendar-resolver-double-book')}
                </button>
                <button type="button" class={css.button} disabled={busy()} onClick={keepBoth}>
                  {t('calendar-resolver-keep')}
                </button>
              </div>
            </>
          )}
        </Show>

        <div class={css.dialogActions}>
          <span class={css.spacer} />
          <button type="button" class={css.button} onClick={props.onClose}>{t('common-close')}</button>
        </div>
      </div>
    </div>
  );
}

/** One grid row's hour cells. Each cell names whose, which hour and what status
 *  in its accessible name; the symbol is decoration, never the only signal. */
function FbCells(props: { hours: number[]; label: string; statusAt: (hour: number) => FbStatus }): JSX.Element {
  return (
    <For each={props.hours}>
      {(h) => {
        const s = (): FbStatus => props.statusAt(h);
        return (
          <td
            class={css.fbCell[s()]}
            aria-label={t('calendar-fb-cell', {
              principal: props.label,
              hour: `${String(h).padStart(2, '0')}:00`,
              status: t(`calendar-fb-${s()}`),
            })}
          >
            <span aria-hidden="true">{s() === 'free' ? '' : s() === 'busy' ? '●' : '◐'}</span>
          </td>
        );
      }}
    </For>
  );
}
