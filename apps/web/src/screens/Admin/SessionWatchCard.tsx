// Security centre > Adaptive protection > Session watch: a read-only summary of
// password-spray tracking, instance-wide spike detection (heightened mode),
// session binding, theft detection, country-change alerts and the recent alerts.

import { createSignal, For, onMount, Show, useContext, type JSX } from 'solid-js';
import { sessionWatchApi, type SessionWatch, type SessionWatchApi } from '../../api/admin/sessionWatch.ts';
import { formatDateTime, formatNumber, t } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import * as css from './admin.css.ts';

const when = (ts: number): string => formatDateTime(new Date(ts < 1e12 ? ts * 1000 : ts).toISOString());
const onOff = (v: boolean | undefined): string => (v === false ? t('admin-session-watch-off') : t('admin-session-watch-on'));

export function SessionWatchCard(props: { api?: SessionWatchApi }): JSX.Element {
  const shell = useContext(ShellContext);
  const api = (): SessionWatchApi => props.api ?? sessionWatchApi;
  const [data, setData] = createSignal<SessionWatch>();
  const [error, setError] = createSignal(false);
  const allowed = (): boolean => !shell || shell.can.can('security.read');
  onMount(() => {
    if (!allowed()) return;
    api().get().then(setData, () => setError(true));
  });
  const progress = (spike: SessionWatch['spike']): string =>
    spike.warmedUp
      ? t('admin-session-watch-warm')
      : t('admin-session-watch-warming', { done: spike.observedBuckets, total: spike.warmupBuckets });
  return (
    <Show when={allowed()}>
      <div class={css.card} data-testid="session-watch">
        <h3 class={css.heading}>{t('admin-session-watch-title')}</h3>
        <p class={css.note}>{t('admin-session-watch-note')}</p>
        <Show when={error()}>
          <p role="alert">{t('admin-session-watch-error')}</p>
        </Show>
        <Show when={data()}>
          {(d) => (
            <>
              <h4>{t('admin-session-watch-spike')}</h4>
              <p data-testid="session-watch-heightened">
                {d().spike.enabled
                  ? d().spike.heightened
                    ? t('admin-session-watch-heightened', { until: d().spike.heightenedUntil ? when(d().spike.heightenedUntil!) : '-' })
                    : t('admin-session-watch-normal')
                  : t('admin-session-watch-spike-off')}
                {' · '}
                {progress(d().spike)}
              </p>
              <Show when={d().spike.metrics.length > 0}>
                <table>
                  <thead>
                    <tr>
                      <th>{t('admin-session-watch-metric')}</th>
                      <th>{t('admin-session-watch-current')}</th>
                      <th>{t('admin-session-watch-baseline')}</th>
                      <th>{t('admin-session-watch-floor')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={d().spike.metrics}>
                      {(m) => (
                        <tr>
                          <td>{m.metric}</td>
                          <td>{formatNumber(m.current)}</td>
                          <td>{formatNumber(m.baseline)}</td>
                          <td>{formatNumber(m.floor)}</td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </Show>
              <h4>{t('admin-session-watch-spray')}</h4>
              <p>
                {d().spray.enabled
                  ? t('admin-session-watch-spray-line', {
                      tracked: d().spray.trackedSources,
                      atChallenge: d().spray.sourcesAtChallengeOrAbove,
                      challenge: d().spray.challengeAt,
                      pace: d().spray.paceAt,
                      lock: d().spray.lockAt,
                      window: d().spray.windowSecs,
                    })
                  : t('admin-session-watch-spray-off')}
              </p>
              <h4>{t('admin-session-watch-policy')}</h4>
              <ul>
                <li>
                  {t('admin-session-watch-binding', {
                    state: onOff(d().sessionBinding.enabled),
                    strictness: d().sessionBinding.strictness ?? '-',
                    webmail: d().sessionBinding.webmailAction ?? '-',
                    admin: d().sessionBinding.adminAction ?? '-',
                    attributes: (d().sessionBinding.attributes ?? []).join(', ') || '-',
                  })}
                </li>
                <li>
                  {t('admin-session-watch-theft', {
                    state: onOff(d().theft.enabled),
                    travel: d().theft.minTravelSecs ?? 0,
                    concurrent: d().theft.concurrentUse ?? '-',
                    webmail: d().theft.webmailAction ?? '-',
                    admin: d().theft.adminAction ?? '-',
                  })}
                </li>
                <li>
                  {t('admin-session-watch-country', {
                    state: onOff(d().countryChange.enabled),
                    mute: d().countryChange.userMuteAllowed ? t('admin-session-watch-on') : t('admin-session-watch-off'),
                    cooldown: d().countryChange.cooldownSecs ?? 0,
                    email: onOff(d().countryChange.emailEnabled),
                  })}
                </li>
              </ul>
              <h4>{t('admin-session-watch-recent')}</h4>
              <Show when={d().recentAlerts.length > 0} fallback={<p class={css.note}>{t('admin-session-watch-recent-empty')}</p>}>
                <table>
                  <thead>
                    <tr>
                      <th>{t('admin-session-watch-time')}</th>
                      <th>{t('admin-session-watch-kind')}</th>
                      <th>{t('admin-session-watch-countries')}</th>
                      <th>{t('admin-session-watch-session')}</th>
                      <th>{t('admin-session-watch-action')}</th>
                      <th>{t('admin-session-watch-surface')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={d().recentAlerts}>
                      {(a) => (
                        <tr>
                          <td>{when(a.ts)}</td>
                          <td>{a.kind}</td>
                          <td>{a.fromCountry || a.toCountry ? `${a.fromCountry ?? '-'} > ${a.toCountry ?? '-'}` : '-'}</td>
                          <td>{a.session ?? '-'}</td>
                          <td>{a.action}</td>
                          <td>{a.surface ?? '-'}</td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </Show>
              <p class={css.note}>{d().caveat ?? t('admin-session-watch-caveat')}</p>
            </>
          )}
        </Show>
      </div>
    </Show>
  );
}
export default SessionWatchCard;
