// Settings > Retention: calendar events and notes. Two opt-in cards, off unless
// the administrator allows them, and a list of removed notes that can still be
// restored. The day limits come from the server; it enforces them again on save.
import { createResource, createSignal, For, Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { formatDateTime } from '../../i18n/format.ts';
import * as api from '../../api/retention.ts';
import { RetentionError, type PimOverview, type PimSummary } from '../../api/retention.ts';
import { SettingsCard } from './layout/SettingsCard.tsx';
import { ControlRow } from './layout/SettingRow.tsx';
import type { CardSpec } from './layout/rows.tsx';
import * as css from './styles.css.ts';

export interface PimApi {
  load: typeof api.loadPim;
  save: typeof api.savePim;
  preview: typeof api.previewPim;
  run: typeof api.runPim;
  restore: typeof api.restoreNote;
}

export const livePim: PimApi = { load: api.loadPim, save: api.savePim, preview: api.previewPim, run: api.runPim, restore: api.restoreNote };

/** Static description of the cards, for Settings search. */
export const PIM_RETENTION_CARDS: readonly CardSpec[] = [
  { id: 'pim-card-calendar', section: 'retention', title: 'pim-cal-title', help: 'pim-cal-help', rows: [{ id: 'pim-cal-enabled', label: 'pim-cal-enabled' }, { id: 'pim-cal-days', label: 'pim-cal-days' }] },
  { id: 'pim-card-notes', section: 'retention', title: 'pim-notes-title', rows: [{ id: 'pim-notes-enabled', label: 'pim-notes-enabled' }, { id: 'pim-notes-days', label: 'pim-notes-days' }] },
];

const when = (seconds: number): string => formatDateTime(seconds * 1000);

export function PimRetention(props: { api?: PimApi }): JSX.Element {
  const a = props.api ?? livePim;
  const [data, { refetch }] = createResource(() => a.load());
  const [message, setMessage] = createSignal('');
  const [result, setResult] = createSignal<PimSummary | null>(null);
  const [busy, setBusy] = createSignal(false);
  // Edits are kept per field; absent means "as saved".
  const [calOn, setCalOn] = createSignal<boolean | null>(null);
  const [calDays, setCalDays] = createSignal<number | null>(null);
  const [calIds, setCalIds] = createSignal<string[] | null>(null);
  const [notesOn, setNotesOn] = createSignal<boolean | null>(null);
  const [notesDays, setNotesDays] = createSignal<number | null>(null);

  async function guarded(work: () => Promise<void>): Promise<void> {
    setBusy(true); setMessage('');
    try { await work(); } catch (e) { setMessage(e instanceof RetentionError ? e.message : t('retention-error')); } finally { setBusy(false); }
  }
  const calendarCfg = (d: PimOverview) => ({
    enabled: calOn() ?? d.calendar.enabled, days: calDays() ?? (d.calendar.days || d.policy.calendar.minDays), calendars: calIds() ?? d.calendar.calendars,
  });
  const notesCfg = (d: PimOverview) => ({ enabled: notesOn() ?? d.notes.enabled, days: notesDays() ?? (d.notes.days || d.policy.notes.minDays) });
  const problem = (days: number, min: number, max: number): string | null =>
    !Number.isInteger(days) || days < min || days > max ? t('pim-days-range', { min, max }) : null;

  return <Show when={data.latest}>{(d) => <>
    <SettingsCard id="pim-card-calendar" title={t('pim-cal-title')} description={t('pim-cal-help')} testId="pim-calendar">
      <Show when={d().policy.calendar.allowed} fallback={<p class={css.meta} data-testid="pim-calendar-off">{t('pim-off')}</p>}>
        <ControlRow controlId="pim-cal-enabled" label={t('pim-cal-enabled')}>{() =>
          <input id="pim-cal-enabled" type="checkbox" class={css.checkbox} checked={calendarCfg(d()).enabled} disabled={busy()} onChange={(e) => setCalOn(e.currentTarget.checked)} />}</ControlRow>
        <ControlRow controlId="pim-cal-days" label={t('pim-cal-days')} help={t('pim-limits', { min: d().policy.calendar.minDays, max: d().policy.calendar.maxDays })}>{(by) =>
          <input id="pim-cal-days" type="number" class={css.input} aria-describedby={by()} min={d().policy.calendar.minDays} max={d().policy.calendar.maxDays} value={calendarCfg(d()).days} disabled={busy()} onInput={(e) => setCalDays(e.currentTarget.valueAsNumber)} />}</ControlRow>
        <fieldset class={css.bareFieldset}>
          <legend class={css.subheading}>{t('pim-cal-calendars')}</legend>
          <For each={d().calendars}>{(c) => <label class={css.check}>
            <input type="checkbox" class={css.checkbox} checked={calendarCfg(d()).calendars.includes(c.id)} disabled={busy()}
              onChange={(e) => { const cur = calendarCfg(d()).calendars; setCalIds(e.currentTarget.checked ? [...cur, c.id] : cur.filter((x) => x !== c.id)); }} />{c.name}
          </label>}</For>
        </fieldset>
        <div class={css.actions}>
          <button type="button" class={css.button} disabled={busy() || (calendarCfg(d()).enabled && problem(calendarCfg(d()).days, d().policy.calendar.minDays, d().policy.calendar.maxDays) !== null)} data-testid="pim-calendar-save"
            onClick={() => void guarded(async () => { await a.save({ calendar: calendarCfg(d()) }); setCalOn(null); setCalDays(null); setCalIds(null); await refetch(); })}>{t('pim-save')}</button>
        </div>
        <Show when={calendarCfg(d()).enabled && problem(calendarCfg(d()).days, d().policy.calendar.minDays, d().policy.calendar.maxDays)}>{(p) => <p role="alert" class={css.error}>{p()}</p>}</Show>
      </Show>
    </SettingsCard>

    <SettingsCard id="pim-card-notes" title={t('pim-notes-title')} description={t('pim-notes-help', { days: d().policy.undoDays })} testId="pim-notes">
      <Show when={d().policy.notes.allowed} fallback={<p class={css.meta} data-testid="pim-notes-off">{t('pim-off')}</p>}>
        <ControlRow controlId="pim-notes-enabled" label={t('pim-notes-enabled')}>{() =>
          <input id="pim-notes-enabled" type="checkbox" class={css.checkbox} checked={notesCfg(d()).enabled} disabled={busy()} onChange={(e) => setNotesOn(e.currentTarget.checked)} />}</ControlRow>
        <ControlRow controlId="pim-notes-days" label={t('pim-notes-days')} help={t('pim-limits', { min: d().policy.notes.minDays, max: d().policy.notes.maxDays })}>{(by) =>
          <input id="pim-notes-days" type="number" class={css.input} aria-describedby={by()} min={d().policy.notes.minDays} max={d().policy.notes.maxDays} value={notesCfg(d()).days} disabled={busy()} onInput={(e) => setNotesDays(e.currentTarget.valueAsNumber)} />}</ControlRow>
        <div class={css.actions}>
          <button type="button" class={css.button} disabled={busy() || (notesCfg(d()).enabled && problem(notesCfg(d()).days, d().policy.notes.minDays, d().policy.notes.maxDays) !== null)} data-testid="pim-notes-save"
            onClick={() => void guarded(async () => { await a.save({ notes: notesCfg(d()) }); setNotesOn(null); setNotesDays(null); await refetch(); })}>{t('pim-save')}</button>
        </div>
        <Show when={notesCfg(d()).enabled && problem(notesCfg(d()).days, d().policy.notes.minDays, d().policy.notes.maxDays)}>{(p) => <p role="alert" class={css.error}>{p()}</p>}</Show>
      </Show>
    </SettingsCard>

    <Show when={d().policy.calendar.allowed || d().policy.notes.allowed}>
      <SettingsCard id="pim-card-run" title={t('pim-run-title')} testId="pim-run">
        <div class={css.actions}>
          <button type="button" class={css.ghost} disabled={busy()} data-testid="pim-preview" onClick={() => void guarded(async () => { setResult(await a.preview()); })}>{t('pim-preview')}</button>
          <button type="button" class={css.button} disabled={busy()} data-testid="pim-run-now" onClick={() => void guarded(async () => { setResult(await a.run()); await refetch(); })}>{t('pim-run')}</button>
        </div>
        <Show when={result()}>{(r) => <div role="status" data-testid="pim-result">
          <p>{r().skipped ? t(`retention-skipped-${r().skipped}`) : t('pim-result', { events: r().eventsDeleted || r().eventsMatched, notes: r().notesRemoved || r().notesMatched })}</p>
          <Show when={r().disabledSkipped > 0}><p class={css.meta}>{t('pim-skipped', { count: r().disabledSkipped })}</p></Show>
        </div>}</Show>
      </SettingsCard>
    </Show>

    <Show when={d().held.length > 0}>
      <SettingsCard id="pim-card-held" title={t('pim-held-title')} wide testId="pim-held">
        <ul class={css.list}>
          <For each={d().held}>{(h) => <li class={css.item}>
            <span class={css.itemMain}><span class={css.itemName}>{h.title || t('pim-untitled')}</span><span class={css.meta}>{t('pim-held-until', { when: when(h.dueAt) })}</span></span>
            <button type="button" class={css.ghost} disabled={busy()} data-testid={`pim-restore-${h.noteId}`}
              onClick={() => void guarded(async () => { await a.restore(h.noteId); await refetch(); })}>{t('pim-restore')}</button>
          </li>}</For>
        </ul>
      </SettingsCard>
    </Show>
    <Show when={message()}><p role="alert" class={css.error}>{message()}</p></Show>
  </>}</Show>;
}
