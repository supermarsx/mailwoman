// Calendar module (plan §2.5, §3 e4): the mounted shell that composes the view
// switcher, the multi-calendar overlay sidebar, all nine views, the event
// create/edit dialog (recurrence + reminders + attendees + free/busy + invite
// controls), conflict badges, and ICS / `.hol` import + export.
//
// `CalendarModule` is the registry mount target (plan §2.5 `AppModule.mount`). It
// is MOCK-BACKED by default (engine is e8, mounting/real-surface swap is e10);
// e10 will pass an engine-backed controller instead. `CalendarApp` takes an
// explicit controller so views + tests drive it without the app store.

import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import type { CalendarEvent } from '../../api/pim-types.ts';
import { createCalendarController, type CalendarBackend, type CalendarController } from './controller.ts';
import { createMockStore, mockSession, createMockJmap, type MockStore } from './mock.ts';
import { CALENDAR_VIEWS, type CalendarView } from './types.ts';
import { ActiveView } from './views.tsx';
import { EventEditor } from './EventEditor.tsx';
import { formatFull, formatMonth, formatMonthYear } from './datetime.ts';
import { HOLIDAY_PACKS } from './holidays.ts';
import * as css from './calendar.css.ts';

/** Build a mock-backed controller (default until e10 wires the real engine). */
export function makeMockController(store: MockStore = createMockStore()): CalendarController {
  const jmap = createMockJmap(store);
  const backend: CalendarBackend = {
    jmap,
    resolveAccount: () =>
      Promise.resolve(mockSession().primaryAccounts['urn:mailwoman:calendars'] ?? null),
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

  onMount(() => {
    void c.load();
  });

  function openNew(): void {
    setEditing(null);
    setEditorOpen(true);
  }
  function openEvent(masterId: string): void {
    setEditing(c.masterById(masterId) ?? null);
    setEditorOpen(true);
  }

  async function onImportFile(e: Event): Promise<void> {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file === undefined) return;
    const text = await file.text();
    const target = c.visibleCalendars()[0]?.id ?? c.calendars()[0]?.id;
    if (target !== undefined) await c.importIcs(target, text);
    input.value = '';
  }

  async function onExport(): Promise<void> {
    const ics = await c.exportIcs({});
    triggerDownload('mailwoman-calendar.ics', ics);
  }

  async function subscribeHoliday(packId: string): Promise<void> {
    const pack = HOLIDAY_PACKS.find((p) => p.id === packId);
    const target = c.visibleCalendars()[0]?.id ?? c.calendars()[0]?.id;
    if (pack === undefined || target === undefined) return;
    await c.importIcs(target, pack.ics);
  }

  let importInput: HTMLInputElement | undefined;

  return (
    <section class={css.module} aria-label="Calendar" data-module="calendar">
      <div class={css.toolbar}>
        <span class={css.title}>{headerLabel(c)}</span>
        <button type="button" class={css.button} onClick={() => c.goPrev()} aria-label="Previous">‹</button>
        <button type="button" class={css.button} onClick={() => c.goToday()}>Today</button>
        <button type="button" class={css.button} onClick={() => c.goNext()} aria-label="Next">›</button>
        <div class={css.viewSwitch} role="tablist" aria-label="Calendar views">
          <For each={CALENDAR_VIEWS}>
            {(view) => (
              <button
                type="button"
                role="tab"
                aria-selected={c.view() === view.id}
                class={c.view() === view.id ? css.viewButton.active : css.viewButton.base}
                onClick={() => c.setView(view.id as CalendarView)}
              >
                {view.label}
              </button>
            )}
          </For>
        </div>
        <span class={css.spacer} />
        <button type="button" class={css.primaryButton} onClick={openNew}>+ Event</button>
        <button type="button" class={css.button} onClick={() => importInput?.click()}>Import</button>
        <button type="button" class={css.button} onClick={() => void onExport()}>Export</button>
        <input
          ref={importInput}
          type="file"
          accept=".ics,.hol,text/calendar"
          style={{ display: 'none' }}
          aria-label="Import calendar file"
          onChange={(e) => void onImportFile(e)}
        />
      </div>

      <div class={css.body}>
        <aside class={css.sidebar}>
          <div>
            <h3 style={{ margin: '0 0 0.25rem', 'font-size': '0.85rem' }}>Calendars</h3>
            <ul class={css.calList}>
              <For each={c.calendars()}>
                {(cal) => (
                  <li class={css.calItem}>
                    <input
                      type="checkbox"
                      checked={cal.isVisible}
                      onChange={() => void c.toggleCalendar(cal.id)}
                      aria-label={`Toggle ${cal.name}`}
                    />
                    <input
                      type="color"
                      value={cal.color}
                      onChange={(e) => void c.setCalendarColor(cal.id, e.currentTarget.value)}
                      aria-label={`Color for ${cal.name}`}
                      style={{ width: '1.4rem', height: '1.4rem', padding: 0, border: 'none', background: 'none' }}
                    />
                    <span>{cal.name}</span>
                    <Show when={cal.isReadOnlyOverlay || cal.caldavUrl !== null}>
                      <span class={css.dimText} title={cal.caldavUrl ?? 'overlay'}>⇅</span>
                    </Show>
                  </li>
                )}
              </For>
            </ul>
            <button type="button" class={css.button} style={{ 'margin-top': '0.5rem' }} onClick={() => void c.createCalendar('New calendar', '#22c55e')}>
              + Calendar
            </button>
          </div>

          <div>
            <h3 style={{ margin: '0 0 0.25rem', 'font-size': '0.85rem' }}>Holidays</h3>
            <select class={css.input} aria-label="Subscribe to holidays" onChange={(e) => { const v = e.currentTarget.value; if (v !== '') void subscribeHoliday(v); e.currentTarget.value = ''; }}>
              <option value="">Add a region…</option>
              <For each={HOLIDAY_PACKS}>{(p) => <option value={p.id}>{p.label}</option>}</For>
            </select>
          </div>

          <Show when={c.error() !== null}>
            <p class={css.dangerText}>{c.error()}</p>
          </Show>
        </aside>

        <main class={css.main}>
          <ActiveView controller={c} onOpenEvent={(inst) => openEvent(inst.event.id)} />
        </main>
      </div>

      <Show when={editorOpen()}>
        <EventEditor controller={c} event={editing()} onClose={() => setEditorOpen(false)} />
      </Show>
    </section>
  );
}

/** Registry mount target (plan §2.5). Mock-backed until e10 swaps in the engine. */
export function CalendarModule(): JSX.Element {
  const controller = makeMockController();
  return <CalendarApp controller={controller} />;
}
