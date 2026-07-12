// Realtime access for components (plan §3 e6).
//
// Components (`SubTabStrip`, `ConnectionToast`) read the realtime controller via
// `useRealtime()`. It resolves the nearest `RealtimeContext` provider when one
// is present (tests wrap components with a fake controller), otherwise the app
// singleton the realtime store slice registers at boot. Keeping the singleton
// out of `AppState` is deliberate: the frozen `AppState` public shape stays
// byte-identical, so the 13 web tests and existing selectors don't change.

import { createContext, useContext } from 'solid-js';
import { createRealtimeController, type RealtimeController } from './controller.ts';

export const RealtimeContext = createContext<RealtimeController>();

let singleton: RealtimeController | null = null;

/** The app-wide controller, created lazily; the realtime slice replaces it. */
export function globalRealtime(): RealtimeController {
  if (singleton === null) singleton = createRealtimeController();
  return singleton;
}

/** Install the app singleton (called by the realtime store slice). */
export function setGlobalRealtime(controller: RealtimeController): void {
  singleton = controller;
}

/** Resolve the controller from context, falling back to the app singleton. */
export function useRealtime(): RealtimeController {
  const ctx = useContext(RealtimeContext);
  return ctx ?? globalRealtime();
}
