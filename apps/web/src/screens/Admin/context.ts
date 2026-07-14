// Admin panel context (plan §3 e7). Provides the reactive `AdminSlice` (session +
// typed `/admin/*` client) to every section so they never re-thread props. Scoped
// to the lazily-loaded admin screen — nothing here is pulled into the mailbox
// bundle (the whole `screens/Admin/**` tree is reached only via `lazy(import)`).

import { createContext, useContext } from 'solid-js';
import type { AdminSlice } from '../../state/slices/admin.ts';

export const AdminContext = createContext<AdminSlice>();

export function useAdmin(): AdminSlice {
  const ctx = useContext(AdminContext);
  if (ctx === undefined) {
    throw new Error('useAdmin must be used within <AdminContext.Provider>');
  }
  return ctx;
}
