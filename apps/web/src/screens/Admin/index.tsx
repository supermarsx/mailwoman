// Admin console root (SPEC §19; t32-admin.md §2.1).
//
// The default export is the lazily-loadable admin route root — App.tsx reaches it
// ONLY through `() => import('./screens/Admin/index.tsx')`, so the whole
// `screens/Admin/**` tree is code-split into its own chunk, ABSENT from the
// login→inbox mailbox bundle (the bundle regression gate, plan §1.7). Each
// section screen is a further lazy chunk (`shell/sections.ts`).
//
// This file is the session gate plus the shell: it probes `/admin/session` and
// renders the [`AdminLogin`] gate without one; with a session it loads the
// admin's capabilities (`GET /admin/me`, `shell/can.ts`), parses the URL into a
// section (`shell/router.ts`) and renders that section inside [`AdminShell`].
//
// Component tests inject mock clients through the props; production uses the
// same-origin HTTP clients.

import { Show, Suspense, createEffect, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { createAdminSlice, createHttpAdminApi, type AdminApi } from '../../state/slices/admin.ts';
import { createThemeSlice } from '../../state/slices/theme.ts';
import type { SliceContext } from '../../state/slices/context.ts';
import { t, loadCatalog } from '../../i18n';
import { AdminContext, useAdmin } from './context.ts';
import { AdminLogin } from './AdminLogin.tsx';
import { AdminShell } from './shell/AdminShell.tsx';
import { SECTIONS, findSection, type SectionDef } from './shell/sections.ts';
import { createAdminRouter, hrefFor, sectionPath, type AdminRouter } from './shell/router.ts';
import { createLeaveCheck } from './shell/useDirtyGuard.ts';
import { createAdminMeLoader, createLoadedCan, type AdminMeLoader } from './shell/can.ts';
import { ShellContext } from './shell/context.ts';
import { createAdminSearch, type AdminSearch } from './shell/CommandPalette.tsx';
import { withTotpGate, type AdminTotpVerifier } from './shell/adminAuth.ts';
import type { Crumb } from './shell/Breadcrumbs.tsx';
import { LanguagePicker, useStoredAdminLocale } from './shell/LanguagePicker.tsx';
import { createDashboardLoader, type DashboardLoader } from '../../api/admin/dashboard.ts';
import * as css from './admin.css.ts';
import * as shellCss from './shell/shell.css.ts';

export interface AdminScreenProps {
  /** The admin client. Defaults to the same-origin HTTP client; tests inject a mock. */
  api?: AdminApi;
  /** `GET /admin/me`. Defaults to the HTTP loader. */
  loadMe?: AdminMeLoader;
  /** `GET /admin/dashboard`. Defaults to the HTTP loader. */
  loadDashboard?: DashboardLoader;
  /** `GET /admin/search`. Defaults to the HTTP client. */
  search?: AdminSearch;
  /** Second sign-in step. Defaults to the HTTP client. */
  verifyTotp?: AdminTotpVerifier;
  /** The section table. Defaults to `SECTIONS`; tests may pass a smaller one. */
  sections?: readonly SectionDef[];
}

export function AdminScreen(props: AdminScreenProps): JSX.Element {
  const admin = createAdminSlice(withTotpGate(props.api ?? createHttpAdminApi()));
  onMount(() => void admin.loadSession());
  onMount(() => void loadCatalog('admin'));
  onMount(() => void loadCatalog('admin-shell'));
  onMount(() => void loadCatalog('admin-dashboard'));
  onMount(() => void loadCatalog('plugins'));
  useStoredAdminLocale();

  // The console takes the theme this browser chose for the webmail (read-only:
  // nothing here changes the stored preference). The slice ignores its context.
  onMount(() => {
    const theme = createThemeSlice({} as SliceContext);
    onCleanup(() => theme.dispose());
  });

  return (
    <AdminContext.Provider value={admin}>
      <Show when={admin.sessionChecked()} fallback={<div class={css.gate}>{t('common-loading')}</div>}>
        <Show when={admin.session()} fallback={<AdminLogin verifyTotp={props.verifyTotp} />}>
          {(session) => <Console {...props} username={session().username} />}
        </Show>
      </Show>
    </AdminContext.Provider>
  );
}

function Console(props: AdminScreenProps & { username: string }): JSX.Element {
  const admin = useAdmin();
  const sections = props.sections ?? SECTIONS;
  const can = createLoadedCan(props.loadMe ?? createAdminMeLoader());
  onMount(() => void can.reload());

  // The "leave with unsaved changes?" question: the router awaits the answer.
  const [leaveOpen, setLeaveOpen] = createSignal(false);
  let answer: ((ok: boolean) => void) | null = null;
  const ask = (): Promise<boolean> =>
    new Promise<boolean>((resolve) => {
      answer = resolve;
      setLeaveOpen(true);
    });
  function settle(ok: boolean): void {
    setLeaveOpen(false);
    answer?.(ok);
    answer = null;
  }

  const base = createAdminRouter(createLeaveCheck(ask));
  onCleanup(() => base.dispose());
  // After an in-app navigation focus moves to the main column, so keyboard and
  // screen-reader users continue in the new section rather than in the sidebar.
  const router: AdminRouter = {
    ...base,
    async navigate(path, opts) {
      const ok = await base.navigate(path, opts);
      if (ok) queueMicrotask(() => document.getElementById('admin-main')?.focus({ preventScroll: true }));
      return ok;
    },
  };
  const navigate = (path: string): Promise<boolean> => router.navigate(path);

  const services = {
    loadDashboard: props.loadDashboard ?? createDashboardLoader(),
    search: props.search ?? createAdminSearch(),
  };

  const section = (): SectionDef | undefined => {
    const s = findSection(router.route().section, sections);
    return s && can.can(s.capability) ? s : undefined;
  };

  createEffect(() => {
    const s = section();
    document.title = s ? `${s.label()} · ${t('admin-brand')}` : t('admin-brand');
  });

  const crumbs = (): Crumb[] => {
    const s = section();
    const r = router.route();
    if (!s || s.id === 'dashboard') return [];
    const items: Crumb[] = [
      { label: t('admin-shell-nav-dashboard'), href: hrefFor('/admin'), onNavigate: () => void navigate('/admin') },
      {
        label: s.label(),
        href: hrefFor(sectionPath(s.id)),
        onNavigate: () => void navigate(sectionPath(s.id)),
      },
    ];
    r.rest.forEach((part, i) => {
      const path = sectionPath(s.id, r.rest.slice(0, i + 1));
      items.push({ label: part, href: hrefFor(path), onNavigate: () => void navigate(path) });
    });
    return items;
  };

  return (
    <ShellContext.Provider
      value={{
        can,
        router,
        services,
        username: () => props.username,
      }}
    >
      <Show when={can.loaded()} fallback={<div class={css.gate}>{t('common-loading')}</div>}>
        <AdminShell
          sections={sections}
          route={router.route()}
          navigate={(p) => void navigate(p)}
          can={can}
          username={props.username}
          onSignOut={() => void admin.logout()}
          search={services.search}
          breadcrumbs={crumbs()}
          leave={{ open: leaveOpen(), onStay: () => settle(false), onLeave: () => settle(true) }}
          languagePicker={<LanguagePicker />}
        >
          <h1 class={shellCss.srOnly}>{t('admin-brand')}</h1>
          <Suspense
            fallback={
              <p class={shellCss.state} role="status">
                {t('admin-shell-loading')}
              </p>
            }
          >
            <Show
              when={section()}
              fallback={
                <div class={shellCss.state} data-testid="admin-unknown-section">
                  <p>{t('admin-shell-unknown-section')}</p>
                  <a
                    class="btn btn--ghost"
                    href={hrefFor('/admin')}
                    onClick={(e) => {
                      e.preventDefault();
                      void navigate('/admin');
                    }}
                  >
                    {t('admin-shell-back-to-dashboard')}
                  </a>
                </div>
              }
            >
              {(s) => <Dynamic component={s().component} api={admin.api} route={router.route()} />}
            </Show>
          </Suspense>
        </AdminShell>
      </Show>
    </ShellContext.Provider>
  );
}

export default AdminScreen;
