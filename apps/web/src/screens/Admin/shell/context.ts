// Shell context: what every admin section may need from the console around it
// (capabilities, navigation, the injected data loaders). Provided by
// `screens/Admin/index.tsx`; tests provide their own.

import { createContext, useContext } from 'solid-js';
import type { Can } from './can.ts';
import type { AdminRouter } from './router.ts';
import type { DashboardLoader } from '../../../api/admin/dashboard.ts';
import type { AdminSearch } from './CommandPalette.tsx';

export interface ShellServices {
  loadDashboard: DashboardLoader;
  search: AdminSearch;
}

export interface ShellContextValue {
  can: Can;
  router: AdminRouter;
  services: ShellServices;
  /** The signed-in admin's name (for per-admin preferences). */
  username: () => string;
}

export const ShellContext = createContext<ShellContextValue>();

export function useShell(): ShellContextValue {
  const ctx = useContext(ShellContext);
  if (ctx === undefined) throw new Error('useShell must be used within <ShellContext.Provider>');
  return ctx;
}
