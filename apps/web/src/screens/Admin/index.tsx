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

import { For, Show, Suspense, onMount, type JSX } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import {
  ADMIN_SECTIONS,
  ADMIN_SECTION_LABELS,
  createAdminSlice,
  createHttpAdminApi,
  type AdminApi,
  type AdminSection,
} from '../../state/slices/admin.ts';
import { AdminContext } from './context.ts';
import { AdminLogin } from './AdminLogin.tsx';
import { Domains } from './Domains.tsx';
import { Users } from './Users.tsx';
import { SecurityPolicy } from './SecurityPolicy.tsx';
import { Integrations } from './Integrations.tsx';
import { Observability } from './Observability.tsx';
import { Appearance } from './Appearance.tsx';
import * as css from './admin.css.ts';

/** The section → component map (§19). */
const SECTION_VIEWS: Record<AdminSection, () => JSX.Element> = {
  domains: Domains,
  users: Users,
  security: SecurityPolicy,
  integrations: Integrations,
  observability: Observability,
  appearance: Appearance,
};

export interface AdminScreenProps {
  /** The admin client. Defaults to the same-origin HTTP client; tests inject a mock. */
  api?: AdminApi;
}

export function AdminScreen(props: AdminScreenProps): JSX.Element {
  const admin = createAdminSlice(props.api ?? createHttpAdminApi());
  onMount(() => void admin.loadSession());

  return (
    <AdminContext.Provider value={admin}>
      <Show when={admin.sessionChecked()} fallback={<div class={css.gate}>Loading…</div>}>
        <Show when={admin.session() !== null} fallback={<AdminLogin />}>
          <div class={css.shell} data-screen="admin">
            <nav class={css.sidebar} aria-label="Admin sections">
              <span class={css.brand}>Mailwoman admin</span>
              <For each={ADMIN_SECTIONS}>
                {(s) => (
                  <button
                    type="button"
                    class={css.navItem}
                    aria-current={admin.section() === s}
                    onClick={() => admin.setSection(s)}
                  >
                    {ADMIN_SECTION_LABELS[s]}
                  </button>
                )}
              </For>
              <button type="button" class="btn btn--ghost" onClick={() => void admin.logout()}>
                Sign out
              </button>
            </nav>
            <main class={css.main}>
              <Suspense fallback={<div class={css.note}>Loading…</div>}>
                <Dynamic component={SECTION_VIEWS[admin.section()]} />
              </Suspense>
            </main>
          </div>
        </Show>
      </Show>
    </AdminContext.Provider>
  );
}

export default AdminScreen;
