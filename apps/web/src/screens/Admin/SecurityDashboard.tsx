// Admin › Security centre › Overview (t97): a mini security dashboard.
//
// One read (`GET /security/dashboard`) fills every tile the signed-in admin may
// see; the server leaves out the rest, so a restricted admin gets fewer tiles,
// not empty ones. The first load shows tile skeletons in place (no page swap);
// later refreshes keep the last numbers on screen while they reload (t75), every
// 60 s while the tab is visible. Each tile links to its detail page.
//
// Charts follow the shell's Sparkline rules (one series per chart, a title and
// a text alternative, no legend). Source addresses arrive masked to a network
// prefix unless this admin can read the authentication logs.

import { createSignal, For, onCleanup, onMount, Show, useContext, type JSX } from 'solid-js';
import { loadSecurityDashboard, type CheckStatus, type SecurityDashboard } from '../../api/admin/securityDashboard.ts';
import { t, formatNumber, formatPercent, formatDateTime, formatLocale, formatTime } from '../../i18n';
import { AsyncError } from '../../components/AsyncState.tsx';
import { Icon } from '../../components/Icon.tsx';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { StatTile } from './shell/StatTile.tsx';
import { Sparkline } from './shell/Sparkline.tsx';
import * as shellCss from './shell/shell.css.ts';
import * as loading from '../../components/asyncState.css.ts';
import * as a11y from '../../components/mailA11y.css.ts';
import * as css from './securityDashboard.css.ts';

/** How often the Overview refreshes while visible. */
export const SECURITY_DASHBOARD_REFRESH_MS = 60_000;

/** Known audit actions with a friendly label; anything else shows as recorded. */
const AUDIT_LABELS = new Set([
  'setting-changed', 'twofa-policy-set', 'passkey-policy-refused', 'step-up-refused', 'step-up-locked',
  'step-up-required', 'step-up-granted', 'admin-console-path-changed', 'admin-console-path-applied',
  'admin-console-path-generated', 'protection-policy-changed',
]);
export function auditLabel(action: string): string {
  return AUDIT_LABELS.has(action) ? t(`admin-secdash-audit-${action}`) : action;
}
/** A country code as its name in the interface language ("PT" → "Portugal"). */
export function countryName(code: string): string {
  try {
    return new Intl.DisplayNames(formatLocale(), { type: 'region' }).of(code.toUpperCase()) ?? code;
  } catch {
    return code;
  }
}

export function SecurityDashboardView(props: { load?: () => Promise<SecurityDashboard> }): JSX.Element {
  const shell = useContext(ShellContext);
  const [data, setData] = createSignal<SecurityDashboard>();
  const [error, setError] = createSignal<unknown>();
  const [busy, setBusy] = createSignal(false);
  async function reload(): Promise<void> {
    if (busy()) return;
    setBusy(true);
    try {
      setData(await (props.load ?? loadSecurityDashboard)());
      setError(undefined);
    } catch (cause) {
      setError(cause);
    } finally {
      setBusy(false);
    }
  }
  onMount(() => {
    void reload();
    const timer = setInterval(() => {
      if (document.visibilityState === 'visible') void reload();
    }, SECURITY_DASHBOARD_REFRESH_MS);
    onCleanup(() => clearInterval(timer));
  });
  const go = (path: string) => (e: MouseEvent): void => {
    if (!shell) return;
    e.preventDefault();
    void shell.router.navigate(path);
  };
  const tab = (id: string): string => sectionPath('security', [id]);
  return (
    <div class={css.page} aria-busy={busy() ? 'true' : 'false'} data-testid="security-dashboard">
      <div class={css.head}>
        <p class={css.lede}>{t('admin-secdash-lede')}</p>
        <Show when={data()}>{(d) => <p class={css.updated}>{t('admin-secdash-updated', { time: formatTime(d().generatedAt * 1000) })}</p>}</Show>
        <button class="btn btn--ghost btn--sm" disabled={busy()} onClick={() => void reload()}>{t('admin-secdash-refresh')}</button>
      </div>
      <Show when={error() !== undefined && data() === undefined}>
        <AsyncError error={error()} onRetry={() => void reload()} />
      </Show>
      <Show when={data() === undefined && error() === undefined}>
        <div class={loading.skeletonCards} data-testid="security-dashboard-skeleton" aria-hidden="true">
          <For each={[0, 1, 2, 3, 4, 5]}>{() => <span class={loading.skeletonCard} />}</For>
        </div>
      </Show>
      <Show when={data()}>{(d) => <>
        <div class={shellCss.tiles}>
          <Show when={d().factors}>{(f) => (
            <StatTile testId="secdash-factors" label={t('admin-secdash-factors')}
              value={f().knownAccounts === 0 ? null : formatPercent(f().withFactor / f().knownAccounts)}
              unavailableReason={t('admin-secdash-factors-none')}
              detail={t('admin-secdash-factors-detail', { passkey: f().passkey, totp: f().totp, total: f().knownAccounts })}
              href={tab('requirements')} onLink={go(tab('requirements'))} linkLabel={t('admin-secdash-open-requirements')} />
          )}</Show>
          <Show when={d().signins}>{(s) => (
            <StatTile testId="secdash-failed" label={t('admin-secdash-failed-24h')} value={s().failed24h}
              detail={t('admin-secdash-failed-detail', { blocked: s().blocked24h, week: s().failed7d + s().blocked7d })}
              href={sectionPath('logs')} onLink={go(sectionPath('logs'))} linkLabel={t('admin-secdash-open-auth-logs')} />
          )}</Show>
          <Show when={d().protection}>{(p) => (
            <StatTile testId="secdash-protection" label={t('admin-secdash-protection')} value={p().protectionEnforced24h}
              detail={t('admin-secdash-protection-detail', { observed: p().protectionObserved24h, bans: p().activeBans, locks: p().activeLocks })}
              href={tab('protection')} onLink={go(tab('protection'))} linkLabel={t('admin-secdash-open-protection')} />
          )}</Show>
          <Show when={d().protection}>{(p) => (
            <StatTile testId="secdash-locks" label={t('admin-secdash-locks')} value={p().activeLocks}
              detail={t('admin-secdash-locks-detail', { admin: p().activeAdminLocks, bans: p().activeBans })}
              href={tab('locks')} onLink={go(tab('locks'))} linkLabel={t('admin-secdash-open-locks')} />
          )}</Show>
          <Show when={d().admins}>{(a) => (
            <StatTile testId="secdash-admins" label={t('admin-secdash-admin-sessions')} value={a().activeSessions}
              detail={t('admin-secdash-admin-sessions-detail', { admins: a().adminsSignedIn })}
              href={sectionPath('admins')} onLink={go(sectionPath('admins'))} linkLabel={t('admin-secdash-open-admins')} />
          )}</Show>
          <Show when={d().posture}>{(p) => (
            <StatTile testId="secdash-posture" label={t('admin-secdash-posture')} value={p().warnings === 0 ? t('admin-secdash-posture-ok') : t('admin-secdash-posture-warnings', { count: p().warnings })}
              detail={t('admin-secdash-posture-detail', { count: p().checks })}
              href={tab('deployment')} onLink={go(tab('deployment'))} linkLabel={t('admin-secdash-open-deployment')} />
          )}</Show>
        </div>

        <div class={css.panels}>
          <Show when={d().factors}>{(f) => (
            <section class={css.panel} data-testid="secdash-factors-panel" aria-labelledby="secdash-factors-h">
              <h3 id="secdash-factors-h" class={css.panelTitle}>{t('admin-secdash-factors')}</h3>
              <ul class={css.facts}>
                <li><span>{t('admin-secdash-factor-passkey')}</span><strong>{formatNumber(f().passkey)}</strong></li>
                <li><span>{t('admin-secdash-factor-totp')}</span><strong>{formatNumber(f().totp)}</strong></li>
                <li><span>{t('admin-secdash-factor-both')}</span><strong>{formatNumber(f().both)}</strong></li>
                <li><span>{t('admin-secdash-factor-none')}</span><strong>{formatNumber(f().knownAccounts - f().withFactor)}</strong></li>
              </ul>
              <p class={f().withoutFactorRequired > 0 ? css.warn : css.note}>
                <Show when={f().withoutFactorRequired > 0}><Icon name="alert" /></Show>
                {f().requiredGlobally || f().requiredDomains > 0
                  ? t('admin-secdash-factor-required', { count: f().withoutFactorRequired, domains: f().requiredDomains, global: f().requiredGlobally ? 'yes' : 'no' })
                  : t('admin-secdash-factor-not-required')}
              </p>
              <ul class={css.facts}>
                <li><a class={css.link} href={sectionPath('admins')} onClick={go(sectionPath('admins'))}>{t('admin-secdash-admins-no-factor')}</a><strong>{formatNumber(f().adminsWithoutFactor)}</strong></li>
                <li><a class={css.link} href={sectionPath('users')} onClick={go(sectionPath('users'))}>{t('admin-secdash-stale-keys')}</a><strong>{formatNumber(f().staleApiKeys)}</strong></li>
                <li><span>{t('admin-secdash-env-admin')}</span><strong>{t(f().environmentAdminLoginEnabled ? 'admin-secdash-on' : 'admin-secdash-off')}</strong></li>
              </ul>
              <a class={css.link} href={tab('requirements')} onClick={go(tab('requirements'))}>{t('admin-secdash-open-requirements')}</a>
            </section>
          )}</Show>

          <Show when={d().signins}>{(s) => (
            <section class={css.panel} data-testid="secdash-signins-panel" aria-labelledby="secdash-signins-h">
              <h3 id="secdash-signins-h" class={css.panelTitle}>{t('admin-secdash-signins')}</h3>
              <Sparkline testId="secdash-spark-24h" title={t('admin-secdash-spark-24h')} tone="danger"
                points={s().hourly.map((b) => ({ t: b.t, value: b.failed + b.blocked }))}
                formatTime={(secs) => formatTime(secs * 1000)} />
              <Sparkline testId="secdash-spark-7d" title={t('admin-secdash-spark-7d')} tone="danger"
                points={s().daily.map((b) => ({ t: b.t, value: b.failed + b.blocked }))}
                formatTime={(secs) => formatDateTime(new Date(secs * 1000).toISOString(), { dateStyle: 'medium' })} />
              <h4 class={css.subTitle}>{t('admin-secdash-top-sources')}</h4>
              <Show when={s().topSources.length > 0} fallback={<p class={css.note}>{t('admin-secdash-no-failures')}</p>}>
                <ol class={css.ranked}>
                  <For each={s().topSources}>{(row) => (
                    <li><span class={css.mono}>{row.source}</span><Show when={row.country}>{(c) => <span class={css.dim}>{countryName(c())}</span>}</Show><strong>{formatNumber(row.count)}</strong></li>
                  )}</For>
                </ol>
              </Show>
              <Show when={s().sourcesMasked}><p class={css.note}>{t('admin-secdash-sources-masked')}</p></Show>
              <h4 class={css.subTitle}>{t('admin-secdash-top-countries')}</h4>
              <Show when={s().geoipReady} fallback={<p class={css.note}>{t('admin-secdash-geoip-missing')}</p>}>
                <Show when={s().topCountries.length > 0} fallback={<p class={css.note}>{t('admin-secdash-no-failures')}</p>}>
                  <ol class={css.ranked}>
                    <For each={s().topCountries}>{(row) => <li><span>{countryName(row.country)}</span><strong>{formatNumber(row.count)}</strong></li>}</For>
                  </ol>
                </Show>
              </Show>
            </section>
          )}</Show>

          <Show when={d().admins}>{(a) => (
            <section class={css.panel} data-testid="secdash-admins-panel" aria-labelledby="secdash-admins-h">
              <h3 id="secdash-admins-h" class={css.panelTitle}>{t('admin-secdash-recent-admins')}</h3>
              <Show when={a().recentSignins.length > 0} fallback={<p class={css.note}>{t('admin-secdash-no-admin-signins')}</p>}>
                <ul class={css.rows}>
                  <For each={a().recentSignins}>{(row) => (
                    <li><strong>{row.username}</strong><span class={css.dim}>{formatDateTime(new Date(row.at * 1000).toISOString())}</span><span class={css.mono}>{row.source}</span></li>
                  )}</For>
                </ul>
              </Show>
              <a class={css.link} href={sectionPath('admins')} onClick={go(sectionPath('admins'))}>{t('admin-secdash-open-admins')}</a>
            </section>
          )}</Show>

          <Show when={d().audit}>{(a) => (
            <section class={css.panel} data-testid="secdash-audit-panel" aria-labelledby="secdash-audit-h">
              <h3 id="secdash-audit-h" class={css.panelTitle}>{t('admin-secdash-audit')}</h3>
              <Show when={a().recent.length > 0} fallback={<p class={css.note}>{t('admin-secdash-no-audit')}</p>}>
                <ul class={css.rows}>
                  <For each={a().recent}>{(row) => (
                    <li><strong>{auditLabel(row.action)}</strong><span class={css.dim}>{formatDateTime(row.at)}</span><span>{row.target?.replace(/^setting:/, '') ?? row.actor}</span></li>
                  )}</For>
                </ul>
              </Show>
              <a class={css.link} href={sectionPath('observability')} onClick={go(sectionPath('observability'))}>{t('admin-secdash-open-audit')}</a>
            </section>
          )}</Show>

          <Show when={d().posture}>{(p) => (
            <section class={css.panel} data-testid="secdash-posture-panel" aria-labelledby="secdash-posture-h">
              <h3 id="secdash-posture-h" class={css.panelTitle}>{t('admin-secdash-posture')}</h3>
              <ul class={css.checks}>
                <Check label={t('admin-secdash-check-https')} status={p().https} />
                <Check label={t('admin-secdash-check-hsts')} status={p().hsts} />
                <Check label={t('admin-secdash-check-cookies')} status={p().secureCookies} />
                <Check label={t('admin-secdash-check-csrf')} status={p().csrf} />
                <Check label={t('admin-secdash-check-admin-factors')} status={p().adminFactors} />
                <Check label={t(p().geoipReady ? 'admin-secdash-check-geoip-ready' : 'admin-secdash-check-geoip-missing')} status={p().geoipReady ? 'pass' : 'info'} />
                <Show when={p().analyticsTag !== null}>
                  <Check label={t(p().analyticsTag ? 'admin-secdash-check-analytics-on' : 'admin-secdash-check-analytics-off')} status="info" />
                </Show>
                <Show when={p().consoleDefaultPath !== undefined}>
                  <Check label={t(p().consoleDefaultPath ? 'admin-secdash-check-console-default' : 'admin-secdash-check-console-custom')} status={p().consoleDefaultPath ? 'warning' : 'pass'}
                    href={tab('console')} onLink={go(tab('console'))} />
                </Show>
              </ul>
              <div class={css.linkRow}>
                <a class={css.link} href={tab('deployment')} onClick={go(tab('deployment'))}>{t('admin-secdash-open-deployment')}</a>
                <a class={css.link} href={sectionPath('diagnostics')} onClick={go(sectionPath('diagnostics'))}>{t('admin-secdash-open-diagnostics')}</a>
              </div>
            </section>
          )}</Show>
        </div>
      </>}</Show>
    </div>
  );
}

function Check(props: { label: string; status: CheckStatus; href?: string; onLink?: (e: MouseEvent) => void }): JSX.Element {
  const word = (): string => t(`admin-secdash-status-${props.status ?? 'info'}`);
  return (
    <li class={css.check} data-status={props.status ?? 'info'}>
      <span class={props.status === 'warning' ? css.checkWarn : props.status === 'pass' ? css.checkPass : css.checkInfo}>
        <Icon name={props.status === 'warning' ? 'alert' : props.status === 'pass' ? 'check' : 'info'} />
      </span>
      <Show when={props.href} fallback={<span>{props.label}</span>}>{(href) => <a class={css.link} href={href()} onClick={(e) => props.onLink?.(e)}>{props.label}</a>}</Show>
      <span class={a11y.srOnly}>{word()}</span>
    </li>
  );
}
