// FROZEN app-shell module registry (plan §2.5) — the V2-lesson wiring contract.
// e0 authors this registry + the four stub module entries + their route stubs;
// the four Batch-B web builders (e4–e7) fill each module's `mount()` + views;
// **e10 registers them into the running shell nav/router and asserts each is
// reachable** (the explicit mount step V2 lacked, plan risk #1). e10 now points
// each `mount()` at the ENGINE-BACKED host (`./mounts.tsx`): the mock backends are
// gone from the running app (Calendar renders over `app.calendarController()`;
// Tasks/Notes/Contacts render over their slices, which speak `client.jmap`). The
// shell router (`./router.ts`) turns the `route` fields below into live surfaces.
//
// The registry is the single source of truth for the shell: nav-rail entries,
// per-module ribbon tabs, command-palette entries, and routes all derive from
// it, so a module cannot ship "unit-green but unmounted".

import { lazy, type JSX } from 'solid-js';

// Each module is lazy-loaded into its own chunk, off the login→inbox mail
// critical path (plan risk #10 / e11 bundle gate). Calendar routes through the
// engine-backed host (`./mounts.tsx`); Tasks/Notes/Contacts are engine-backed via
// their slices, so their module components are lazy-imported directly. These are
// module-level constants (stable references) so `mount()` never re-creates them.
const CalendarMount = lazy(() => import('./mounts.tsx'));
const TasksMount = lazy(() => import('../modules/tasks/index.tsx').then((m) => ({ default: m.TasksModule })));
const NotesMount = lazy(() => import('../modules/notes/index.tsx').then((m) => ({ default: m.NotesModule })));
const ContactsMount = lazy(() => import('../modules/contacts/index.tsx').then((m) => ({ default: m.ContactsModule })));

/** A module's root view component (a Solid component function). */
export type ModuleComponent = () => JSX.Element;

/** A ribbon tab a module contributes to the shell ribbon (§2.5). */
export interface RibbonTabEntry {
  id: string;
  label: string;
}

/** A command-palette entry a module contributes (§2.5). `run` is wired to real
 *  actions by each module / e10; the registry only declares the seam. */
export interface CommandPaletteEntry {
  id: string;
  label: string;
  run: () => void;
}

/**
 * One mountable app module (§2.5). `mount()` returns the module's root
 * component — a factory (not the component directly) so e10 can lazy-load it
 * off the mail critical path via a dynamic import (plan risk #10).
 */
export interface AppModule {
  /** Stable module id (`'calendar' | 'tasks' | 'notes' | 'contacts'`). */
  id: string;
  /** Nav-rail label. */
  label: string;
  /** Nav-rail glyph (emoji, matching the ribbon icon style). */
  icon: string;
  /** Base route (§2.5 uses `/calendar/:view?`, `/tasks`, `/notes/:id?`,
   *  `/contacts/:id?`; the param forms are wired by e10's router). */
  route: string;
  /** Produce the module's root component (direct now; lazy `import()` at e10). */
  mount: () => ModuleComponent;
  ribbonTabs: RibbonTabEntry[];
  commandPaletteEntries: CommandPaletteEntry[];
}

/**
 * The four V3 PIM modules (plan §0.5). Mail stays the existing screen; these
 * mount beside it. `route` is the base hash route (plan §2.5 param forms:
 * `/calendar/:view?`, `/tasks`, `/notes/:id?`, `/contacts/:id?`); the shell
 * router (`./router.ts`) resolves them. `mount()` returns the engine-backed host.
 */
export const APP_MODULES: readonly AppModule[] = [
  {
    id: 'calendar',
    label: 'Calendar',
    icon: '📅',
    route: '/calendar',
    mount: () => CalendarMount,
    ribbonTabs: [],
    commandPaletteEntries: [],
  },
  {
    id: 'tasks',
    label: 'Tasks',
    icon: '✅',
    route: '/tasks',
    mount: () => TasksMount,
    ribbonTabs: [],
    commandPaletteEntries: [],
  },
  {
    id: 'notes',
    label: 'Notes',
    icon: '🗒️',
    route: '/notes',
    mount: () => NotesMount,
    ribbonTabs: [],
    commandPaletteEntries: [],
  },
  {
    id: 'contacts',
    label: 'Contacts',
    icon: '👤',
    route: '/contacts',
    mount: () => ContactsMount,
    ribbonTabs: [],
    commandPaletteEntries: [],
  },
];

/** Look up a module by id (nav/router helper for e10). */
export function moduleById(id: string): AppModule | undefined {
  return APP_MODULES.find((m) => m.id === id);
}
