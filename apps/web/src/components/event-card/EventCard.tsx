// The Outlook-style event card (t53), shared by the reader (a message with a
// calendar part) and the attachment preview (one `.ics` part). It shows what
// the server read from the message — title, time in the viewer's zone (and the
// organizer's when it differs), recurrence in words, location and join link,
// organizer and attendees with their answers, the description as text — beside
// a mini agenda of the viewer's own day, and offers the actions the message's
// METHOD allows. Nothing is applied or sent without a click; the server checks
// the sender, the organizer/attendee rules and SEQUENCE/DTSTAMP again on every
// action.

import { createMemo, createResource, createSignal, For, Match, onMount, Show, Switch, type JSX } from 'solid-js';
import type { Client } from '../../api/client.ts';
import { createConfiguredClient } from '../../api/transport.ts';
import { PimMethodError } from '../../modules/calendar/api.ts';
import { requestCalendarFocus } from '../../modules/calendar/focusHandoff.ts';
import { formatDateWith, isolate, loadCatalog, t } from '../../i18n';
import { wallTimeCandidates } from '../../i18n/timezones.ts';
import { Icon } from '../Icon.tsx';
import { messageTimeZone } from '../TimeHelpers.tsx';
import { toast } from '../toast/index.ts';
import {
  destroyEvent, ownCalendars, readItip, respondToMessage,
  type ItipItem, type ItipView, type Outcome, type PickableCalendar, type ResponseAction,
} from './api.ts';
import { DayAgenda } from './DayAgenda.tsx';
import {
  describeRecurrence, displayDescription, eventRange, formatRange, joinLink, linkSegments, organizerTime,
  participantAddress, people, safeUrl, type CardEvent, type CardParticipant, type EventRange,
} from './eventInfo.ts';
import * as css from './eventCard.css.ts';

/** Attendee lists longer than this start collapsed. */
const PEOPLE_SHOWN = 4;
const PROVIDER_NAMES = { teams: 'Microsoft Teams', zoom: 'Zoom', meet: 'Google Meet', jitsi: 'Jitsi Meet', webex: 'Webex' } as const;

export interface EventCardProps {
  accountId: string;
  emailId: string;
  /** One part of the message (`<emailId>.<n>`), for an attachment preview. */
  blobId?: string;
  client?: Pick<Client, 'jmap'>;
  /** Rendered when the part is not a calendar the server can read. */
  fallback?: JSX.Element;
  /** The viewer's time zone (tests); the display-zone setting otherwise. */
  zone?: string;
}

type Status = 'accepted' | 'tentative' | 'declined';
const STATUS_OF: Record<Exclude<ResponseAction, 'counter'>, Status> = { accept: 'accepted', tentative: 'tentative', decline: 'declined' };

/** "Open in calendar": the calendar module, on the event's day. */
function OpenInCalendar(props: { day: Date | null }): JSX.Element {
  return (
    <a class="btn btn--ghost" href="#/calendar/day" onClick={() => { if (props.day) requestCalendarFocus(props.day); }}>
      <Icon name="calendar" />{t('calendar-card-open')}
    </a>
  );
}

function Rsvp(props: { status: string | undefined }): JSX.Element {
  const tone = (): keyof typeof css.rsvpTone => {
    const s = props.status ?? 'needs-action';
    return s === 'accepted' || s === 'tentative' || s === 'declined' ? s : 'needs-action';
  };
  return <span class={`${css.rsvp} ${css.rsvpTone[tone()]}`} aria-hidden="true" />;
}

function personLabel(key: string, p: CardParticipant): string {
  const address = participantAddress(key, p);
  return p.name && p.name !== address ? `${p.name} <${address}>` : address;
}

function statusWord(status: string | undefined): string {
  switch (status) {
    case 'accepted': return t('calendar-card-rsvp-accepted');
    case 'tentative': return t('calendar-card-rsvp-tentative');
    case 'declined': return t('calendar-card-rsvp-declined');
    default: return t('calendar-card-rsvp-none');
  }
}

/** Plain text with its `http(s)` links as links (`rel=noopener noreferrer`). */
function LinkedText(props: { text: string }): JSX.Element {
  return (
    <For each={linkSegments(props.text)}>{(seg) => seg.url
      ? <a href={seg.url} target="_blank" rel="noopener noreferrer nofollow">{seg.text}</a>
      : <>{seg.text}</>}
    </For>
  );
}

function Details(props: { ev: CardEvent; zone: string; range: EventRange | null; participants?: Record<string, CardParticipant> | null | undefined }): JSX.Element {
  const [allPeople, setAllPeople] = createSignal(false);
  const ev = (): CardEvent => props.participants ? { ...props.ev, participants: props.participants } : props.ev;
  const who = createMemo(() => people(ev()));
  const location = (): string => (props.ev.locations ?? []).map((l) => l.name ?? '').filter(Boolean).join('; ');
  const join = createMemo(() => joinLink(props.ev));
  const recurrence = createMemo(() => describeRecurrence(props.ev));
  const description = createMemo(() => displayDescription(props.ev.description));
  const counts = createMemo(() => {
    const c = { accepted: 0, tentative: 0, declined: 0 };
    for (const [, p] of who().attendees) if (p.participationStatus === 'accepted' || p.participationStatus === 'tentative' || p.participationStatus === 'declined') c[p.participationStatus] += 1;
    return c;
  });
  const shown = (): Array<[string, CardParticipant]> => allPeople() ? who().attendees : who().attendees.slice(0, PEOPLE_SHOWN);
  return (
    <div>
      <dl class={css.rows}>
        <Show when={props.range}>{(range) => (
          <div class={css.row}>
            <span class={css.rowIcon}><Icon name="clock" /></span>
            <div class={css.rowMain}>
              <dt>{t('calendar-card-when')}</dt>
              <dd data-card="when">{formatRange(range(), props.zone)}</dd>
              <Show when={organizerTime(props.ev, range(), props.zone)}>{(other) => <dd class={css.dim} data-card="organizer-time">{other()}</dd>}</Show>
              <Show when={recurrence()}>{(r) => <dd class={css.dim} data-card="recurrence"><Icon name="sync" /> {r()}</dd>}</Show>
            </div>
          </div>
        )}</Show>
        <Show when={location() || join()}>
          <div class={css.row}>
            <span class={css.rowIcon}><Icon name="pin" /></span>
            <div class={css.rowMain}>
              <dt>{t('calendar-card-where')}</dt>
              <Show when={location()}>
                <dd dir="auto" data-card="where">
                  <Show when={safeUrl(location())} fallback={<LinkedText text={location()} />}>
                    {(url) => <a href={url()} target="_blank" rel="noopener noreferrer nofollow">{location()}</a>}
                  </Show>
                </dd>
              </Show>
              <Show when={join()}>{(j) => (
                <dd>
                  <a class={`btn btn--primary ${css.join}`} href={j().url} target="_blank" rel="noopener noreferrer nofollow" data-card="join">
                    <Icon name="link" />{t('calendar-card-join', { provider: PROVIDER_NAMES[j().provider] })}
                  </a>
                </dd>
              )}</Show>
            </div>
          </div>
        </Show>
        <Show when={who().organizer}>{(org) => (
          <div class={css.row}>
            <span class={css.rowIcon}><Icon name="mail" /></span>
            <div class={css.rowMain}>
              <dt>{t('calendar-card-organizer')}</dt>
              <dd dir="auto" data-card="organizer">{t('calendar-card-organizer-is', { who: isolate(personLabel(org()[0], org()[1])) })}</dd>
            </div>
          </div>
        )}</Show>
        <Show when={who().attendees.length > 0}>
          <div class={css.row}>
            <span class={css.rowIcon}><Icon name="users" /></span>
            <div class={css.rowMain}>
              <dt>{t('calendar-card-attendees')}</dt>
              <dd data-card="attendee-summary">{t('calendar-card-attendee-summary', { count: who().attendees.length, accepted: counts().accepted, tentative: counts().tentative, declined: counts().declined })}</dd>
              <dd>
                <ul class={css.people} aria-label={t('calendar-card-attendees')}>
                  <For each={shown()}>{([key, p]) => (
                    <li class={css.person}>
                      <Rsvp status={p.participationStatus} />
                      <span class={css.personName} dir="auto" title={personLabel(key, p)}>{p.name || participantAddress(key, p)}</span>
                      <span class={css.dim}>{statusWord(p.participationStatus)}</span>
                    </li>
                  )}</For>
                </ul>
                <Show when={who().attendees.length > PEOPLE_SHOWN}>
                  <button type="button" class={css.linkButton} aria-expanded={allPeople()} onClick={() => setAllPeople(!allPeople())}>
                    {allPeople() ? t('calendar-card-people-fewer') : t('calendar-card-people-all', { count: who().attendees.length })}
                  </button>
                </Show>
              </dd>
            </div>
          </div>
        </Show>
      </dl>
      <Show when={description().text}>
        <p class={css.description} dir="auto" data-card="description"><LinkedText text={description().text} />{description().clipped ? '…' : ''}</p>
      </Show>
    </div>
  );
}

/** The day a calendar should open on for `range`. */
function dayOf(range: EventRange | null): Date | null {
  if (!range) return null;
  if (!range.allDay) return range.start;
  const [y, m, d] = range.start.toISOString().slice(0, 10).split('-').map(Number) as [number, number, number];
  return new Date(y, m - 1, d);
}

/** A `datetime-local` value in `viewerZone` → the event's own wall time. */
function proposalStart(value: string, viewerZone: string, eventZone: string | null | undefined): string | null {
  const instant = wallTimeCandidates(value.length === 16 ? `${value}:00` : value, viewerZone)[0];
  if (!instant) return null;
  if (!eventZone) return value.length === 16 ? `${value}:00` : value;
  const zone = eventZone === 'Etc/UTC' ? 'UTC' : eventZone;
  const p = Object.fromEntries(new Intl.DateTimeFormat('en-CA', { timeZone: zone, hourCycle: 'h23', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit' })
    .formatToParts(instant).map((x) => [x.type, x.value]));
  return `${p['year']}-${p['month']}-${p['day']}T${p['hour']}:${p['minute']}:${p['second']}`;
}

function toLocalInput(d: Date, zone: string): string {
  const p = Object.fromEntries(new Intl.DateTimeFormat('en-CA', { timeZone: zone, hourCycle: 'h23', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
    .formatToParts(d).map((x) => [x.type, x.value]));
  return `${p['year']}-${p['month']}-${p['day']}T${p['hour']}:${p['minute']}`;
}

export function EventCard(props: EventCardProps): JSX.Element {
  onMount(() => void loadCatalog('calendar'));
  const client = props.client ?? createConfiguredClient();
  const zone = (): string => props.zone ?? messageTimeZone();
  const [busy, setBusy] = createSignal(false);
  const [result, setResult] = createSignal<string | null>(null);
  const [sendResponse, setSendResponse] = createSignal(true);
  const [comment, setComment] = createSignal('');
  const [proposing, setProposing] = createSignal(false);
  const [proposal, setProposal] = createSignal('');
  const [calendarId, setCalendarId] = createSignal<string>('');

  async function read(apply = false, extra: { calendarId?: string; uids?: string[] } = {}): Promise<ItipView | null> {
    try {
      return await readItip(client, {
        accountId: props.accountId, emailId: props.emailId, apply,
        ...(props.blobId ? { blobId: props.blobId } : {}), ...extra,
      });
    } catch (error) {
      // Not a calendar the server reads, or a server without the method.
      if (!apply && error instanceof PimMethodError && ['invalidArguments', 'unknownMethod', 'notFound'].includes(error.type)) return null;
      throw error;
    }
  }
  const [view, { mutate, refetch }] = createResource(
    () => [props.accountId, props.emailId, props.blobId ?? ''] as const,
    () => read(false),
  );
  const [calendars] = createResource(
    () => (view() ? props.accountId : undefined),
    async (accountId) => { try { return await ownCalendars(client, accountId); } catch { return [] as PickableCalendar[]; } },
  );
  const ready = (): ItipView | null => (view.loading || view.error ? null : view() ?? null);
  const pickedCalendar = (): string | undefined => calendarId() || calendars()?.find((c) => c.role === 'default')?.id || calendars()?.[0]?.id;

  const range = createMemo(() => { const v = ready(); return v ? eventRange(v.event, zone()) : null; });
  const myStatus = (): string | null => ready()?.stored?.participationStatus ?? null;
  const kind = (): 'invite' | 'cancel' | 'reply' | 'counter' | 'publish' => {
    switch (ready()?.method) {
      case 'REQUEST': return 'invite';
      case 'CANCEL': return 'cancel';
      case 'REPLY': return 'reply';
      case 'COUNTER': return 'counter';
      default: return 'publish';
    }
  };
  const answerable = (): boolean => kind() === 'invite' && !!ready()?.me && (['new', 'update', 'current'] as Outcome[]).includes(ready()!.outcome);
  const cancelled = (): boolean => kind() === 'cancel' || ready()?.event.status === 'cancelled';

  async function run(work: () => Promise<string | null>): Promise<void> {
    if (busy()) return;
    const key = `${props.accountId}\u0000${props.emailId}\u0000${props.blobId ?? ''}`;
    setBusy(true);
    try {
      const message = await work();
      if (key !== `${props.accountId}\u0000${props.emailId}\u0000${props.blobId ?? ''}`) return;
      setResult(message);
      if (message) toast.success(message);
      const next = await read(false);
      mutate(next);
    } catch (error) {
      const code = error instanceof PimMethodError ? error.code : null;
      const message = code === 'stale' ? t('calendar-card-out-of-date')
        : code === 'notInvited' ? t('calendar-card-not-invited-short')
          : code === 'notOrganizer' ? t('calendar-invite-not-organizer')
            : t('calendar-invite-apply-failed');
      setResult(message);
      toast.error(message);
    } finally {
      setBusy(false);
    }
  }

  function respond(action: ResponseAction): void {
    void run(async () => {
      const v = ready()!;
      const counter = action === 'counter' ? proposalStart(proposal(), zone(), v.event.timeZone) : null;
      if (action === 'counter' && !counter) return t('calendar-card-propose-invalid');
      const cal = v.outcome === 'new' ? pickedCalendar() : undefined;
      const res = await respondToMessage(client, {
        accountId: props.accountId, emailId: props.emailId, action, sendResponse: sendResponse(), comment: comment(),
        ...(props.blobId ? { blobId: props.blobId } : {}), ...(cal ? { calendarId: cal } : {}),
        ...(counter ? { counter: { start: counter, duration: v.event.duration || 'PT1H' } } : {}),
      });
      setProposing(false);
      if (action === 'counter') return res.sent ? t('calendar-card-proposal-sent', { organizer: isolate(res.to) }) : t('calendar-card-not-sent');
      const what = statusWord(STATUS_OF[action]);
      if (!res.sendResponse) return t('calendar-card-answered-silently', { answer: what });
      return res.sent ? t('calendar-card-answered-sent', { answer: what, organizer: isolate(res.to) }) : t('calendar-card-answered-not-sent', { answer: what });
    });
  }

  function apply(extra: { uids?: string[] } = {}): void {
    void run(async () => {
      const v = ready()!;
      const cal = v.outcome === 'new' || v.outcome === 'publish' ? pickedCalendar() : undefined;
      const res = await read(true, { ...extra, ...(cal ? { calendarId: cal } : {}) });
      if (!res?.applied) return t('calendar-invite-unchanged');
      if (res.outcome === 'publish' || res.imported) return t('calendar-card-added', { count: res.imported?.length ?? 1 });
      return t('calendar-invite-applied');
    });
  }

  function remove(): void {
    void run(async () => {
      let v = ready()!;
      if (v.outcome === 'cancel') {
        const applied = await read(true);
        if (applied) v = applied;
      }
      if (!v.eventId) return t('calendar-invite-unchanged');
      return (await destroyEvent(client, props.accountId, v.eventId)) ? t('calendar-card-removed') : t('calendar-invite-unchanged');
    });
  }

  const kicker = (): string => {
    const v = ready()!;
    switch (kind()) {
      case 'invite': return v.outcome === 'update' ? t('calendar-card-kind-update') : t('calendar-card-kind-request');
      case 'cancel': return t('calendar-card-kind-cancel');
      case 'reply': return t('calendar-card-kind-reply');
      case 'counter': return t('calendar-card-kind-counter');
      default: return t('calendar-card-kind-publish', { count: v.items.length });
    }
  };
  const badge = (): { text: string; tone: keyof typeof css.badgeTone } | null => {
    const v = ready()!;
    const s = myStatus();
    if (kind() === 'invite' && (s === 'accepted' || s === 'tentative' || s === 'declined')) return { text: statusWord(s), tone: s };
    if (kind() === 'cancel' && v.stored?.status === 'cancelled') return { text: t('calendar-card-cancelled'), tone: 'declined' };
    if (v.eventId && kind() !== 'reply' && kind() !== 'counter') return { text: t('calendar-card-in-calendar'), tone: 'info' };
    return null;
  };
  /** The status line under the details. */
  const notice = (): { text: string; tone: keyof typeof css.noticeTone } | null => {
    const v = ready()!;
    const replier = (): string => {
      const entry = Object.entries(v.event.participants ?? {}).find(([k, p]) => p.role !== 'organizer' && participantAddress(k, p).toLowerCase() === v.sender);
      return isolate(entry?.[1].name || v.sender);
    };
    switch (v.outcome) {
      case 'stale': return { text: t('calendar-card-out-of-date'), tone: 'warn' };
      case 'notOrganizer': return { text: t('calendar-invite-not-organizer'), tone: 'danger' };
      case 'notAttendee': return { text: t('calendar-invite-not-attendee'), tone: 'danger' };
      case 'unknownEvent': return { text: t('calendar-invite-unknown'), tone: 'info' };
      case 'update': return { text: t('calendar-invite-newer'), tone: 'info' };
      case 'cancel': return { text: t('calendar-invite-cancelled'), tone: 'danger' };
      case 'reply': return { text: t('calendar-card-replied', { who: replier(), answer: v.answer ?? 'needs-action' }), tone: v.answer === 'declined' ? 'danger' : 'ok' };
      case 'unsupported': return { text: t('calendar-card-counter', { who: isolate(v.sender) }), tone: 'info' };
      case 'new':
        if (kind() === 'invite' && !v.me) return { text: t('calendar-card-not-invited'), tone: 'warn' };
        return v.me ? { text: t('calendar-card-invited-as', { address: isolate(v.me) }), tone: 'info' } : null;
      case 'current':
        if (kind() === 'reply') return { text: t('calendar-card-replied', { who: replier(), answer: v.answer ?? 'needs-action' }), tone: 'ok' };
        if (kind() === 'cancel') return { text: t('calendar-invite-cancelled'), tone: 'danger' };
        return null;
      default:
        return v.todos > 0 ? { text: t('calendar-card-todos-skipped', { count: v.todos }), tone: 'info' } : null;
    }
  };
  const tile = (): { month: string; day: string } | null => {
    const r = range();
    if (!r) return null;
    const tz = r.allDay ? 'UTC' : zone();
    return { month: formatDateWith(r.start, { month: 'short', timeZone: tz }), day: formatDateWith(r.start, { day: 'numeric', timeZone: tz }) };
  };
  const calendarPicker = (): JSX.Element => (
    <Show when={(calendars() ?? []).length > 1}>
      <label class={css.optionRow}>
        <span>{t('calendar-card-calendar')}</span>
        <select value={pickedCalendar() ?? ''} onChange={(e) => setCalendarId(e.currentTarget.value)} disabled={busy()}>
          <For each={calendars() ?? []}>{(c) => <option value={c.id}>{c.name}</option>}</For>
        </select>
      </label>
    </Show>
  );

  return (
    <Show when={view.error || ready()} fallback={<Show when={!view.loading && !view.error && view() === null}>{props.fallback}</Show>}>
      <section class={css.card} aria-label={ready() && kind() === 'publish' ? t('calendar-card-region-event') : t('calendar-invite-title')} data-event-card={ready()?.outcome ?? 'error'}>
        <Show when={view.error}>
          <div class={css.options}>
            <p role="alert">{t('calendar-invite-load-failed')}</p>
            <div><button type="button" class="btn btn--ghost" onClick={() => void refetch()}>{t('calendar-invite-retry')}</button></div>
          </div>
        </Show>
        <Show when={ready()}>{(v) => (
          <>
            <header class={`${css.head} ${cancelled() ? css.headCancelled : ''}`}>
              <Show when={tile()}>{(d) => <div class={css.dateTile} aria-hidden="true"><span class={css.dateTileMonth}>{d().month}</span><span class={css.dateTileDay}>{d().day}</span></div>}</Show>
              <div class={css.headText}>
                <p class={css.kicker}>{kicker()}</p>
                <h3 class={`${css.title} ${cancelled() ? css.titleCancelled : ''}`} dir="auto">{v().event.title || t('calendar-untitled')}</h3>
                <p class={css.dim}>{t('calendar-invite-sender', { sender: isolate(v().sender) })}</p>
              </div>
              <Show when={badge()}>{(b) => <span class={`${css.badge} ${css.badgeTone[b().tone]}`} data-card="badge">{b().text}</span>}</Show>
            </header>

            <Show when={kind() !== 'publish' || v().items.length === 1} fallback={
              <div class={css.body}>
                <ul class={css.itemList} aria-label={t('calendar-card-events')}>
                  <For each={v().items}>{(item: ItipItem) => {
                    const r = (): EventRange | null => eventRange(item.event, zone());
                    return (
                      <li class={css.item}>
                        <div>
                          <strong dir="auto">{item.event.title || t('calendar-untitled')}</strong>
                          <Show when={r()}>{(rr) => <p class={css.dim}>{formatRange(rr(), zone())}</p>}</Show>
                          <Show when={describeRecurrence(item.event)}>{(rec) => <p class={css.dim}>{rec()}</p>}</Show>
                        </div>
                        <Show when={item.eventId} fallback={
                          <button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => apply({ uids: [item.event.uid ?? ''] })}>{t('calendar-card-add-one')}</button>
                        }>
                          <span class={`${css.badge} ${css.badgeTone.info}`}>{t('calendar-card-in-calendar')}</span>
                        </Show>
                      </li>
                    );
                  }}</For>
                </ul>
              </div>
            }>
              <div class={css.body}>
                <Details ev={v().event} zone={zone()} range={range()} participants={kind() === 'reply' && v().stored?.participants ? v().stored!.participants : undefined} />
                <Show when={range() && kind() !== 'reply'}>
                  <DayAgenda accountId={props.accountId} client={client} range={range()!} uid={v().event.uid} zone={zone()} calendars={calendars() ?? []} />
                </Show>
              </div>
            </Show>

            <Show when={notice()}>{(n) => <p class={`${css.notice} ${css.noticeTone[n().tone]}`} role="status" data-card="notice">{n().text}</p>}</Show>
            <Show when={result()}>{(r) => <p class={`${css.notice} ${css.noticeTone.ok}`} role="status" data-card="result" style={{ 'margin-block-start': '0.5rem' }}>{r()}</p>}</Show>

            <Switch>
              <Match when={answerable()}>
                <div class={css.actions}>
                  <div class={css.actionsGroup} role="group" aria-label={t('calendar-card-respond')}>
                    <button type="button" class="btn btn--primary" disabled={busy()} aria-pressed={myStatus() === 'accepted'} onClick={() => respond('accept')}><Icon name="check" />{t('calendar-card-accept')}</button>
                    <button type="button" class="btn" disabled={busy()} aria-pressed={myStatus() === 'tentative'} onClick={() => respond('tentative')}><Icon name="tentative" />{t('calendar-card-tentative')}</button>
                    <button type="button" class="btn" disabled={busy()} aria-pressed={myStatus() === 'declined'} onClick={() => respond('decline')}><Icon name="close" />{t('calendar-card-decline')}</button>
                    <button type="button" class="btn btn--ghost" disabled={busy()} aria-expanded={proposing()} onClick={() => {
                      if (!proposal() && range()) setProposal(toLocalInput(range()!.start, zone()));
                      setProposing(!proposing());
                    }}><Icon name="clock" />{t('calendar-card-propose')}</button>
                  </div>
                  <Show when={v().eventId}><OpenInCalendar day={dayOf(range())} /></Show>
                </div>
                <div class={css.options}>
                  <label class={css.optionRow}>
                    <input type="checkbox" checked={sendResponse()} disabled={busy()} onChange={(e) => setSendResponse(e.currentTarget.checked)} />
                    <span>{t('calendar-card-send-response', { organizer: isolate(v().organizer ?? v().sender) })}</span>
                  </label>
                  <Show when={sendResponse()}>
                    <label>
                      <span class={css.dim}>{t('calendar-card-comment')}</span>
                      <textarea class={css.comment} maxLength={2000} value={comment()} disabled={busy()} onInput={(e) => setComment(e.currentTarget.value)} />
                    </label>
                  </Show>
                  <Show when={v().outcome === 'new'}>{calendarPicker()}</Show>
                  <Show when={proposing()}>
                    <div class={css.optionRow}>
                      <label class={css.optionRow}>
                        <span>{t('calendar-card-propose-start')}</span>
                        <input type="datetime-local" value={proposal()} disabled={busy()} onInput={(e) => setProposal(e.currentTarget.value)} />
                      </label>
                      <button type="button" class="btn" disabled={busy() || !proposal()} onClick={() => respond('counter')}>{t('calendar-card-propose-send')}</button>
                    </div>
                  </Show>
                </div>
              </Match>
              <Match when={kind() === 'invite' && v().outcome === 'new'}>
                <div class={css.actions}>
                  <button type="button" class="btn btn--primary" disabled={busy()} onClick={() => apply()}>{t('calendar-invite-add')}</button>
                  {calendarPicker()}
                  <span class={css.dim}>{t('calendar-invite-no-reply')}</span>
                </div>
              </Match>
              <Match when={kind() === 'cancel' && (v().outcome === 'cancel' || (v().outcome === 'current' && v().eventId))}>
                <div class={css.actions}>
                  <button type="button" class="btn btn--primary" disabled={busy()} onClick={remove}><Icon name="delete" />{t('calendar-card-remove')}</button>
                </div>
              </Match>
              <Match when={kind() === 'reply' && v().outcome === 'reply'}>
                <div class={css.actions}>
                  <button type="button" class="btn btn--primary" disabled={busy()} onClick={() => apply()}>{t('calendar-card-record-reply')}</button>
                  <Show when={v().eventId}><OpenInCalendar day={dayOf(range())} /></Show>
                </div>
              </Match>
              <Match when={kind() === 'publish' && v().outcome === 'publish'}>
                <div class={css.actions}>
                  <button type="button" class="btn btn--primary" disabled={busy()} onClick={() => apply()}>
                    <Icon name="plus" />{v().items.length > 1 ? t('calendar-card-add-all', { count: v().items.filter((i) => !i.eventId).length }) : t('calendar-invite-add')}
                  </button>
                  {calendarPicker()}
                </div>
              </Match>
              <Match when={v().eventId}>
                <div class={css.actions}><OpenInCalendar day={dayOf(range())} /></div>
              </Match>
            </Switch>
          </>
        )}</Show>
      </section>
    </Show>
  );
}
