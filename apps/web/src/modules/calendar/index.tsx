// Calendar module (plan §2.5, §3 e4): the mounted shell that composes the view
// switcher, the multi-calendar overlay sidebar, all nine views, the event
// create/edit dialog (recurrence + reminders + attendees + free/busy + invite
// controls), conflict badges, and ICS / `.hol` import + export.
//
// `CalendarApp` takes an explicit controller so views + tests drive it without
// the app store. The app shell (`shell/mounts.tsx`) renders it over the
// engine-backed controller from `state/slices/calendar.ts`; `CalendarModule`
// below is the mock-backed variant.

import { For, Index, Show, createSignal, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
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
  const [editorOpen, setEditorOpen] = createSignal(false);
  const [editing, setEditing] = createSignal<CalendarEvent | null>(null);
  const [defaultStart, setDefaultStart] = createSignal<Date | undefined>(undefined);
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
    void c.load();
  });

  function openNew(): void {
    setEditing(null);
    setDefaultStart(undefined);
    setEditorOpen(true);
  }
  function openNewAt(day: Date): void {
    setEditing(null);
    setDefaultStart(day);
    setEditorOpen(true);
  }
  function openEvent(masterId: string): void {
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

  return (
    <section class={css.module} aria-label={t('calendar-title')} data-module="calendar">
      <div class={css.toolbar}>
        <span class={css.title} aria-live="polite">{headerLabel(c)}</span>
        <button type="button" class={css.button} onClick={() => c.goPrev()} aria-label={t('calendar-prev')}>‹</button>
        <button type="button" class={css.button} onClick={() => c.goToday()}>{t('calendar-today')}</button>
        <button type="button" class={css.button} onClick={() => c.goNext()} aria-label={t('calendar-next')}>›</button>
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
        <button type="button" class={css.primaryButton} onClick={openNew}>{t('calendar-new-event')}</button>
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

      <div class={css.body}>
        <aside class={css.sidebar} aria-label={t('calendar-calendars-heading')}>
          <div>
            <h3 class={css.sideHeading}>{t('calendar-calendars-heading')}</h3>
            <ul class={css.calList}>
              {/* Index rather than For (t32-h12): toggling a calendar replaces its
                  object, and a re-created row would drop keyboard focus from the checkbox. */}
              <Index each={c.calendars()}>
                {(cal) => (
                  <li class={css.calItem}>
                    <input
                      type="checkbox"
                      checked={cal().isVisible}
                      onChange={() => void c.toggleCalendar(cal().id)}
                      aria-label={t('calendar-toggle', { name: cal().name })}
                    />
                    <input
                      type="color"
                      value={cal().color}
                      onChange={(e) => void c.setCalendarColor(cal().id, e.currentTarget.value)}
                      aria-label={t('calendar-color-for', { name: cal().name })}
                      class={css.colorInput}
                    />
                    <span class={css.grow}><bdi>{cal().name}</bdi></span>
                    <Show when={cal().isReadOnlyOverlay || cal().caldavUrl !== null}>
                      <span class={css.dimText} title={cal().caldavUrl ?? t('calendar-synced')} aria-label={t('calendar-synced')}>⇅</span>
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
            <button type="button" class={`${css.button} ${css.addCalendar}`} onClick={() => void c.createCalendar(t('calendar-new-calendar-name'), '#22c55e')}>
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

        <main class={css.main}>
          <ActiveView controller={c} onOpenEvent={(inst) => openEvent(inst.event.id)} onNewAt={(day) => openNewAt(day)} />
        </main>
      </div>

      <Show when={editorOpen()}>
        <EventEditor controller={c} event={editing()} defaultStart={defaultStart()} onClose={() => setEditorOpen(false)} />
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
    </section>
  );
}

/** The module over the in-memory mock (the app shell mounts `CalendarApp` over the engine instead). */
export function CalendarModule(): JSX.Element {
  const controller = makeMockController();
  return <CalendarApp controller={controller} />;
}
