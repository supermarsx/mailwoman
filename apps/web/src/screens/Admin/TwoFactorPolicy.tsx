// Admin › Require two-factor (26.16, plan §3 e16 — DQ2).
//
// A require-2FA policy panel beside `SecurityPolicy.tsx`. DQ2: any user may enrol a
// factor (opt-in); an admin may additionally REQUIRE a second factor org-wide
// (global) or for one mail domain. An enrolled user is always required regardless
// — this panel governs the org/domain *requirement*, which forces enrolment on
// next login for accounts that have not yet enrolled.
//
// It follows the SSO/metadata/rethread LOCAL-SIGNAL mount pattern: the frozen
// `AdminSection` union is untouched — `index.tsx` layers this in with a
// `twofaActive` signal, exactly like `rethreadActive`. It reads/writes e3's
// separate `GET|POST /admin/2fa/policy` surface via `TwofaPolicyApi` (not the
// frozen `AdminApi`); the optional `AdminApi` is used only to suggest managed
// domains in the add form.
//
// WCAG 2.2 AA: every control is labelled; status/error use role="status"/"alert".

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { createHttpAdminApi, type AdminApi } from '../../state/slices/admin.ts';
import {
  createHttpTwofaPolicyApi,
  type TwofaPolicyApi,
  type TwofaPolicyRow,
} from './twofaPolicy.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './admin.css.ts';

export interface TwoFactorPolicyProps {
  /** The 2FA-policy client. Defaults to the same-origin admin client; tests mock it. */
  policy?: TwofaPolicyApi;
  /** Lists managed domains for the add-form suggestions. Defaults to the admin client. */
  api?: AdminApi;
}

export function TwoFactorPolicy(props: TwoFactorPolicyProps): JSX.Element {
  const policyApi = props.policy ?? createHttpTwofaPolicyApi();
  const adminApi = props.api ?? createHttpAdminApi();

  onMount(() => void loadCatalog('admin'));

  const [rows, { refetch }] = createResource(() => policyApi.list());
  // Managed domains for the add-form datalist (best-effort; empty on error).
  const [domains] = createResource(async () => {
    try {
      return (await adminApi.listDomains()).map((d) => d.name);
    } catch {
      return [] as string[];
    }
  });

  const [newDomain, setNewDomain] = createSignal('');
  const [newRequire, setNewRequire] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);

  /** The global row (or a synthetic default when none has been set yet). */
  const globalRow = (): TwofaPolicyRow => {
    const found = (rows() ?? []).find((r) => r.scopeKind === 'global');
    return found ?? { scopeKind: 'global', scopeValue: '', require2fa: false };
  };
  /** The per-domain rows, sorted by domain for a stable list. */
  const domainRows = (): TwofaPolicyRow[] =>
    (rows() ?? [])
      .filter((r) => r.scopeKind === 'domain')
      .sort((a, b) => a.scopeValue.localeCompare(b.scopeValue));

  async function upsert(scopeKind: 'global' | 'domain', scopeValue: string, require2fa: boolean): Promise<void> {
    setError(null);
    setSaved(false);
    try {
      await policyApi.set({ scopeKind, scopeValue, require2fa });
      setSaved(true);
      await refetch();
    } catch {
      setError(t('admin-2fa-save-error'));
    }
  }

  async function onAddDomain(e: Event): Promise<void> {
    e.preventDefault();
    const domain = newDomain().trim().toLowerCase();
    if (domain === '') return;
    await upsert('domain', domain, newRequire());
    setNewDomain('');
    setNewRequire(true);
  }

  return (
    <section class={css.section} aria-label={t('admin-2fa-title')} data-testid="admin-2fa">
      <h2 class={css.heading}>{t('admin-2fa-title')}</h2>
      <p class={css.note}>{t('admin-2fa-intro')}</p>

      <Show when={rows.error as unknown}>
        <p class={css.error} role="alert">{t('admin-2fa-load-error')}</p>
      </Show>
      <Show when={error()}>
        <p class={css.error} role="alert">{error()}</p>
      </Show>

      {/* ── Global requirement ── */}
      <div class={css.card}>
        <label class="field">
          <input
            type="checkbox"
            checked={globalRow().require2fa}
            data-testid="admin-2fa-global"
            aria-label={t('admin-2fa-global-label')}
            onChange={(e) => void upsert('global', '', e.currentTarget.checked)}
          />{' '}
          {t('admin-2fa-global')}
        </label>
        <p class={css.note}>{t('admin-2fa-global-note')}</p>
      </div>

      {/* ── Per-domain requirements ── */}
      <div class={css.card}>
        <h3 class={css.heading} style={{ 'font-size': '1rem' }}>{t('admin-2fa-domains-heading')}</h3>
        <Show
          when={domainRows().length > 0}
          fallback={<p class={css.note}>{t('admin-2fa-domains-empty')}</p>}
        >
          <div class={css.tableWrap}>
            <table class={css.table}>
              <thead>
                <tr>
                  <th>{t('admin-2fa-col-domain')}</th>
                  <th>{t('admin-2fa-col-require')}</th>
                </tr>
              </thead>
              <tbody>
                <For each={domainRows()}>
                  {(row) => (
                    <tr data-testid="admin-2fa-domain-row">
                      <td><bdi>{row.scopeValue}</bdi></td>
                      <td>
                        <input
                          type="checkbox"
                          checked={row.require2fa}
                          aria-label={t('admin-2fa-require-for', { domain: row.scopeValue })}
                          onChange={(e) => void upsert('domain', row.scopeValue, e.currentTarget.checked)}
                        />
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>

        <form class="field" onSubmit={(e) => void onAddDomain(e)}>
          <label class="field">
            <span>{t('admin-2fa-add-domain')}</span>
            <input
              type="text"
              list="admin-2fa-domain-options"
              placeholder={t('admin-2fa-add-domain-placeholder')}
              value={newDomain()}
              data-testid="admin-2fa-add-domain"
              onInput={(e) => setNewDomain(e.currentTarget.value)}
            />
            <datalist id="admin-2fa-domain-options">
              <For each={domains() ?? []}>{(d) => <option value={d} />}</For>
            </datalist>
          </label>
          <label class="field">
            <input
              type="checkbox"
              checked={newRequire()}
              aria-label={t('admin-2fa-add-require-label')}
              onChange={(e) => setNewRequire(e.currentTarget.checked)}
            />{' '}
            {t('admin-2fa-add-require')}
          </label>
          <button type="submit" class="btn btn--primary" data-testid="admin-2fa-add-submit">
            {t('admin-2fa-add-save')}
          </button>
        </form>
      </div>

      <Show when={saved()}>
        <p class={css.note} role="status">{t('admin-saved')}</p>
      </Show>
    </section>
  );
}

export default TwoFactorPolicy;
