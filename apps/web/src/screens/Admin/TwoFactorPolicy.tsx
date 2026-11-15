// Admin › Require two-factor (26.16, plan §3 e16 — DQ2).
//
// The require-2FA policy panel, a section of its own in the admin nav
// (`index.tsx`). DQ2: any user may enrol a
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
// WCAG 2.2 AA: every control is labelled. "Saved" and a failed save are toasts
// (Feedback, t32 f39): role="status" / role="alert".

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { createHttpAdminApi, type AdminApi } from '../../state/slices/admin.ts';
import {
  createHttpTwofaPolicyApi,
  type TwofaPolicyApi,
  type TwofaPolicyRow,
} from './twofaPolicy.ts';
import { t, loadCatalog } from '../../i18n';
import { Feedback } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';
import * as settingsCss from '../Settings/styles.css.ts';

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

  const [rowsResource, { refetch }] = createResource(() => policyApi.list());
  // `latest`, not `rowsResource()` (t75): the call form, read while the policy
  // reloads after a save, suspends the console's page boundary and the page is
  // swapped for the page skeleton and back. `latest` suspends only for the first
  // load (the page skeleton, so no made-up "off" is shown); after that the last
  // rows stay while they reload.
  const rows = () => rowsResource.latest;
  // Managed domains for the add-form datalist (best-effort; empty on error).
  const [domains] = createResource(async () => {
    try {
      return [...new Set((await adminApi.listDomains()).flatMap(d=>[d.name,...(d.aliases??[])]).map(name=>name.trim().toLowerCase()).filter(Boolean))].sort();
    } catch {
      return [] as string[];
    }
  });

  const [saving,setSaving]=createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);

  /** The global row (or a synthetic default when none has been set yet). */
  const globalRow = (): TwofaPolicyRow => {
    const found = (rows() ?? []).find((r) => r.scopeKind === 'global');
    return found ?? { scopeKind: 'global', scopeValue: '', require2fa: false };
  };
  /** The per-domain rows, sorted by domain for a stable list. */
  const domainRows = (): TwofaPolicyRow[] => [...new Set([...(domains.latest??[]),...(rows()??[]).filter(row=>row.scopeKind==='domain').map(row=>row.scopeValue)])].sort().map(domain=>(rows()??[]).find(row=>row.scopeKind==='domain'&&row.scopeValue===domain)??{scopeKind:'domain',scopeValue:domain,require2fa:false});

  async function upsert(scopeKind: 'global' | 'domain', scopeValue: string, require2fa: boolean): Promise<void> {
    if(saving())return;setSaving(true);
    setError(null);
    setSaved(false);
    try {
      await policyApi.set({ scopeKind, scopeValue, require2fa });
      setSaved(true);
      await refetch();
    } catch {
      setError(t('admin-2fa-save-error'));
    } finally {setSaving(false);}
  }

  return (
    <section class={css.section} aria-label={t('admin-2fa-title')} data-testid="admin-2fa">
      <h2 class={css.heading}>{t('admin-2fa-title')}</h2>
      <p class={css.note}>{t('admin-2fa-intro')}</p>

      <Show when={rowsResource.error as unknown}>
        <p class={css.error} role="alert">{t('admin-2fa-load-error')}</p>
      </Show>
      <Feedback kind="error" as="toast" message={error()} />

      {/* ── Global requirement ── */}
      <div class={css.card}>
        <label class={settingsCss.check}>
          <input
            class={settingsCss.checkbox}
            type="checkbox"
            disabled={saving()||rowsResource.loading}
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
        <h3 class={`${css.heading} ${css.subHeading}`}>{t('admin-2fa-domains-heading')}</h3>
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
                          class={settingsCss.checkbox}
                          disabled={saving()||globalRow().require2fa}
                          checked={row.require2fa||globalRow().require2fa}
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


      </div>

      <Feedback kind="success" message={saved() ? t('admin-saved') : null} />
    </section>
  );
}

export default TwoFactorPolicy;
