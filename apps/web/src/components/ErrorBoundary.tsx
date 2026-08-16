// Error boundaries for the app's async surfaces, and the lazy-route wrapper that
// makes "Retry" mean something for a failed dynamic import.
//
// The app had NO error boundary of any kind. Every `lazy()` route sat inside a
// bare `<Suspense fallback="Loading…">`, so a chunk that fails to load — a 404
// against a tab open across a redeploy, or one dropped request — left that
// fallback on screen permanently. Nothing recovered it and nothing reported it;
// the surface simply never arrived.
//
// The retry is the part that needs care. Solid's `lazy()` memoises the import
// promise, INCLUDING its rejection: re-rendering the same lazy component after a
// failure replays the stored error without touching the network. So a boundary
// whose retry only calls `reset()` produces a button that cannot succeed, which
// is worse than showing none — it reads as a remedy and is not one. `LazyRoute`
// therefore builds a NEW `lazy()` per attempt, so retrying genuinely re-imports.

import {
  ErrorBoundary as SolidErrorBoundary,
  createMemo,
  createSignal,
  lazy,
  Suspense,
  type Component,
  type JSX,
} from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { AsyncError, AsyncPending } from './AsyncState.tsx';

/**
 * Catch anything thrown while rendering `children` and offer a way back.
 *
 * `reset()` re-renders the subtree, which recovers a surface whose failure was
 * transient (a signal that was briefly inconsistent, a fetch the caller will
 * re-issue). `onRetry` runs FIRST, so a caller can re-arm whatever it owns —
 * refetch, re-import — before the subtree is rebuilt.
 */
export function AsyncBoundary(props: {
  children: JSX.Element;
  onRetry?: (() => void) | undefined;
  /**
   * Renders the caught failure. Defaults to an error message plus a retry.
   * Override for a surface whose documented contract is to stay invisible when
   * it cannot load — for those, an error card IS the regression.
   */
  fallback?: ((error: unknown, retry: () => void) => JSX.Element) | undefined;
}): JSX.Element {
  return (
    <SolidErrorBoundary
      fallback={(error: unknown, reset: () => void) => {
        const retry = (): void => {
          props.onRetry?.();
          reset();
        };
        return props.fallback !== undefined ? (
          props.fallback(error, retry)
        ) : (
          <AsyncError error={error} onRetry={retry} />
        );
      }}
    >
      {props.children}
    </SolidErrorBoundary>
  );
}

/** What a dynamic `import()` of a route module resolves to. */
export type RouteModule = { default: Component };

/**
 * A lazily-imported route with a bounded pending state and a retry that really
 * re-imports.
 *
 * Two failure modes, deliberately handled in two places:
 *
 *  * **The import rejects** (chunk missing, network down). Caught in the loader
 *    and turned into an error state directly, so this does not depend on a
 *    boundary catching a rejected lazy — behaviour that varies between Solid
 *    versions and would fail silently as a permanent spinner if it changed.
 *  * **The module loads and then throws while rendering.** Nothing in the loader
 *    can see that; `AsyncBoundary` catches it.
 *
 * Bumping `attempt` rebuilds the memo, which constructs a fresh `lazy()` around
 * a fresh `import()` — the only way past Solid's memoised rejection.
 */
export function LazyRoute(props: {
  load: () => Promise<RouteModule>;
  /** Shown while the chunk is in flight; bounded like every pending state. */
  message?: string;
  timeoutMs?: number;
  /**
   * For a surface whose contract is to render nothing when it is unavailable
   * (the UI-plugin tier). It shows no pending state and no error card — but it
   * is still bounded and still contained: a failure cannot escape to the parent.
   */
  failSoft?: boolean;
}): JSX.Element {
  const [attempt, setAttempt] = createSignal(0);
  const retry = (): void => {
    setAttempt((n) => n + 1);
  };

  const component = createMemo<Component>(() => {
    attempt(); // rebuild on retry: a new lazy(), a new import()
    return lazy(async () => {
      try {
        return await props.load();
      } catch (error) {
        // Resolve to a VIEW rather than rejecting: a rejected lazy is the path
        // that produced the permanent spinner.
        if (props.failSoft === true) return { default: () => null };
        return { default: () => <AsyncError error={error} onRetry={retry} /> };
      }
    });
  });

  return (
    <AsyncBoundary
      onRetry={retry}
      {...(props.failSoft === true ? { fallback: () => null } : {})}
    >
      <Suspense
        fallback={
          props.failSoft === true ? null : (
            <AsyncPending
              {...(props.message !== undefined ? { message: props.message } : {})}
              {...(props.timeoutMs !== undefined ? { timeoutMs: props.timeoutMs } : {})}
              onRetry={retry}
            />
          )
        }
      >
        <Dynamic component={component()} />
      </Suspense>
    </AsyncBoundary>
  );
}
