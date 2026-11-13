// Security centre > Adaptive protection > Session watch: a read-only summary of
// password-spray tracking, instance-wide spike detection (heightened mode),
// session binding, theft detection, country-change alerts and the recent alerts.
// Laid out as tiles (state at a glance), a metrics table with a bar per metric
// against its floor, a policy list and a recent-alerts table with relative times.
// Rows stack into labelled cards under 560px.

import { createSignal, For, onMount, Show, useContext, type JSX } from 'solid-js';
import { sessionWatchApi, type SessionWatch, type SessionWatchApi } from '../../api/admin/sessionWatch.ts';
import { formatDateTime, formatNumber, formatPercent, formatRelative, t } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import { StatusBadge, type StatusTone } from './shell/StatusBadge.tsx';
import { adminDocsHref } from './shell/docsLink.ts';
import * as css from './admin.css.ts';
import * as pg from './protocolGuard.css.ts';
import {SessionNoticeDetails} from '../Settings/SessionNoticeDetails.tsx';

const iso = (ts: number): string => new Date(ts < 1e12 ? ts * 1000 : ts).toISOString();
const when = (ts: number): string => formatDateTime(iso(ts));
const onOff = (v: boolean | undefined): string => (v === false ? t('admin-session-watch-off') : t('admin-session-watch-on'));
const KNOWN_METRICS = ['failures', 'sources', 'usernames', 'failureRatio', 'distributedSources'];
const KNOWN_KINDS = ['countryChange', 'impossibleTravel', 'concurrentUse', 'countryUnavailable', 'fingerprintMismatch', 'rotationReuse'];
const metricName = (m: string): string => (KNOWN_METRICS.includes(m) ? t(`admin-session-watch-metric-${m}`) : m);
const kindName = (k: string): string => (KNOWN_KINDS.includes(k) ? t(`admin-session-watch-kind-${k}`) : k);
const metricValue = (metric: string, v: number): string => (metric === 'failureRatio' ? formatPercent(v) : formatNumber(v, { maximumFractionDigits: 1 }));

function Tile(props: { value: JSX.Element; label: string; hint?: string; caption?: string | undefined }): JSX.Element {
  return (
    <div class={pg.tile} title={props.hint}>
      <span class={typeof props.value === 'string' && props.value.length > 14 ? pg.tileValueCompact : pg.tileValue}>{props.value}</span>
      <span class={pg.tileLabel}>{props.label}</span>
      <Show when={props.caption}><span class={pg.tileCaption} data-testid="tile-caption">{props.caption}</span></Show>
    </div>
  );
}

function Policy(props: { label: string; enabled: boolean | undefined; detail: string }): JSX.Element {
  return (
    <li>
      <StatusBadge tone={props.enabled === false ? 'off' : 'ok'}>{props.label}: {onOff(props.enabled)}</StatusBadge> <span class={css.note}>{props.detail}</span>
    </li>
  );
}

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
  const stateTone = (s: SessionWatch['spike']): StatusTone => (!s.enabled ? 'off' : s.heightened ? 'warn' : 'ok');
  return (
    <Show when={allowed()}>
      <div class={css.card} data-testid="session-watch">
        <h3 class={css.heading}>{t('admin-session-watch-title')}</h3>
        <p class={css.note}>
          {t('admin-session-watch-note')} <a href={adminDocsHref('security-center')} target="_blank" rel="noopener noreferrer">{t('admin-ux-learn-more')}</a>
        </p>
        <Show when={error()}>
          <p role="alert">{t('admin-session-watch-error')}</p>
        </Show>
        <Show when={!data() && !error()}>
          <p class={css.note} role="status">{t('admin-session-watch-loading')}</p>
        </Show>
        <Show when={data()}>
          {(d) => (
            <>
              <div class={pg.tiles}>
                <Tile
                  label={t('admin-session-watch-spike')}
                  hint={t('admin-session-watch-spike-help')}
                  value={
                    <span data-testid="session-watch-heightened">
                      <StatusBadge tone={stateTone(d().spike)}>
                        {!d().spike.enabled
                          ? t('admin-session-watch-spike-off')
                          : d().spike.heightened
                            ? t('admin-session-watch-heightened', { until: d().spike.heightenedUntil ? when(d().spike.heightenedUntil!) : '-' })
                            : t('admin-session-watch-normal')}
                      </StatusBadge>
                    </span>
                  }
                />
                <Tile
                  label={d().spike.warmedUp ? t('admin-session-watch-warm') : t('admin-session-watch-baseline')}
                  hint={t('admin-session-watch-baseline-help')}
                  value={d().spike.warmedUp ? t('admin-session-watch-ready') : t('admin-session-watch-warming-value')}
                  caption={d().spike.warmedUp ? undefined : t('admin-session-watch-warming', { done: d().spike.observedBuckets, total: d().spike.warmupBuckets })}
                />
                <Tile
                  label={t('admin-session-watch-tile-sources')}
                  hint={t('admin-session-watch-tile-sources-help')}
                  value={d().spray.enabled ? formatNumber(d().spray.trackedSources) : t('admin-session-watch-off')}
                />
                <Tile
                  label={t('admin-session-watch-tile-challenged')}
                  hint={t('admin-session-watch-tile-challenged-help')}
                  value={d().spray.enabled ? formatNumber(d().spray.sourcesAtChallengeOrAbove) : t('admin-session-watch-off')}
                />
                <Tile label={t('admin-session-watch-tile-alerts')} value={formatNumber(d().recentAlerts.length)} />
              </div>

              <Show when={d().spike.metrics.length > 0}>
                <h4>{t('admin-session-watch-metrics')}</h4>
                <table class={pg.table} aria-label={t('admin-session-watch-metrics')}>
                  <thead>
                    <tr>
                      <th scope="col">{t('admin-session-watch-metric')}</th>
                      <th scope="col" class="num">{t('admin-session-watch-current')}</th>
                      <th scope="col" class="num" title={t('admin-session-watch-baseline-help')}>{t('admin-session-watch-baseline')}</th>
                      <th scope="col" class="num" title={t('admin-session-watch-floor-help')}>{t('admin-session-watch-floor')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={d().spike.metrics}>
                      {(m) => (
                        <tr>
                          <td class={pg.label} title={KNOWN_METRICS.includes(m.metric) ? t(`admin-session-watch-metric-help-${m.metric}`) : undefined}>
                            <span>{metricName(m.metric)}</span>
                            <progress class={pg.bar} max={Math.max(m.floor, m.current, 1)} value={m.current} aria-hidden="true" tabindex="-1" />
                          </td>
                          <td class="num" data-label={t('admin-session-watch-current')}>
                            <Show when={m.current > 0} fallback={<span class={pg.zero}>{metricValue(m.metric, 0)}</span>}>
                              <span class={pg.hit}>{m.floor > 0 && m.current >= m.floor ? <span aria-label={t('admin-session-watch-over-floor')}>! </span> : null}{metricValue(m.metric, m.current)}</span>
                            </Show>
                          </td>
                          <td class="num" data-label={t('admin-session-watch-baseline')}>{metricValue(m.metric, m.baseline)}</td>
                          <td class="num" data-label={t('admin-session-watch-floor')}>{metricValue(m.metric, m.floor)}</td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </Show>

              <h4>{t('admin-session-watch-spray')}</h4>
              <p class={css.note}>
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
                <Policy
                  label={t('admin-session-watch-policy-binding')}
                  enabled={d().sessionBinding.enabled}
                  detail={t('admin-session-watch-binding', {
                    state: onOff(d().sessionBinding.enabled),
                    strictness: d().sessionBinding.strictness ?? '-',
                    webmail: d().sessionBinding.webmailAction ?? '-',
                    admin: d().sessionBinding.adminAction ?? '-',
                    attributes: (d().sessionBinding.attributes ?? []).join(', ') || '-',
                  })}
                />
                <Policy
                  label={t('admin-session-watch-policy-theft')}
                  enabled={d().theft.enabled}
                  detail={t('admin-session-watch-theft', {
                    state: onOff(d().theft.enabled),
                    travel: d().theft.minTravelSecs ?? 0,
                    concurrent: d().theft.concurrentUse ?? '-',
                    webmail: d().theft.webmailAction ?? '-',
                    admin: d().theft.adminAction ?? '-',
                  })}
                />
                <Policy
                  label={t('admin-session-watch-policy-country')}
                  enabled={d().countryChange.enabled}
                  detail={t('admin-session-watch-country', {
                    state: onOff(d().countryChange.enabled),
                    mute: d().countryChange.userMuteAllowed ? t('admin-session-watch-on') : t('admin-session-watch-off'),
                    cooldown: d().countryChange.cooldownSecs ?? 0,
                    email: onOff(d().countryChange.emailEnabled),
                  })}
                />
              </ul>

              <h4>{t('admin-session-watch-recent')}</h4>
              <Show when={d().recentAlerts.length > 0} fallback={<p class={css.note}>{t('admin-session-watch-recent-empty')}</p>}>
                <table class={pg.table} aria-label={t('admin-session-watch-recent')}>
                  <thead>
                    <tr>
                      <th scope="col">{t('admin-session-watch-time')}</th>
                      <th scope="col">{t('admin-session-watch-kind')}</th>
                      <th scope="col">{t('admin-session-watch-countries')}</th>
                      <th scope="col">{t('admin-session-watch-session')}</th>
                      <th scope="col">{t('admin-session-watch-action')}</th>
                      <th scope="col">{t('admin-session-watch-surface')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={d().recentAlerts}>
                      {(a) => (
                        <tr>
                          <td data-label={t('admin-session-watch-time')}>
                            <time datetime={iso(a.ts)} title={when(a.ts)}>{formatRelative(iso(a.ts))}</time>
                          </td>
                          <td data-label={t('admin-session-watch-kind')}>{kindName(a.kind)}</td>
                          <td data-label={t('admin-session-watch-countries')}>{a.fromCountry || a.toCountry ? `${a.fromCountry ?? '-'} > ${a.toCountry ?? '-'}` : '-'}<Show when={a.context}><SessionNoticeDetails context={a.context}/></Show></td>
                          <td data-label={t('admin-session-watch-session')}>{a.session ?? '-'}</td>
                          <td data-label={t('admin-session-watch-action')}>{a.action}</td>
                          <td data-label={t('admin-session-watch-surface')}>{a.surface ?? '-'}</td>
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
