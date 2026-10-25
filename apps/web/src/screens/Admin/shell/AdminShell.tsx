// The admin console frame (t32-admin.md §2.1).
//
// Desktop: a sidebar with the sections in their groups (icon + text label), the
// language slot and Sign out at the foot, and the main column with breadcrumbs
// and the search button. Phone width (< 761 px): a top bar with a menu button
// that opens the same navigation as a drawer (focus kept inside, Escape closes,
// focus returns to the menu button).
//
// Sections the admin's roles do not include are not listed (`can.ts`). Ctrl+K
// (Cmd+K on macOS) opens the command palette anywhere in the console.
//
// Personal appearance lives in the dedicated preferences section. The palette lists the
// UI fields of the sections the admin may open (`fieldAnchors.ts`); and a location
// naming a field (`#/<section>?field=<id>`) focuses and highlights that field once
// the section has rendered it.

import { For, Show, createEffect, createMemo, createSignal, on, onCleanup, onMount, type JSX } from 'solid-js';
import { isolate, loadCatalog, t } from '../../../i18n';
import { createFocusTrap, createRovingTabindex } from '../../../components/a11y';
import type { Can } from './can.ts';
import { GROUPS, type SectionDef } from './sections.ts';
import { fieldPath, hrefFor, sectionPath, type AdminRoute } from './router.ts';
import { Icon } from './icons.tsx';
import { Breadcrumbs, type Crumb } from './Breadcrumbs.tsx';
import { CommandPalette, type AdminSearch, type PaletteField } from './CommandPalette.tsx';
import { fieldAnchors, focusFieldWhenReady } from './fieldAnchors.ts';
import { ConfirmDialog } from './ConfirmDialog.tsx';
import { ResizeHandle } from '../../../components/ResizeHandle.tsx';
import { useOptionalPersonalPreferences } from './personalPreferences.tsx';
import * as css from './shell.css.ts';

export interface AdminShellProps {
  sections: readonly SectionDef[];
  route: AdminRoute;
  navigate: (path: string) => void;
  can: Can;
  username: string;
  onSignOut: () => void;
  search: AdminSearch;
  breadcrumbs?: readonly Crumb[];
  /** Footer slot for the language picker (t32-e9 / H). */
  languagePicker?: JSX.Element;
  /** The "leave with unsaved changes?" question, owned by the router. */
  leave: { open: boolean; onStay: () => void; onLeave: () => void };
  children: JSX.Element;
}

/** Follow a same-tab plain click in place; let modified clicks open a new tab. */
function inPlace(e: MouseEvent, go: () => void): void {
  if (e.defaultPrevented || e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;
  e.preventDefault();
  go();
}

export function AdminShell(props: AdminShellProps): JSX.Element {
  const preferences = useOptionalPersonalPreferences();
  const [sidebarWidth, setSidebarWidth] = createSignal(220);
  const [sidebarCollapsed, setSidebarCollapsed] = createSignal(false);
  createEffect(() => {
    const raw = preferences?.data()['shellLayout'];
    const layout = raw && typeof raw === 'object' ? raw as Record<string, unknown> : {};
    const width = layout['sidebarWidth'];
    setSidebarWidth(typeof width === 'number' && Number.isFinite(width) ? Math.max(180, Math.min(480, width)) : 220);
    setSidebarCollapsed(layout['sidebarCollapsed'] === true);
  });
  const saveLayout = () => preferences?.patch({ shellLayout: { sidebarWidth: sidebarWidth(), sidebarCollapsed: sidebarCollapsed() } });
  const [drawerOpen, setDrawerOpen] = createSignal(false);
  const [paletteOpen, setPaletteOpen] = createSignal(false);
  const [navEl, setNavEl] = createSignal<HTMLElement>();
  const [drawerEl, setDrawerEl] = createSignal<HTMLElement>();

  const visible = createMemo(() => props.sections.filter((s) => (s.capability === undefined || props.can.can(s.capability))));
  const groups = createMemo(() =>
    GROUPS.map((g) => ({ ...g, items: visible().filter((s) => s.group === g.id) })).filter((g) => g.items.length > 0),
  );
  const current = (): SectionDef | undefined => visible().find((s) => s.id === props.route.section);

  onMount(() => void loadCatalog('admin-ux'));

  // Fields of the sections this admin may open, for the palette (UX10).
  const fields = createMemo<PaletteField[]>(() => {
    const open = new Map(visible().map((s) => [s.id, s]));
    return fieldAnchors()
      .filter((a) => open.has(a.section) && (a.capability === undefined || props.can.can(a.capability)))
      .map((a) => ({
        id: a.id,
        label: t(a.labelId),
        sectionLabel: open.get(a.section)!.label(),
        path: fieldPath(a.section, a.id, a.rest ?? []),
        ...(a.keywords ? { keywords: a.keywords } : {}),
      }));
  });
  // Field labels live in each section's catalog; load them when the palette opens.
  createEffect(() => {
    if (!paletteOpen()) return;
    for (const c of new Set(fieldAnchors().map((a) => a.catalog).filter((c): c is string => c !== undefined))) void loadCatalog(c);
  });

  // `?field=<id>`: focus and highlight the field once its section has drawn it.
  createEffect(
    on(
      () => [props.route.section, props.route.rest.join('/'), props.route.field] as const,
      ([, , field]) => {
        if (field === undefined) return;
        const ctl = new AbortController();
        void focusFieldWhenReady(field, { signal: ctl.signal });
        onCleanup(() => ctl.abort());
      },
    ),
  );

  createRovingTabindex(navEl, { orientation: 'vertical' });
  createFocusTrap(drawerEl, { active: drawerOpen, onEscape: () => setDrawerOpen(false) });

  onMount(() => {
    const onKey = (e: KeyboardEvent): void => {
      if ((e.ctrlKey || e.metaKey) && !e.altKey && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setDrawerOpen(false);
        setPaletteOpen(true);
      }
    };
    document.addEventListener('keydown', onKey);
    onCleanup(() => document.removeEventListener('keydown', onKey));
  });

  function go(id: string): void {
    setDrawerOpen(false);
    props.navigate(sectionPath(id));
  }

  const Nav = (p: { roving: boolean }): JSX.Element => (
    <nav ref={p.roving ? setNavEl : undefined} class={css.nav} aria-label={t('admin-nav')}>
      <For each={groups()}>
        {(g) => (
          <div class={css.group} role="group" aria-labelledby={`admin-group-${g.id}${p.roving ? '' : '-d'}`}>
            <h2 id={`admin-group-${g.id}${p.roving ? '' : '-d'}`} class={css.groupLabel}>
              {g.label()}
            </h2>
            <ul class={css.navList}>
              <For each={g.items}>
                {(s) => (
                  <li>
                    <a
                      class={css.navItem}
                      href={hrefFor(sectionPath(s.id))}
                      data-roving-item={p.roving ? '' : undefined}
                      data-section={s.id}
                      aria-current={props.route.section === s.id ? 'page' : undefined}
                      onClick={(e) => inPlace(e, () => go(s.id))}
                    >
                      <Icon name={s.icon} />
                      <span>{s.label()}</span>
                    </a>
                  </li>
                )}
              </For>
            </ul>
          </div>
        )}
      </For>
    </nav>
  );

  const Footer = (): JSX.Element => (
    <div class={css.sidebarFooter}>
      {props.languagePicker}
      <p class={css.help}>{t('admin-shell-signed-in-as', { name: isolate(props.username) })}</p>
      <button type="button" class={['btn btn--ghost', css.iconButton].join(' ')} onClick={() => props.onSignOut()}>
        <Icon name="signOut" />
        <span>{t('admin-sign-out')}</span>
      </button>
    </div>
  );

  const Brand = (): JSX.Element => (
    <span class={css.brand}>
      <span class={css.brandMark} aria-hidden="true">
        <span />
        <span />
        <span />
      </span>
      {t('admin-brand')}
    </span>
  );

  return (
    <div class={css.frame} data-screen="admin" data-sidebar-collapsed={sidebarCollapsed() ? '' : undefined} style={{ '--admin-sidebar-width': `${sidebarWidth()}px` }}>
      <aside id="admin-sidebar" class={css.sidebar}>
        <Brand />
        <Nav roving />
        <Footer />
      </aside>
      <div class={css.sidebarResize}>
        <ResizeHandle label={t('admin-shell-sidebar-resize')} controls="admin-sidebar" value={sidebarWidth()} min={180} max={480}
          direction={document.documentElement.dir === 'rtl' ? -1 : 1} onChange={setSidebarWidth} onCommit={saveLayout} />
      </div>

      <header class={css.topbar}>
        <button
          type="button"
          class={['btn btn--ghost', css.iconButton, css.phoneTarget].join(' ')}
          aria-expanded={drawerOpen()}
          aria-controls="admin-drawer"
          onClick={() => setDrawerOpen(true)}
        >
          <Icon name="menu" />
          <span class={css.srOnly}>{t('admin-shell-menu')}</span>
        </button>
        <span class={css.topbarTitle}>{current()?.label() ?? t('admin-brand')}</span>
        <button
          type="button"
          class={['btn btn--ghost', css.iconButton, css.phoneTarget].join(' ')}
          onClick={() => setPaletteOpen(true)}
        >
          <Icon name="search" />
          <span class={css.srOnly}>{t('admin-shell-search')}</span>
        </button>
      </header>

      <Show when={drawerOpen()}>
        <div class={css.scrim} onClick={() => setDrawerOpen(false)} />
        <div
          ref={setDrawerEl}
          id="admin-drawer"
          class={css.drawer}
          role="dialog"
          aria-modal="true"
          aria-label={t('admin-shell-menu')}
          tabindex="-1"
        >
          <div class={css.drawerHead}>
            <Brand />
            <button
              type="button"
              class={['btn btn--ghost', css.iconButton, css.phoneTarget].join(' ')}
              onClick={() => setDrawerOpen(false)}
            >
              <Icon name="close" />
              <span class={css.srOnly}>{t('admin-shell-close-menu')}</span>
            </button>
          </div>
          <Nav roving={false} />
          <Footer />
        </div>
      </Show>

      <main class={css.main} id="admin-main" tabindex="-1">
        <div class={css.mainHead}>
          <button type="button" class={`btn btn--ghost ${css.sidebarToggle}`} aria-expanded={!sidebarCollapsed()} aria-controls="admin-sidebar"
            aria-label={t(sidebarCollapsed() ? 'admin-shell-sidebar-show' : 'admin-shell-sidebar-hide')}
            title={t(sidebarCollapsed() ? 'admin-shell-sidebar-show' : 'admin-shell-sidebar-hide')}
            onClick={() => { setSidebarCollapsed(!sidebarCollapsed()); saveLayout(); }}><Icon name="menu" /></button>
          <Breadcrumbs items={props.breadcrumbs ?? []} />
          <button
            type="button"
            class={['btn btn--ghost', css.iconButton].join(' ')}
            onClick={() => setPaletteOpen(true)}
            aria-keyshortcuts="Control+K Meta+K"
          >
            <Icon name="search" />
            <span>{t('admin-shell-search')}</span>
            <kbd class={css.kbd}>{t('admin-shell-search-key')}</kbd>
          </button>
        </div>
        {props.children}
      </main>

      <CommandPalette
        open={paletteOpen()}
        onClose={() => setPaletteOpen(false)}
        sections={visible().map((s) => ({ id: s.id, label: s.label(), icon: s.icon, path: sectionPath(s.id) }))}
        fields={fields()}
        search={props.search}
        onOpen={(path) => props.navigate(path)}
      />

      <ConfirmDialog
        open={props.leave.open}
        title={t('admin-shell-leave-title')}
        body={t('admin-shell-leave-body')}
        confirmLabel={t('admin-shell-leave-confirm')}
        onConfirm={() => props.leave.onLeave()}
        onCancel={() => props.leave.onStay()}
      />
    </div>
  );
}
