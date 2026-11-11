import { InlineLoading } from '../../components/InlineLoading.tsx';
import {createEntityContextMenu} from '../../components/entityContextMenu.ts';
import {handleModuleActions} from '../../state/moduleActions.ts';
import { Icon } from '../../components/Icon.tsx';
import { ColorField } from '../../components/color/ColorPicker.tsx';
import {MiniCalendar} from './MiniCalendar.tsx';
// Calendar module (plan §2.5, §3 e4): the mounted shell that composes the view
// switcher, the multi-calendar overlay sidebar, all nine views, the event
// create/edit dialog (recurrence + reminders + attendees + free/busy + invite
// controls), conflict badges, and ICS / `.hol` import + export.
//
// `CalendarApp` takes an explicit controller so views + tests drive it without
// the app store. The app shell (`shell/mounts.tsx`) renders it over the
// engine-backed controller from `state/slices/calendar.ts`; `CalendarModule`
// below is the mock-backed variant.

import { For, Index, Show, createSignal, createUniqueId, onMount, type JSX } from 'solid-js';
import { rawT, t, loadCatalog } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import type { Calendar, CalendarEvent } from '../../api/pim-types.ts';
import { createCalendarController, type CalendarBackend, type CalendarController } from './controller.ts';
import { MOCK_IDENTITY, createMockFeeds, createMockStore, mockSession, createMockJmap, type MockStore } from './mock.ts';
import { CALENDAR_VIEWS, type CalendarView } from './types.ts';
import { ActiveView } from './views.tsx';
import { EventEditor } from './EventEditor.tsx';
import { ShareDialog } from './ShareDialog.tsx';
import { ConflictResolver } from './ConflictResolver.tsx';
import { HolLocationPicker } from './HolLocationPicker.tsx';
import { formatFull, formatMonth, formatMonthYear } from './datetime.ts';
import { readCalendarFile } from './decode.ts';
import { PimMethodError } from './api.ts';
import { FeedError } from './feeds.ts';
import { holLocations, holPack, type HolLocation } from './hol-locations.ts';
import { HOLIDAY_PACKS } from './holidays.ts';
import * as css from './calendar.css.ts';

/** Build a mock-backed controller (no server: the in-memory store in `mock.ts`). */
export function makeMockController(store: MockStore = createMockStore()): CalendarController {
  const jmap = createMockJmap(store);
  const backend: CalendarBackend = {
    jmap,
    feeds: createMockFeeds(store),
    resolveAccount: () =>
      Promise.resolve(mockSession().primaryAccounts['urn:mailwoman:calendars'] ?? null),
    resolveIdentity: () => Promise.resolve(MOCK_IDENTITY),
  };
  return createCalendarController(backend);
}

/** A short header label describing the focused window for the active view. */
function headerLabel(controller: CalendarController): string {
  const v = controller.view();
  const f = controller.focusDate();
  if (v === 'day') return formatFull(f);
  if (v === 'year') return String(f.getFullYear());
  if (v === 'month' || v === 'tri-month') return formatMonthYear(f);
  return `${formatMonth(f)} ${f.getFullYear()}`;
}

function triggerDownload(filename: string, text: string): void {
  if (typeof document === 'undefined' || typeof URL.createObjectURL !== 'function') return;
  const blob = new Blob([text], { type: 'text/calendar' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export interface CalendarAppProps {
  controller: CalendarController;
}

export function CalendarApp(props: CalendarAppProps): JSX.Element {
  const c = props.controller;
  const [mobileCalendars,setMobileCalendars]=createSignal(false);
  const calendarSidebarId=createUniqueId();
  const [editorOpen, setEditorOpen] = createSignal(false);
  const [editing, setEditing] = createSignal<CalendarEvent | null>(null);
  const [defaultStart, setDefaultStart] = createSignal<Date | undefined>(undefined);
  const [defaultCalendar,setDefaultCalendar]=createSignal<string>();
  const [resolverOpen, setResolverOpen] = createSignal(false);
  // P1 share dialog target, P3 quick-add line, P4 category filter, P6 webcal URL.
  const [sharing, setSharing] = createSignal<Calendar | null>(null);
  const [quickText, setQuickText] = createSignal('');
  const [catFilter, setCatFilter] = createSignal('');
  const [webcalUrl, setWebcalUrl] = createSignal('');
  // Action outcomes use the shell's shared, dismissible toast region.
  function setFeedback(feedback: { kind: 'error' | 'info'; text: string } | null): void {
    const id = 'calendar-feedback';
    if (feedback === null) {
      toast.dismiss(id);
    } else {
      const show = feedback.kind === 'error' ? toast.error : toast.success;
      show(feedback.text, { id, testId: id });
    }
  }
  // The locations of a picked `.hol` file that has more than one, while the user
  // chooses among them. Such a file is never imported whole.
  const [locationChoice, setLocationChoice] = createSignal<HolLocation[] | null>(null);

  /** Run a user action; on failure show `failKey`'s message instead of throwing. */
  async function attempt(failKey: string, fn: () => Promise<void>): Promise<void> {
    setFeedback(null);
    try {
      await fn();
    } catch {
      setFeedback({ kind: 'error', text: t(failKey) });
    }
  }

  async function importInto(text: string): Promise<void> {
    const target = c.visibleCalendars()[0]?.id ?? c.calendars()[0]?.id;
    if (target === undefined) return;
    setFeedback(null);
    try {
      const { count, skipped } = await c.importIcs(target, text);
      const done = t('calendar-import-done', { count });
      setFeedback({ kind: 'info', text: skipped > 0 ? `${done} ${t('calendar-import-skipped', { count: skipped })}` : done });
    } catch (e) {
      // A cancellation or reply is applied from its message, not imported.
      const itip = e instanceof PimMethodError && e.code === 'itipNotImportable';
      setFeedback({ kind: 'error', text: t(itip ? 'calendar-import-itip-message' : 'calendar-import-failed') });
    }
  }

  onMount(() => {
    void loadCatalog('calendar');
    void loadCatalog('appearance');
    void c.load();
  });

  function openNew(calendarId?:string): void {
    setDefaultCalendar(calendarId);
    setEditing(null);
    setDefaultStart(undefined);
    setEditorOpen(true);
  }
  function openNewAt(day: Date): void {
    setDefaultCalendar(undefined);
    setEditing(null);
    setDefaultStart(day);
    setEditorOpen(true);
  }
  function openEvent(masterId: string): void {
    setDefaultCalendar(undefined);
    setEditing(c.masterById(masterId) ?? null);
    setDefaultStart(undefined);
    setEditorOpen(true);
  }

  async function onImportFile(e: Event): Promise<void> {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file === undefined) return;
    setFeedback(null);
    let text: string;
    try {
      // Outlook's `.hol` is UTF-16, which `file.text()` reads in some browsers
      // and not in others; `readCalendarFile` goes by the byte order mark.
      text = await readCalendarFile(file);
    } catch {
      setFeedback({ kind: 'error', text: t('calendar-import-failed') });
      return;
    } finally {
      input.value = '';
    }
    const locations = holLocations(text);
    if (locations.length > 1) setLocationChoice(locations);
    else await importInto(text);
  }

  async function onLocationsChosen(chosen: HolLocation[]): Promise<void> {
    setLocationChoice(null);
    await importInto(holPack(chosen));
  }

  async function onExport(): Promise<void> {
    await attempt('calendar-export-failed', async () => {
      triggerDownload('mailwoman-calendar.ics', await c.exportIcs());
    });
  }

  async function subscribeHoliday(packId: string): Promise<void> {
    const pack = HOLIDAY_PACKS.find((p) => p.id === packId);
    if (pack === undefined) return;
    await importInto(pack.ics);
  }

  async function onQuickAdd(): Promise<void> {
    const text = quickText().trim();
    if (text === '') return;
    await attempt('calendar-quick-add-failed', async () => {
      await c.quickAdd(text);
      setQuickText('');
    });
  }

  /**
   * Run a feed action like {@link attempt}. A holiday pack with more than one
   * location is refused by the server, which imports such a pack only from a
   * file, after the user picks; that refusal gets its own message.
   */
  async function attemptFeed(failKey: string, fn: () => Promise<void>): Promise<void> {
    setFeedback(null);
    try {
      await fn();
    } catch (err) {
      const many = err instanceof FeedError && err.code === 'holMultipleLocations';
      setFeedback({ kind: 'error', text: t(many ? 'calendar-subscribe-hol-many-locations' : failKey) });
    }
  }

  async function onWebcalSubscribe(): Promise<void> {
    const url = webcalUrl().trim();
    if (url === '') return;
    await attemptFeed('calendar-subscribe-failed', async () => {
      await c.subscribeUrl(url);
      setWebcalUrl('');
    });
  }

  async function onRefreshFeed(id: string): Promise<void> {
    await attemptFeed('calendar-subscribe-refresh-failed', () => c.refreshSubscription(id));
  }

  function onCategoryFilter(value: string): void {
    setCatFilter(value);
    c.setCategoryFilter(value === '' ? null : value);
  }

  let importInput: HTMLInputElement | undefined;
  handleModuleActions('calendar',action=>{if(action==='new')openNew();else if(action==='import')queueMicrotask(()=>importInput?.click());});
  const calendarMenu=createEntityContextMenu({items:c.calendars,id:cal=>cal.id,name:cal=>cal.name,scope:c.identity,onError:()=>setFeedback({kind:'error',text:t('mail-context-failed')}),actions:cal=>[
    {id:'new',label:t('calendar-new-event'),icon:'plus',disabled:cal.isReadOnlyOverlay,run:()=>openNew(cal.id)},
    {id:'visible',label:t('common-context-show'),icon:'eye',checked:cal.isVisible,run:()=>c.toggleCalendar(cal.id)},
    {id:'only',label:t('common-context-show-only'),icon:'calendar',run:async()=>{for(const other of c.calendars())if(other.isVisible!==(other.id===cal.id))await c.toggleCalendar(other.id);}},
    {id:'color',label:t('common-context-color'),icon:'palette',run:()=>undefined,children:[['green','#22c55e'],['blue','#3b82f6'],['violet','#a855f7'],['red','#ef4444'],['orange','#f97316'],['yellow','#eab308']].map(([name,color])=>({id:color!,label:t(`appearance-accent-${name}`),icon:'palette',radio:true,checked:cal.color.toLowerCase()===color,run:()=>c.setCalendarColor(cal.id,color!)}))},
    {id:'share',label:t('calendar-share'),icon:'share',disabled:cal.isReadOnlyOverlay,separatorBefore:true,run:()=>{setSharing(cal);}},
    {id:'export',label:t('common-context-export'),icon:'download',run:async()=>triggerDownload('calendar.ics',await c.exportIcs({calendarId:cal.id}))},
    {id:'refresh',label:t('common-refresh'),icon:'sync',run:()=>cal.isReadOnlyOverlay&&cal.caldavUrl?c.refreshSubscription(cal.id):c.load()},
  ]});
  const eventMenu=createEntityContextMenu({items:c.masters,id:event=>event.id,name:event=>event.title,scope:c.identity,onError:()=>setFeedback({kind:'error',text:t('mail-context-failed')}),actions:event=>{
    const readOnly=c.calendars().find(cal=>cal.id===event.calendarId)?.isReadOnlyOverlay??true;
    return [
      {id:'open',label:t('common-open'),icon:'calendar',run:()=>openEvent(event.id)},
      {id:'duplicate',label:t('common-duplicate'),icon:'copy',disabled:readOnly,run:async()=>{const id=await c.createEvent({calendarId:event.calendarId,title:event.title,description:event.description,start:event.start,timeZone:event.timeZone,duration:event.duration,showWithoutTime:event.showWithoutTime,locations:event.locations});if(id)openEvent(id);}},
      {id:'export',label:t('common-context-export'),icon:'download',run:async()=>triggerDownload('event.ics',await c.exportIcs({eventIds:[event.id]}))},
      {id:'delete',label:t('common-delete'),icon:'delete',disabled:readOnly,separatorBefore:true,run:()=>c.deleteEvent(event.id)},
    ];
  }});
  const [contextDate,setContextDate]=createSignal('');
  const dateMenu=createEntityContextMenu({items:()=>contextDate()?[{id:contextDate(),name:contextDate()}]:[],id:item=>item.id,name:item=>item.name,scope:c.identity,actions:item=>[
    {id:'new',label:t('calendar-new-event'),icon:'plus',disabled:!c.calendars().some(cal=>!cal.isReadOnlyOverlay),run:()=>openNewAt(new Date(`${item.id}T09:00:00`))},
    {id:'day',label:t('calendar-view-day'),icon:'calendar',run:()=>{c.goToDate(new Date(`${item.id}T12:00:00`));c.setView('day');}},
    {id:'week',label:t('calendar-view-week'),icon:'calendar',run:()=>{c.goToDate(new Date(`${item.id}T12:00:00`));c.setView('week');}},
  ]});
  const eventContext=(event:MouseEvent|KeyboardEvent)=>{
    if(event.type!=='contextmenu'&&(event as KeyboardEvent).key!=='ContextMenu'&&!((event as KeyboardEvent).shiftKey&&(event as KeyboardEvent).key==='F10'))return;
    const id=(event.target as Element).closest<HTMLElement>('[data-calendar-event-id]')?.dataset.calendarEventId;
    const date=(event.target as Element).closest<HTMLElement>('[data-date]')?.dataset.date;
    if(!id&&(!date||!/^\d{4}-\d{2}-\d{2}$/.test(date)))return;
    if(!id)setContextDate(date!);
    const binding=id?eventMenu.bind(id):dateMenu.bind(date!);if(event.type==='contextmenu')binding.onContextMenu(event as MouseEvent);else binding.onKeyDown(event as KeyboardEvent);
  };

  return (
    <section class={css.module} aria-label={t('calendar-title')} data-module="calendar" onContextMenu={eventContext} onKeyDown={eventContext}>
      <div class={css.toolbar}>
        <span class={css.title} aria-live="polite">{headerLabel(c)}</span>
        <button type="button" class={css.button} onClick={() => c.goPrev()} aria-label={t('calendar-prev')}><Icon name="chevronLeft" /></button>
        <button type="button" class={css.button} onClick={() => c.goToday()}>{t('calendar-today')}</button>
        <button type="button" class={css.button} onClick={() => c.goNext()} aria-label={t('calendar-next')}><Icon name="chevron" /></button>
        <div class={css.viewSwitch} role="tablist" aria-label={t('calendar-views')}>
          <For each={CALENDAR_VIEWS}>
            {(view) => (
              <button
                type="button"
                role="tab"
                aria-selected={c.view() === view.id}
                class={c.view() === view.id ? css.viewButton.active : css.viewButton.base}
                onClick={() => c.setView(view.id as CalendarView)}
              >
                {t(`calendar-view-${view.id}`)}
              </button>
            )}
          </For>
        </div>
        <span class={css.spacer} />
        <Show when={c.conflicts().length > 0}>
          <button
            type="button"
            class={css.conflictButton}
            onClick={() => setResolverOpen(true)}
            aria-label={t('calendar-resolve-conflicts', { count: c.conflicts().length })}
          >
            {t('calendar-resolve-conflicts', { count: c.conflicts().length })}
          </button>
        </Show>
        <button type="button" class={css.primaryButton} onClick={()=>openNew()}>{t('calendar-new-event')}</button>
        <input
          class={`${css.input} ${css.quickAdd}`}
          placeholder={t('calendar-quick-add-placeholder')}
          value={quickText()}
          onInput={(e) => setQuickText(e.currentTarget.value)}
          onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), void onQuickAdd())}
          aria-label={t('calendar-quick-add')}
        />
        <button type="button" class={css.button} onClick={() => void onQuickAdd()} aria-label={t('calendar-quick-add-do')}>{t('calendar-quick-add-btn')}</button>
        <button type="button" class={css.button} onClick={() => importInput?.click()}>{t('calendar-import')}</button>
        <button type="button" class={css.button} onClick={() => void onExport()}>{t('calendar-export')}</button>
        <input
          ref={importInput}
          type="file"
          accept=".ics,.hol,text/calendar"
          class={css.fileInput}
          aria-label={t('calendar-import-file')}
          onChange={(e) => void onImportFile(e)}
        />
      </div>

      <button type="button" class={css.mobileSidebarToggle} aria-expanded={mobileCalendars()} aria-controls={calendarSidebarId} onClick={()=>setMobileCalendars(!mobileCalendars())}>{t('calendar-calendars-heading')}</button>
      <div class={css.body}>
        <aside id={calendarSidebarId} class={`${css.sidebar} ${mobileCalendars()?'':css.mobileSidebarClosed}`} aria-label={t('calendar-calendars-heading')}>
          <MiniCalendar date={c.focusDate()} select={c.goToDate}/>
          <div>
            <h3 class={css.sideHeading}>{t('calendar-calendars-heading')}</h3>
            <ul class={css.calList}>
              {/* Index rather than For (t32-h12): toggling a calendar replaces its
                  object, and a re-created row would drop keyboard focus from the checkbox. */}
              <Index each={c.calendars()}>
                {(cal) => (
                  <li class={css.calItem} tabIndex={0} {...calendarMenu.bind(cal().id)}>
                    <input
                      type="checkbox"
                      checked={cal().isVisible}
                      onChange={() => void c.toggleCalendar(cal().id)}
                      aria-label={t('calendar-toggle', { name: cal().name })}
                    />
                    <ColorField
                      value={cal().color}
                      label={t('calendar-color-for', { name: cal().name })}
                      showValue={false}
                      testId="calendar-color"
                      contrast={{ against: 'surface', use: 'non-text' }}
                      onChange={(color) => { if (color) void c.setCalendarColor(cal().id, color); }}
                    />
                    <span class={css.grow}><bdi>{cal().name}</bdi></span>
                    <Show when={cal().isReadOnlyOverlay || cal().caldavUrl !== null}>
                      <span class={css.dimText} title={cal().caldavUrl ?? t('calendar-synced')} aria-label={t('calendar-synced')}><Icon name="sync" /></span>
                    </Show>
                    <Show when={cal().isReadOnlyOverlay && cal().caldavUrl !== null}>
                      <button
                        type="button"
                        class={css.button}
                        onClick={() => void onRefreshFeed(cal().id)}
                        aria-label={t('calendar-subscribe-refresh', { name: cal().name })}
                      >
                        {t('calendar-subscribe-refresh-btn')}
                      </button>
                    </Show>
                    <button
                      type="button"
                      class={css.button}
                      onClick={() => setSharing(cal())}
                      aria-label={t('calendar-share-for', { name: cal().name })}
                    >
                      {t('calendar-share')}
                    </button>
                  </li>
                )}
              </Index>
            </ul>
            <button type="button" class={`${css.button} ${css.addCalendar}`} onClick={() => void c.createCalendar(rawT('calendar-new-calendar-name'), '#22c55e')}>
              {t('calendar-add-calendar')}
            </button>
          </div>

          <div>
            <h3 class={css.sideHeading}>{t('calendar-filter-heading')}</h3>
            <input
              class={css.input}
              placeholder={t('calendar-filter-category-placeholder')}
              value={catFilter()}
              onInput={(e) => onCategoryFilter(e.currentTarget.value)}
              aria-label={t('calendar-filter-category')}
            />
          </div>

          <div>
            <h3 class={css.sideHeading}>{t('calendar-subscribe-heading')}</h3>
            <div class={css.row}>
              <input
                class={css.input}
                type="url"
                placeholder={t('calendar-subscribe-url-placeholder')}
                value={webcalUrl()}
                onInput={(e) => setWebcalUrl(e.currentTarget.value)}
                onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), void onWebcalSubscribe())}
                aria-label={t('calendar-subscribe-url')}
              />
              <button type="button" class={css.button} onClick={() => void onWebcalSubscribe()}>
                {t('calendar-subscribe-add')}
              </button>
            </div>
          </div>

          <div>
            <h3 class={css.sideHeading}>{t('calendar-holidays-heading')}</h3>
            <select class={css.input} aria-label={t('calendar-subscribe-holidays')} onChange={(e) => { const v = e.currentTarget.value; if (v !== '') void subscribeHoliday(v); e.currentTarget.value = ''; }}>
              <option value="">{t('calendar-add-region')}</option>
              <For each={HOLIDAY_PACKS}>{(p) => <option value={p.id}>{p.label}</option>}</For>
            </select>
          </div>

          <Show when={c.error() !== null}>
            <p class={css.dangerText} role="alert">{c.error()}</p>
          </Show>
        </aside>

        <main class={css.main} aria-busy={c.loading()}>
          <Show when={c.loading()}><InlineLoading /></Show>
          <ActiveView controller={c} onOpenEvent={(inst) => openEvent(inst.event.id)} onNewAt={(day) => openNewAt(day)} />
        </main>
      </div>

      <Show when={editorOpen()}>
        <EventEditor controller={c} event={editing()} defaultStart={defaultStart()} defaultCalendarId={defaultCalendar()} onClose={() => setEditorOpen(false)} />
      </Show>

      <Show when={resolverOpen()}>
        <ConflictResolver controller={c} onClose={() => setResolverOpen(false)} />
      </Show>

      <Show when={sharing()}>
        {(cal) => <ShareDialog controller={c} calendar={cal()} onClose={() => setSharing(null)} />}
      </Show>

      <Show when={locationChoice()}>
        {(locations) => (
          <HolLocationPicker
            locations={locations()}
            onImport={(chosen) => void onLocationsChosen(chosen)}
            onClose={() => setLocationChoice(null)}
          />
        )}
      </Show>
      {calendarMenu.menu()}{eventMenu.menu()}{dateMenu.menu()}
    </section>
  );
}

/** The module over the in-memory mock (the app shell mounts `CalendarApp` over the engine instead). */
export function CalendarModule(): JSX.Element {
  const controller = makeMockController();
  return <CalendarApp controller={controller} />;
}
