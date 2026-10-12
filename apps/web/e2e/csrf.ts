import { test as base, type APIRequestContext } from '@playwright/test';

/**
 * Playwright `test` for specs that write to mw-server WITHOUT going through the
 * web client — through the `request` fixture, `page.request`, or a context made
 * with `playwright.request.newContext()`.
 *
 * Every e2e stack runs with `MW_CSRF_STRICT=true` (the `deploy.sh` default). In
 * that mode a POST, PUT, PATCH or DELETE that a cookie authorises is answered
 * `403 {"error":"missing or invalid CSRF token"}` unless it repeats the `mw_csrf`
 * cookie in `X-CSRF-Token`. The web client does that in `apiFetch`
 * (`src/api/http.ts`). A spec that talks to the server directly is its own
 * client, so it has to do the same; importing `test` from this file instead of
 * `@playwright/test` makes those three request contexts do it:
 *
 *   on a write, when the context's cookie jar holds `mw_csrf` and the call set no
 *   `x-csrf-token` of its own, the cookie's value is sent as `X-CSRF-Token`.
 *
 * Nothing else changes: no header is added to a GET, or when there is no such
 * cookie (before sign-in, or against a route that needs none).
 *
 * Convention: a spec that writes with a request context imports `test` and
 * `expect` from `./csrf.ts`. A spec that drives the UI only does not need it.
 *
 * What this does NOT show is that the server enforces anything — these requests
 * always carry the token. `csrf-strict.spec.ts`, which runs in every project,
 * sends a write without it and expects the refusal; it imports the stock
 * `@playwright/test` so that nothing adds the header for it.
 */

export * from '@playwright/test';

const CSRF_COOKIE = 'mw_csrf';
const CSRF_HEADER = 'x-csrf-token';
const SAFE_METHODS = new Set(['GET', 'HEAD', 'OPTIONS']);
const WRITE_VERBS = ['post', 'put', 'patch', 'delete'] as const;

type Options = { headers?: { [key: string]: string }; method?: string };

const patched = new WeakSet<object>();

/** The `mw_csrf` value in `ctx`'s cookie jar, preferring the cookie for `url`'s host. */
async function csrfCookie(ctx: APIRequestContext, url: string): Promise<string | undefined> {
  const { cookies } = await ctx.storageState();
  const candidates = cookies.filter((c) => c.name === CSRF_COOKIE && c.value !== '');
  let host: string | undefined;
  try {
    host = new URL(url).hostname;
  } catch {
    host = undefined; // a path: resolved against the context's baseURL
  }
  const forHost = candidates.find((c) => host !== undefined && c.domain.replace(/^\./, '') === host);
  return (forHost ?? candidates[0])?.value;
}

/** `options` with `X-CSRF-Token` added, unless the call set one or there is no cookie. */
async function withToken<O extends Options>(ctx: APIRequestContext, url: string, options: O | undefined): Promise<O | undefined> {
  const given = options?.headers ?? {};
  if (Object.keys(given).some((k) => k.toLowerCase() === CSRF_HEADER)) return options;
  const token = await csrfCookie(ctx, url);
  if (token === undefined) return options;
  return { ...(options as O), headers: { ...given, [CSRF_HEADER]: token } };
}

/**
 * Make `ctx` send the CSRF token on writes (see the file comment). Idempotent.
 * Exported for a spec that builds a request context some other way.
 */
export function withCsrf<T extends APIRequestContext>(ctx: T): T {
  if (patched.has(ctx)) return ctx;
  patched.add(ctx);
  for (const verb of WRITE_VERBS) {
    const original = ctx[verb].bind(ctx);
    ctx[verb] = async (url, options) => original(url, await withToken(ctx, url, options));
  }
  const originalFetch = ctx.fetch.bind(ctx);
  ctx.fetch = async (urlOrRequest, options) => {
    const isUrl = typeof urlOrRequest === 'string';
    const method = (options?.method ?? (isUrl ? 'GET' : urlOrRequest.method())).toUpperCase();
    if (SAFE_METHODS.has(method)) return originalFetch(urlOrRequest, options);
    const url = isUrl ? urlOrRequest : urlOrRequest.url();
    return originalFetch(urlOrRequest, await withToken(ctx, url, options));
  };
  return ctx;
}

export const test = base.extend<object, { csrfRequestContexts: void }>({
  request: async ({ request }, use) => {
    await use(withCsrf(request));
  },
  // `page.request` is `context.request`.
  context: async ({ context }, use) => {
    withCsrf(context.request);
    await use(context);
  },
  // `playwright.request.newContext()`: patched once per worker, for every context
  // made after that.
  csrfRequestContexts: [
    async ({ playwright }, use) => {
      const factory = playwright.request;
      if (!patched.has(factory)) {
        patched.add(factory);
        const original = factory.newContext.bind(factory);
        factory.newContext = async (options) => withCsrf(await original(options));
      }
      await use();
    },
    { scope: 'worker', auto: true },
  ],
});
