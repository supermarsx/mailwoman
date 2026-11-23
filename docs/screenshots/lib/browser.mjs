// Browser setup and the signed-in API helpers the seeds and captures share.
import { webRequire } from './paths.mjs';

let pw;
/** Playwright, from apps/web's node_modules (no install of its own). */
export function playwright() {
  pw ??= webRequire('@playwright/test');
  return pw;
}

/**
 * A desktop Chrome user agent without "HeadlessChrome": request protection
 * refuses the headless one, and the app's shared worker takes the process UA,
 * not a context's, so it is set as a launch argument.
 */
export function userAgent() {
  return playwright().devices['Desktop Chrome'].userAgent;
}

export function launchArgs() {
  return [`--user-agent=${userAgent()}`, '--font-render-hinting=none', '--disable-lcd-text', '--force-color-profile=srgb'];
}

export async function launch() {
  return playwright().chromium.launch({ args: launchArgs() });
}

/** The `mw_csrf` cookie of `context`, sent back as X-CSRF-Token on writes. */
async function csrf(context) {
  const { cookies } = await context.storageState();
  return cookies.find((c) => c.name === 'mw_csrf' && c.value)?.value;
}

/**
 * A request helper over `context` (cookies shared with its pages): JSON in and
 * out, CSRF token on writes, the storage-authority header when known, one retry
 * on the sign-in cooldown.
 */
export function api(context, base) {
  let authority;
  let account;
  const call = async (method, path, body, extraHeaders = {}) => {
    const headers = { origin: base, ...extraHeaders };
    if (method !== 'GET') {
      const token = await csrf(context);
      if (token) headers['x-csrf-token'] = token;
      // The server fences a write that names a storage authority to the session's account.
      if (authority && account) {
        headers['x-mailwoman-storage-authority'] = authority;
        headers['x-mailwoman-account'] = account;
      }
    }
    const url = path.startsWith('http') ? path : `${base}${path}`;
    let response;
    for (let attempt = 0; attempt < 4; attempt += 1) {
      response = await context.request.fetch(url, { method, headers, data: body === undefined ? undefined : body, failOnStatusCode: false });
      if (response.status() !== 429) break;
      const wait = Number(response.headers()['retry-after'] ?? '3');
      await new Promise((r) => setTimeout(r, (Number.isFinite(wait) ? Math.min(wait, 30) : 3) * 1000 + 250));
    }
    const text = await response.text();
    let json = null;
    try {
      json = text ? JSON.parse(text) : null;
    } catch {
      /* not JSON */
    }
    return { status: response.status(), ok: response.ok(), json, text };
  };
  const must = async (method, path, body, headers) => {
    const r = await call(method, path, body, headers);
    if (!r.ok) throw new Error(`${method} ${path}: ${r.status} ${r.text.slice(0, 300)}`);
    return r;
  };
  return {
    call,
    must,
    /** Sign in as the demo user; remembers the storage authority for settings writes. */
    async signInUser(user) {
      await must('POST', '/api/login', { username: user.username, password: user.password });
      const me = await must('GET', '/api/me');
      authority = me.json?.storageAuthority;
      account = me.json?.accountId;
      return me.json;
    },
    /** Use the session the context already has (cookies from storage or a form sign-in). */
    async adoptUserSession() {
      const me = await must('GET', '/api/me');
      authority = me.json?.storageAuthority;
      account = me.json?.accountId;
      return me.json;
    },
    async signInAdmin(stack) {
      await must('POST', `${stack.adminPath}/login`, { username: stack.admin.username, password: stack.admin.password });
    },
  };
}
