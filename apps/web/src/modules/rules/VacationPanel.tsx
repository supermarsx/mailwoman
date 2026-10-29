import { createEffect, createSignal, on, Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import type { Vacation, VacationApi } from './vacation-api.ts';
import * as css from './styles.css.ts';

/** `runsAt` says who sends the reply (see `vacationSupport`); the panel is only
 *  mounted when something will. */
export function VacationPanel(props: { api: VacationApi; runsAt?: 'server-sieve' | 'engine' | 'server' }): JSX.Element {
  const [value, setValue] = createSignal<Vacation | null>(null);
  const [state, setState] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [saved, setSaved] = createSignal<null | 'on' | 'off'>(null);
  createEffect(on(() => props.api, (api) => {
    setValue(null); setError(''); setSaved(null); setBusy(true);
    void api.get().then((response) => { if (api === props.api) { setValue(response.value); setState(response.state); } })
      .catch((failure: unknown) => { if (api === props.api) setError(failure instanceof Error ? failure.message : String(failure)); })
      .finally(() => { if (api === props.api) setBusy(false); });
  }));
  function patch(next: Partial<Vacation>): void { setValue((old) => old ? { ...old, ...next } : old); setSaved(null); }
  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault(); const api = props.api; const draft = value(); if (!draft || busy()) return;
    setBusy(true); setError(''); setSaved(null);
    try {
      await api.save(draft, state()); const response = await api.get();
      if (api === props.api) { setValue(response.value); setState(response.state); setSaved(response.value.isEnabled ? 'on' : 'off'); }
    } catch (failure) { if (api === props.api) setError(failure instanceof Error ? failure.message : String(failure)); }
    finally { if (api === props.api) setBusy(false); }
  }
  const date = (v: string | null): string => v ? new Date(v).toISOString().slice(0, 16) : '';
  return <form class={css.panel} aria-busy={busy()} onSubmit={(event) => void save(event)}>
    <h3 class={css.heading}>{t('rules-vacation-title')}</h3>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <p class={css.prose} data-testid="vacation-runs-at">{t(props.runsAt === 'engine' ? 'rules-vacation-runs-engine' : 'rules-vacation-runs-server')}</p>
    <Show when={saved()}>{(how) => <p role="status">{t(how() === 'on' ? 'rules-vacation-saved' : 'rules-vacation-saved-off')}</p>}</Show>
    <Show when={value()}>{(current) => <>
      <label class={css.row}><input type="checkbox" checked={current().isEnabled} disabled={busy()} onChange={(event) => patch({ isEnabled: event.currentTarget.checked })} />{t('rules-vacation-enabled')}</label>
      <label class={css.label}>{t('rules-vacation-from')}<input type="datetime-local" class={css.input} value={date(current().fromDate)} disabled={busy()} onInput={(event) => patch({ fromDate: event.currentTarget.value ? `${event.currentTarget.value}:00Z` : null })} /></label>
      <label class={css.label}>{t('rules-vacation-to')}<input type="datetime-local" class={css.input} value={date(current().toDate)} disabled={busy()} onInput={(event) => patch({ toDate: event.currentTarget.value ? `${event.currentTarget.value}:00Z` : null })} /></label>
      <label class={css.label}>{t('rules-cond-subject')}<input class={css.input} value={current().subject ?? ''} disabled={busy()} onInput={(event) => patch({ subject: event.currentTarget.value || null })} /></label>
      <label class={css.label}>{t('rules-vacation-body')}<textarea class={css.input} rows={8} value={current().textBody ?? ''} disabled={busy()} onInput={(event) => patch({ textBody: event.currentTarget.value, htmlBody: null })} /></label>
      <button type="submit" class={css.btnPrimary} disabled={busy()}>{t('common-save')}</button>
    </>}</Show>
  </form>;
}
