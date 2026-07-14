// V6 Admin panel screen (SPEC §19, plan §2.6, §3 e7).
//
// The default export is the lazily-loadable admin route root — App.tsx reaches it
// ONLY through `lazy(() => import('./screens/Admin/index.tsx'))`, so the whole
// `screens/Admin/**` tree is code-split into its own chunk, ABSENT from the
// login→inbox mailbox bundle (the bundle regression gate, plan §1.7).
//
// It is gated on a SEPARATE admin session (plan §2.5): the root probes
// `/admin/session`; with no session it renders the [`AdminLogin`] gate, otherwise
// the §19 panel (Domains / Users / Security-policy / Integrations / Observability /
// Appearance). Component tests inject a mock `AdminApi` via the `api` prop; the
// production default is the same-origin HTTP client.

import { createSignal, For, Show, Suspense, onMount, type JSX } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import {
  ADMIN_SECTIONS,
  createAdminSlice,
  createHttpAdminApi,
  type AdminApi,
  type AdminSection,
} from '../../state/slices/admin.ts';
import { t, loadCatalog } from '../../i18n';
import { createRovingTabindex } from '../../components/a11y';
import { AdminContext } from './context.ts';
import { AdminLogin } from './AdminLogin.tsx';
import { Domains } from './Domains.tsx';
import { Users } from './Users.tsx';
import { SecurityPolicy } from './SecurityPolicy.tsx';
import { Integrations } from './Integrations.tsx';
import { Observability } from './Observability.tsx';
import { Appearance } from './Appearance.tsx';
import { AdminPlugins } from './Plugins/index.tsx';
import { AdminAssist } from './Assist/index.tsx';
import { AdminSso } from './Sso/index.tsx';
import * as css from './admin.css.ts';

/** Localised nav labels per section (the source labels live in admin.ftl). */
const NAV_LABEL: Record<AdminSection, () => string> = {
  domains: () => t('admin-nav-domains'),
  users: () => t('admin-nav-users'),
  security: () => t('admin-nav-security'),
  integrations: () => t('admin-nav-integrations'),
  observability: () => t('admin-nav-observability'),
  appearance: () => t('admin-nav-appearance'),
  plugins: () => t('admin-nav-plugins'),
  assist: () => t('admin-nav-assist'),
};

/** The section → component map (§19 + V7 plugins/assist, plan §3 e14). */
const SECTION_VIEWS: Record<AdminSection, () => JSX.Element> = {
  domains: Domains,
  users: Users,
  security: SecurityPolicy,
  integrations: Integrations,
  observability: Observability,
  appearance: Appearance,
  // Wrapped so their optional props default to the production HTTP clients.
  plugins: () => <AdminPlugins />,
  assist: () => <AdminAssist />,
};

export interface AdminScreenProps {
  /** The admin client. Defaults to the same-origin HTTP client; tests inject a mock. */
  api?: AdminApi;
}

export function AdminScreen(props: AdminScreenProps): JSX.Element {
  const admin = createAdminSlice(props.api ?? createHttpAdminApi());
  // Reactive ref: the nav is behind an async session gate, so a plain `let` would
  // still be undefined when the roving effect first runs. A signal re-runs it when
  // the nav actually mounts.
  const [navEl, setNavEl] = createSignal<HTMLElement>();
  // t9 SSO (§18.3): a local section layered on top of the frozen `AdminSection`
  // set — the shared `state/slices/admin.ts` union stays untouched (ownership
  // boundary), so SSO is tracked with its own flag rather than in `admin.section`.
  const [ssoActive, setSsoActive] = createSignal(false);
  onMount(() => void admin.loadSession());
  onMount(() => void loadCatalog('admin'));
  // Roving-tabindex nav: one Tab lands on the current section, arrows move
  // between sections (WAI-ARIA vertical nav pattern).
  createRovingTabindex(navEl, { orientation: 'vertical' });

  return (
    <AdminContext.Provider value={admin}>
      <Show when={admin.sessionChecked()} fallback={<div class={css.gate}>{t('common-loading')}</div>}>
        <Show when={admin.session() !== null} fallback={<AdminLogin />}>
          <div class={css.shell} data-screen="admin">
            <nav ref={setNavEl} class={css.sidebar} aria-label={t('admin-nav')}>
              <span class={css.brand}>{t('admin-brand')}</span>
              <For each={ADMIN_SECTIONS}>
                {(s) => (
                  <button
                    type="button"
                    class={css.navItem}
                    data-roving-item
                    aria-current={!ssoActive() && admin.section() === s}
                    onClick={() => {
                      setSsoActive(false);
                      admin.setSection(s);
                    }}
                  >
                    {NAV_LABEL[s]()}
                  </button>
                )}
              </For>
              <button
                type="button"
                class={css.navItem}
                data-roving-item
                aria-current={ssoActive()}
                onClick={() => setSsoActive(true)}
              >
                {t('admin-nav-sso')}
              </button>
              <button type="button" class="btn btn--ghost" onClick={() => void admin.logout()}>
                {t('admin-sign-out')}
              </button>
            </nav>
            <main class={css.main}>
              <Suspense fallback={<div class={css.note}>{t('common-loading')}</div>}>
                <Show when={ssoActive()} fallback={<Dynamic component={SECTION_VIEWS[admin.section()]} />}>
                  <AdminSso />
                </Show>
              </Suspense>
            </main>
          </div>
        </Show>
      </Show>
    </AdminContext.Provider>
  );
}

export default AdminScreen;
