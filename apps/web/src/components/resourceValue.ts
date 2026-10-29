import type { Resource } from 'solid-js';

/**
 * A resource's value without suspending: the resolved value, kept while it
 * refreshes, and `undefined` while it is first loading or has failed.
 *
 * Calling `resource()` (or reading `resource.latest` before the first value)
 * while a fetch is in flight registers with the nearest `<Suspense>`. Inside
 * the mailbox that boundary is the route's own, around the whole shell: it
 * swaps the shell out for its "Loading…" fallback and back, and every message
 * frame moved out of the document and back in reloads (t60: blank bodies,
 * images fetched twice). Components under the shell read resources through
 * this instead.
 */
export function resourceValue<T>(resource: Resource<T>): T | undefined {
  return resource.state === 'ready' || resource.state === 'refreshing' ? resource.latest : undefined;
}
