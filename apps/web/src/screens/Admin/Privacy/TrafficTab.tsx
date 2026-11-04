import { For, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { AsyncPending } from '../../../components/AsyncState.tsx';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';
import { trafficTime, type PrivacyTabContext } from './model.ts';
import { ProxyCharts } from './ProxyCharts.tsx';

export default function TrafficTab(props: { ctx: PrivacyTabContext }): JSX.Element {
  const ctx = (): PrivacyTabContext => props.ctx;
  return (
    <AdminSubsection title={t('privacy-process-stats')} actions={<button class="btn" disabled={ctx().statsLoading()} onClick={() => ctx().refresh()}>{t('privacy-refresh')}</button>}>
      <Show when={ctx().statsError()}><p role="alert">{describeAdminError(ctx().statsError())}</p></Show>
      <Show when={ctx().statsLoading() && !ctx().stats()}><AsyncPending /></Show>
      <Show when={!ctx().statsError() ? ctx().stats() : undefined}>{(value) => <div class={css.section} aria-busy={ctx().statsLoading()}>
        <div class={css.tableWrap} tabindex="0" role="region" aria-label={t('privacy-traffic-summary')}>
          <table class={css.table} aria-label={t('privacy-traffic-summary')}>
            <thead><tr><th scope="col">{t('privacy-traffic-class')}</th><th scope="col">{t('privacy-allowed')}</th><th scope="col">{t('privacy-refused')}</th><th scope="col">{t('privacy-failed')}</th><th scope="col">{t('privacy-total-duration')}</th></tr></thead>
            <tbody><For each={value().traffic} fallback={<tr><td colspan="5">{t('privacy-no-traffic')}</td></tr>}>{(row) => <tr><th scope="row">{row.class.replaceAll('_', ' ')}</th><td>{row.allowed}</td><td>{row.refused}</td><td>{row.failed}</td><td>{trafficTime(value().traffic)(row)}</td></tr>}</For></tbody>
          </table>
        </div>
        <div class={css.tableWrap} tabindex="0" role="region" aria-label={t('privacy-cache-summary')}>
          <table class={css.table} aria-label={t('privacy-cache-summary')}>
            <thead><tr><th scope="col">{t('privacy-metric')}</th><th scope="col">{t('privacy-value')}</th></tr></thead>
            <tbody>
              <tr><th scope="row">{t('privacy-cache-entries')}</th><td>{value().cacheEntries}</td></tr>
              <tr><th scope="row">{t('privacy-cache-bytes')}</th><td>{value().cacheBytes}</td></tr>
              <tr><th scope="row">{t('privacy-replaced-trackers')}</th><td>{value().trackersReplaced}</td></tr>
            </tbody>
          </table>
        </div>
        <ProxyCharts stats={value()} loading={ctx().statsLoading()} error={ctx().statsError()} view="traffic" />
      </div>}</Show>
    </AdminSubsection>
  );
}
