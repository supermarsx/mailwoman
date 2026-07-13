// Platform access for components (plan §3 e0/e6).
//
// Components consume the capability layer via `usePlatform()`. It resolves the
// nearest `PlatformContext` provider when one is present (tests wrap components
// with a fake `Platform`), otherwise the app-wide impl from `getPlatform()`
// (browser by default; the native impl after e7 calls `initPlatform()` at boot).
//
// Mirrors `realtime/context.ts`: keeping the singleton out of `AppState` leaves the
// frozen `AppState` public shape byte-identical, so existing web tests/selectors
// are untouched — the platform layer is strictly additive.

import { createContext, useContext } from 'solid-js';
import { getPlatform, type Platform } from './index.ts';

export const PlatformContext = createContext<Platform>();

/** Resolve the platform from context, falling back to the app-wide impl. */
export function usePlatform(): Platform {
  const ctx = useContext(PlatformContext);
  return ctx ?? getPlatform();
}
