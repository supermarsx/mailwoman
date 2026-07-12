import { createContext, useContext } from 'solid-js';
import type { AppState } from './store.ts';

export const AppContext = createContext<AppState>();

export function useApp(): AppState {
  const ctx = useContext(AppContext);
  if (ctx === undefined) {
    throw new Error('useApp must be used within <AppContext.Provider>');
  }
  return ctx;
}
