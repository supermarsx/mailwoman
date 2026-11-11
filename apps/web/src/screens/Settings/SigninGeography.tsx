import {formatDate as regionalDate} from '../../i18n/format.ts';
import { createResource, createSignal, Show, type JSX } from 'solid-js';
import { apiFetch } from '../../api/http.ts';
import { basePath } from '../../api/basePath.ts';
import { fetchSettings, putSetting, SettingsApiError } from '../../api/settings.ts';
import { t } from '../../i18n';
import { ControlRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';


const MODE = 'security.signin.accountGeoMode';
const COUNTRIES = 'security.signin.accountGeoAllowCountries';
const IPS = 'security.signin.accountGeoAllowIps';

type Context = { ip: string | null; country: string | null; database: string };
type HistoryEntry = { cc: string; first: number; last: number; n: number };
type History = { retentionDays: number; maxCountries: number; countries: HistoryEntry[] };
const HISTORY = '/api/security/signin-geography/history';
const MODES = ['off', 'block-unusual-countries', 'block-foreign-countries'] as const;

const list = (value: unknown): string[] => (Array.isArray(value) ? value.filter((v): v is string => typeof v === 'string') : []);
const split = (text: string): string[] =>
  [...new Set(text.split(/[\s,;]+/).map((s) => s.trim()).filter(Boolean))];

/**
 * Per-account sign-in country policy (off by default). Uses the ordinary user
 * settings API, so an administrator lock shows as read-only. Saving writes the
 * allowlists first and the mode last; the server refuses a policy that would
 * block this very request (409 signinGeographySelfLockout).
 */
export function SigninGeography(): JSX.Element {
  const [data, { refetch }] = createResource(async () => {
    const [settings, context] = await Promise.all([
      fetchSettings(),
      apiFetch(`${basePath()}/api/security/signin-geography/context`, { credentials: 'same-origin' }).then(
        async (r) => (r.ok ? ((await r.json()) as Context) : null),
        () => null,
      ),
    ]);
    return { settings, context };
  });
  const [history, { refetch: refetchHistory }] = createResource(async () => {
    try {
      const r = await apiFetch(`${basePath()}${HISTORY}`, { credentials: 'same-origin' });
      return r.ok ? ((await r.json()) as History) : null;
    } catch { return null; }
  });
  const [forgetting, setForgetting] = createSignal(false);
  async function forget(): Promise<void> {
    setForgetting(true);
    try {
      await apiFetch(`${basePath()}${HISTORY}`, { method: 'DELETE', credentials: 'same-origin' });
      await refetchHistory();
    } finally { setForgetting(false); }
  }
  const day = (seconds: number): string => regionalDate(seconds*1000);
  const [mode, setMode] = createSignal<string | null>(null);
  const [countries, setCountries] = createSignal<string | null>(null);
  const [ips, setIps] = createSignal<string | null>(null);
  const [message, setMessage] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const entry = (key: string) => data()?.settings[key];
  const currentMode = () => mode() ?? String(entry(MODE)?.value ?? 'off');
  const currentCountries = () => countries() ?? list(entry(COUNTRIES)?.value).join(', ');
  const currentIps = () => ips() ?? list(entry(IPS)?.value).join('\n');
  const locked = () => entry(MODE)?.locked === true;
  const addMine = (what: 'ip' | 'country') => {
    const context = data()?.context;
    if (what === 'ip' && context?.ip) setIps(split(`${currentIps()}\n${context.ip}`).join('\n'));
    if (what === 'country' && context?.country) setCountries(split(`${currentCountries()},${context.country}`).join(', '));
  };
  async function save(): Promise<void> {
    setBusy(true);
    setMessage('');
    try {
      const wanted = currentMode();
      // Lists first, mode last: the guard checks the policy the write would produce.
      if (!entry(COUNTRIES)?.locked) await putSetting(COUNTRIES, split(currentCountries()).map((c) => c.toUpperCase()));
      if (!entry(IPS)?.locked) await putSetting(IPS, split(currentIps()));
      if (!locked()) await putSetting(MODE, wanted);
      await refetch();
      setMode(null);
      setCountries(null);
      setIps(null);
      setMessage(t('signin-geo-saved'));
    } catch (error) {
      setMessage(
        error instanceof SettingsApiError && error.code === 'signinGeographySelfLockout'
          ? t('signin-geo-self-lockout')
          : error instanceof SettingsApiError && error.status === 422
            ? t('signin-geo-invalid')
            : t('signin-geo-error'),
      );
    } finally {
      setBusy(false);
    }
  }
  return (
    <SettingsCard id="signin-geo-card" title={t('signin-geo-title')} description={t('signin-geo-help')}>
      <p class={css.prose} role="note" data-testid="signin-geo-notice">
        {t('signin-geo-accuracy')}
      </p>
      <Show when={data()}>
        <Show when={locked()}>
          <p class={css.prose}>{t('signin-geo-locked')}</p>
        </Show>
        <ControlRow controlId="signin-geo-mode" label={t('signin-geo-mode')}>{() =>
          <select id="signin-geo-mode" class={css.input} disabled={locked()} value={currentMode()} onChange={(e) => setMode(e.currentTarget.value)}>
            {MODES.map((m) => (
              <option value={m}>{t(`signin-geo-mode-${m}`)}</option>
            ))}
          </select>
        }</ControlRow>
        <ControlRow controlId="signin-geo-countries" label={t('signin-geo-countries')}>{() =>
          <input id="signin-geo-countries" class={css.input} value={currentCountries()} placeholder="PT, ES" onInput={(e) => setCountries(e.currentTarget.value)} />
        }</ControlRow>
        <ControlRow controlId="signin-geo-ips" stacked label={t('signin-geo-ips')}>{() =>
          <textarea id="signin-geo-ips" class={css.input} rows={3} value={currentIps()} placeholder="203.0.113.0/24" onInput={(e) => setIps(e.currentTarget.value)} />
        }</ControlRow>
        <p class={css.meta}>
          {data()?.context?.ip ? t('signin-geo-current', { ip: data()!.context!.ip!, country: data()!.context!.country ?? t('signin-geo-unknown-country') }) : t('signin-geo-no-context')}
        </p>
        <div class={css.actions}>
          <button type="button" disabled={!data()?.context?.ip} onClick={() => addMine('ip')}>
            {t('signin-geo-add-ip')}
          </button>
          <button type="button" disabled={!data()?.context?.country} onClick={() => addMine('country')}>
            {t('signin-geo-add-country')}
          </button>
          <button type="button" disabled={busy()} onClick={() => void save()}>
            {t('signin-geo-save')}
          </button>
        </div>
        <Show when={message()}>
          <p role="status">{message()}</p>
        </Show>
        <Show when={history()}>
          {(h) => (
            <div data-testid="signin-geo-history">
              <h4 class={css.subheading}>{t('signin-geo-history-title')}</h4>
              <p class={css.prose}>{t('signin-geo-history-help', { days: h().retentionDays })}</p>
              <Show when={h().countries.length > 0} fallback={<p class={css.meta}>{t('signin-geo-history-empty')}</p>}>
                <ul>
                  {h().countries.map((c) => (
                    <li>{t('signin-geo-history-entry', { country: c.cc, first: day(c.first), last: day(c.last), count: c.n })}</li>
                  ))}
                </ul>
              </Show>
              <div class={css.actions}>
                <button type="button" disabled={forgetting() || h().countries.length === 0} onClick={() => void forget()}>
                  {t('signin-geo-history-forget')}
                </button>
              </div>
            </div>
          )}
        </Show>
      </Show>
    </SettingsCard>
  );
}
export default SigninGeography;
