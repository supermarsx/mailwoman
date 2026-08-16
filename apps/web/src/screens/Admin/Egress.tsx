// Egress-proxy routes admin (26.20 t22-e16), over t22-e12's API (`2eede09`):
//
//   GET  /admin/egress/proxies              → { proxies: ProxyView[] }
//   POST /admin/egress/proxies              ← PutProxyReq (create or replace)
//   POST /admin/egress/proxies/{id}/delete
//
// Admin-session-gated and cookie-authed, same as the sibling `/admin/plugins`
// client. A local section layered on the frozen `AdminSection` union, exactly as
// SSO / server-metadata / rethread / 2FA-policy already do — the union in
// `state/slices/admin.ts` is another lane's ownership boundary and stays untouched.
//
// ── Credentials are write-only, and that is a SERVER property the UI honours ──
// `ProxyView` carries no password field at all — not masked, not emptied — only
// `hasCredentials`. So this screen has nothing to display and nothing to send
// back, which is what makes the contract real rather than a UI convention. The
// form follows the established admin pattern (`Admin/Sso/index.tsx`): the field
// is blank on load, its placeholder says the stored value is kept, and the key is
// **omitted entirely** from the request unless the admin typed a new one.
//
// The server honours that: `35214a3` made an omitted password carry the sealed
// value forward (`None` keep / `Some("") ` clear / `Some(s)` set), so editing a
// host or port no longer destroys the credential. Before it, an omitted password
// was written as empty and every edit silently wiped the route's auth.
//
// ── "Test this route" reports what happened, not whether a request succeeded ──
// `POST /admin/egress/proxies/{id}/test` puts the VERDICT in the body and uses the
// status only for whether the test RAN. So `200` covers every outcome including
// the failures, and a UI keying off the status would call a broken route healthy.
// Success here is an allowlist of exactly `connected`; `outcome` and `stage` are
// both rendered, because "refused by policy at the tunnel" and "auth rejected at
// connect" are different problems with different fixes.

import { createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { basePath } from '../../api/basePath.ts';
import { t } from '../../i18n/index.ts';
import * as a11y from '../../components/mailA11y.css.ts';
import * as css from './admin.css.ts';

/** A configured route, as the server is willing to describe it. */
export interface EgressProxyView {
  id: string;
  scheme: string;
  host: string;
  port: number;
  username: string;
  /** Whether a password is sealed for this route. The password itself is never sent. */
  hasCredentials: boolean;
  allowPlaintext: boolean;
  createdAt: string;
  updatedAt: string;
}

/**
 * A create-or-replace request.
 *
 * `password` is optional and its ABSENCE is meaningful: omit it to keep whatever
 * is stored. It is never populated from a view (there is nothing to populate it
 * from) and never carries a mask or placeholder — a masked string sent back as a
 * value is the same defect as returning the secret, arrived at from the other end.
 */
export interface PutProxyInput {
  id: string;
  scheme: string;
  host: string;
  port: number;
  username: string;
  password?: string;
  allowPlaintext: boolean;
}

/**
 * What a route test concluded. Every value is a verdict the server reached; none
 * of them is an error in reaching it.
 */
export type EgressOutcome =
  | 'connected'
  | 'authRejected'
  | 'refusedByPolicy'
  | 'dnsFailed'
  | 'unreachable'
  | 'originTlsFailed'
  | 'routeInvalid';

/** How far the attempt got before it stopped — the part an operator acts on. */
export type EgressStage = 'dns' | 'connect' | 'tunnel' | 'origin';

/**
 * The body of `POST /admin/egress/proxies/{id}/test`.
 *
 * **The HTTP status says only whether the test RAN.** `200` means a verdict was
 * reached, *including a negative one*; `404` no such route, `401` not admin, `5xx`
 * the test itself could not run. So a client that keys off the status learns
 * nothing about the route — which is the whole reason the endpoint is shaped this
 * way, and the failure mode this screen must not reintroduce at the render layer.
 */
export interface EgressTestResult {
  /**
   * One of {@link EgressOutcome} — but typed as `string` deliberately.
   *
   * The enum may gain a variant (proxy auth rejection is being split out of a
   * bundled one upstream). A value this build does not recognise must render as
   * unrecognised; it must not fail to compile, and above all it must not fall
   * through to "success". Success is therefore an allowlist of exactly
   * `'connected'`, never the absence of a known failure.
   */
  outcome: string;
  /** One of {@link EgressStage}; `string` for the same reason as `outcome`. */
  stage: string;
  endpoint: string;
  /** Set from the transport's own progress, not from a route being configured —
   *  so it is displayable as fact rather than as intent. */
  traversedProxy: boolean;
  /** Human-readable, and never a credential. */
  detail: string;
}

/** Raised when an `/admin/egress/*` request fails. */
export class EgressApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'EgressApiError';
    this.status = status;
  }
}

/**
 * The admin surface this screen depends on. Tests pass a fake;
 * {@link createHttpEgressAdminApi} is the production `fetch` implementation.
 */
export interface EgressAdminApi {
  list(): Promise<EgressProxyView[]>;
  put(input: PutProxyInput): Promise<void>;
  remove(id: string): Promise<void>;
  /**
   * Run a live test of one route and return the verdict it reached.
   *
   * Throws only when the test could not be RUN (no such route, not admin, the
   * probe itself failed). A route that is broken is a resolved `EgressTestResult`
   * with a non-`connected` outcome — not an exception, and not an HTTP error.
   */
  test(id: string): Promise<EgressTestResult>;
}

/** The production client. Same-origin, cookie-authed against the admin domain. */
export function createHttpEgressAdminApi(base = basePath()): EgressAdminApi {
  const root = `${base}/admin/egress/proxies`;
  async function send(url: string, body?: unknown): Promise<Response> {
    const init: RequestInit = { method: 'POST', credentials: 'same-origin' };
    if (body !== undefined) {
      init.headers = { 'content-type': 'application/json' };
      init.body = JSON.stringify(body);
    }
    return fetch(url, init);
  }
  return {
    async list() {
      const res = await fetch(root, { credentials: 'same-origin' });
      if (!res.ok) throw new EgressApiError(res.status, `list egress proxies failed (${res.status})`);
      const body = (await res.json()) as { proxies?: EgressProxyView[] };
      return body.proxies ?? [];
    },
    async put(input) {
      const res = await send(root, input);
      if (!res.ok) throw new EgressApiError(res.status, `save egress proxy failed (${res.status})`);
    },
    async remove(id) {
      const res = await send(`${root}/${encodeURIComponent(id)}/delete`);
      if (!res.ok) throw new EgressApiError(res.status, `delete egress proxy failed (${res.status})`);
    },
    async test(id) {
      const res = await send(`${root}/${encodeURIComponent(id)}/test`);
      // `!res.ok` means the test could not run. It never means the ROUTE failed —
      // a failing route is a 200 carrying a negative outcome, and collapsing the
      // two here would throw away the distinction the endpoint exists to make.
      if (!res.ok) throw new EgressApiError(res.status, `test egress proxy failed (${res.status})`);
      return (await res.json()) as EgressTestResult;
    },
  };
}

/** Localised label per outcome. Unrecognised values fall to `unknown` — see
 *  `EgressTestResult.outcome` for why that path has to exist. */
const OUTCOME_LABEL: Record<string, () => string> = {
  connected: () => t('admin-egress-outcome-connected'),
  authRejected: () => t('admin-egress-outcome-auth-rejected'),
  refusedByPolicy: () => t('admin-egress-outcome-refused-by-policy'),
  dnsFailed: () => t('admin-egress-outcome-dns-failed'),
  unreachable: () => t('admin-egress-outcome-unreachable'),
  originTlsFailed: () => t('admin-egress-outcome-origin-tls-failed'),
  routeInvalid: () => t('admin-egress-outcome-route-invalid'),
};

const STAGE_LABEL: Record<string, () => string> = {
  dns: () => t('admin-egress-stage-dns'),
  connect: () => t('admin-egress-stage-connect'),
  tunnel: () => t('admin-egress-stage-tunnel'),
  origin: () => t('admin-egress-stage-origin'),
};

/**
 * Whether a verdict is a success.
 *
 * An ALLOWLIST of exactly one value, never `!isFailure`. The outcome set is
 * expected to grow, and with a denylist every future variant would arrive
 * pre-approved as healthy — the same defect as reading the HTTP status, moved
 * one layer in.
 */
export function isConnected(result: EgressTestResult): boolean {
  return result.outcome === 'connected';
}

function outcomeLabel(outcome: string): string {
  return (OUTCOME_LABEL[outcome] ?? (() => t('admin-egress-outcome-unknown')))();
}
function stageLabel(stage: string): string {
  return (STAGE_LABEL[stage] ?? (() => t('admin-egress-stage-unknown')))();
}

const SCHEMES = ['http', 'socks5'] as const;

export interface AdminEgressProps {
  /** Injected by tests; defaults to the production HTTP client. */
  api?: EgressAdminApi;
}

export function AdminEgress(props: AdminEgressProps): JSX.Element {
  const api = props.api ?? createHttpEgressAdminApi();

  const [routes, setRoutes] = createSignal<EgressProxyView[]>([]);
  const [loaded, setLoaded] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // ── Form state ────────────────────────────────────────────────────────────
  const [editingId, setEditingId] = createSignal<string | null>(null);
  const [id, setId] = createSignal('');
  const [scheme, setScheme] = createSignal<string>('http');
  const [host, setHost] = createSignal('');
  const [port, setPort] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [allowPlaintext, setAllowPlaintext] = createSignal(false);
  // Always blank — on first render and on every load-for-edit. There is no value
  // to seed it from, by design, and seeding it with a mask is what this must not do.
  const [password, setPassword] = createSignal('');

  // Per-route test verdicts, and separately the routes whose test could not be RUN
  // at all. Conflating them is the defect the endpoint's shape exists to prevent:
  // "this route is refused by policy" and "we could not ask" are different facts.
  const [verdicts, setVerdicts] = createSignal<Record<string, EgressTestResult>>({});
  const [testFailures, setTestFailures] = createSignal<Record<string, true>>({});
  const [testing, setTesting] = createSignal<string | null>(null);

  async function runTest(id: string): Promise<void> {
    setTesting(id);
    setTestFailures((prev) => {
      const next = { ...prev };
      delete next[id];
      return next;
    });
    try {
      const result = await api.test(id);
      setVerdicts((prev) => ({ ...prev, [id]: result }));
    } catch {
      // The test did not run. No verdict is recorded, because none was reached.
      setVerdicts((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
      setTestFailures((prev) => ({ ...prev, [id]: true }));
    } finally {
      setTesting(null);
    }
  }

  // Whether the last load actually SUCCEEDED, tracked separately from `routes()`
  // being empty. They are different facts: "this deployment configures no routes,
  // so egress goes direct" is a statement about the deployment, and rendering it
  // because a fetch failed asserts something we did not learn.
  const [listed, setListed] = createSignal(false);

  async function reload(): Promise<void> {
    try {
      setRoutes(await api.list());
      setListed(true);
      setError(null);
    } catch {
      setListed(false);
      setError(t('admin-egress-load-error'));
    } finally {
      setLoaded(true);
    }
  }
  onMount(() => void reload());

  function resetForm(): void {
    setEditingId(null);
    setId('');
    setScheme('http');
    setHost('');
    setPort('');
    setUsername('');
    setAllowPlaintext(false);
    setPassword('');
  }

  /** Load a route into the form. The password stays blank — see the file header. */
  function edit(row: EgressProxyView): void {
    setEditingId(row.id);
    setId(row.id);
    setScheme(row.scheme);
    setHost(row.host);
    setPort(String(row.port));
    setUsername(row.username);
    setAllowPlaintext(row.allowPlaintext);
    setPassword('');
  }

  /**
   * Build the request. The password key is present ONLY when the admin typed
   * something into this session's form; it is never derived from a view, a mask,
   * or a placeholder.
   */
  function buildInput(): PutProxyInput {
    const input: PutProxyInput = {
      id: id().trim(),
      scheme: scheme(),
      host: host().trim(),
      port: Number(port()),
      username: username().trim(),
      allowPlaintext: allowPlaintext(),
    };
    const typed = password();
    if (typed !== '') input.password = typed;
    return input;
  }

  async function save(e: Event): Promise<void> {
    e.preventDefault();
    const input = buildInput();
    if (input.id === '' || input.host === '' || !Number.isFinite(input.port) || input.port <= 0) {
      setError(t('admin-egress-invalid'));
      return;
    }
    // The server refuses this too; catching it here explains it in place rather
    // than as a bare 400.
    if (input.password !== undefined && input.username === '') {
      setError(t('admin-egress-username-required'));
      return;
    }
    try {
      await api.put(input);
      setError(null);
      resetForm();
      await reload();
    } catch {
      setError(t('admin-egress-save-error'));
    }
  }

  async function remove(row: EgressProxyView): Promise<void> {
    try {
      await api.remove(row.id);
      setError(null);
      if (editingId() === row.id) resetForm();
      await reload();
    } catch {
      setError(t('admin-egress-delete-error'));
    }
  }

  return (
    <section class={css.section} data-testid="admin-egress">
      <h2 class={css.heading}>{t('admin-egress-heading')}</h2>
      <p class={css.note}>{t('admin-egress-intro')}</p>

      <Show when={error() !== null}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <Show when={loaded()} fallback={<p class={css.note}>{t('common-loading')}</p>}>
        <Show
          when={routes().length > 0}
          fallback={
            // Only claim "nothing is configured" when we actually learned it.
            <Show when={listed()}>
              <p class={css.note} data-testid="egress-empty">
                {t('admin-egress-empty')}
              </p>
            </Show>
          }
        >
          <div class={css.tableWrap}>
            <table class={css.table}>
              <thead>
                <tr>
                  <th scope="col">{t('admin-egress-col-id')}</th>
                  <th scope="col">{t('admin-egress-col-route')}</th>
                  <th scope="col">{t('admin-egress-col-username')}</th>
                  <th scope="col">{t('admin-egress-col-credentials')}</th>
                  <th scope="col">{t('admin-egress-col-plaintext')}</th>
                  <th scope="col">{t('admin-egress-col-actions')}</th>
                </tr>
              </thead>
              <tbody>
                <For each={routes()}>
                  {(row) => (
                    <>
                    <tr data-testid={`egress-row-${row.id}`}>
                      <td class={css.mono}>{row.id}</td>
                      <td class={css.mono}>{`${row.scheme}://${row.host}:${row.port}`}</td>
                      <td>{row.username}</td>
                      <td>
                        {/* The only credential fact the server will state, and the
                            only one this screen needs. */}
                        {row.hasCredentials ? t('admin-egress-cred-set') : t('admin-egress-cred-none')}
                      </td>
                      <td>{row.allowPlaintext ? t('admin-egress-plaintext-on') : t('admin-egress-plaintext-off')}</td>
                      <td>
                        <button
                          type="button"
                          class={`btn btn--ghost ${a11y.focusable}`}
                          data-testid={`egress-edit-${row.id}`}
                          onClick={() => edit(row)}
                        >
                          {t('common-edit')}
                        </button>
                        <button
                          type="button"
                          class={`btn btn--ghost ${a11y.focusable}`}
                          data-testid={`egress-test-${row.id}`}
                          disabled={testing() === row.id}
                          onClick={() => void runTest(row.id)}
                        >
                          {testing() === row.id ? t('admin-egress-testing') : t('admin-egress-test')}
                        </button>
                        <button
                          type="button"
                          class={`btn btn--ghost ${a11y.focusable}`}
                          data-testid={`egress-delete-${row.id}`}
                          onClick={() => void remove(row)}
                        >
                          {t('common-delete')}
                        </button>
                      </td>
                    </tr>
                    <Show when={verdicts()[row.id] !== undefined}>
                      {(_present) => {
                        const r = (): EgressTestResult => verdicts()[row.id]!;
                        return (
                          <tr
                            data-testid={`egress-result-${row.id}`}
                            data-outcome={r().outcome}
                            // The single machine-readable success bit, derived from
                            // the allowlist — never from the HTTP status, which is
                            // 200 for failures too.
                            data-ok={String(isConnected(r()))}
                          >
                            <td colSpan={6}>
                              <p
                                class={isConnected(r()) ? css.note : css.error}
                                role="status"
                                data-testid={`egress-verdict-${row.id}`}
                              >
                                <strong>{outcomeLabel(r().outcome)}</strong>
                                {' — '}
                                {/* The stage is half the diagnosis: the same failure
                                    at `connect` and at `origin` are different faults. */}
                                <span data-testid={`egress-stage-${row.id}`}>{stageLabel(r().stage)}</span>
                              </p>
                              <p class={css.note}>
                                <span class={css.mono}>{r().endpoint}</span>
                                {' · '}
                                <span data-testid={`egress-proxied-${row.id}`}>
                                  {r().traversedProxy
                                    ? t('admin-egress-proxied-yes')
                                    : t('admin-egress-proxied-no')}
                                </span>
                              </p>
                              <Show when={r().detail !== ''}>
                                <p class={css.note}>{r().detail}</p>
                              </Show>
                            </td>
                          </tr>
                        );
                      }}
                    </Show>
                    <Show when={testFailures()[row.id] === true}>
                      <tr data-testid={`egress-test-failed-${row.id}`}>
                        <td colSpan={6}>
                          {/* Distinct from every verdict: we did not learn anything
                              about the route, so nothing is claimed about it. */}
                          <p class={css.error} role="alert">
                            {t('admin-egress-test-failed')}
                          </p>
                        </td>
                      </tr>
                    </Show>
                  </>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </Show>

      <form class={css.card} onSubmit={(e) => void save(e)}>
        <h3 class={css.heading}>
          {editingId() !== null ? t('admin-egress-edit-heading') : t('admin-egress-add-heading')}
        </h3>
        <div class={css.grid}>
          <label>
            <span>{t('admin-egress-id')}</span>
            <input
              value={id()}
              onInput={(e) => setId(e.currentTarget.value)}
              data-testid="egress-id"
              // The id is the primary key: editing it would create a second route
              // rather than rename this one.
              disabled={editingId() !== null}
            />
          </label>
          <label>
            <span>{t('admin-egress-scheme')}</span>
            <select value={scheme()} onChange={(e) => setScheme(e.currentTarget.value)} data-testid="egress-scheme">
              <For each={SCHEMES}>{(s) => <option value={s}>{s}</option>}</For>
            </select>
          </label>
          <label>
            <span>{t('admin-egress-host')}</span>
            <input value={host()} onInput={(e) => setHost(e.currentTarget.value)} data-testid="egress-host" />
          </label>
          <label>
            <span>{t('admin-egress-port')}</span>
            <input
              type="number"
              value={port()}
              onInput={(e) => setPort(e.currentTarget.value)}
              data-testid="egress-port"
            />
          </label>
          <label>
            <span>{t('admin-egress-username')}</span>
            <input
              value={username()}
              onInput={(e) => setUsername(e.currentTarget.value)}
              data-testid="egress-username"
              autocomplete="off"
            />
          </label>
          <label>
            <span>{t('admin-egress-password')}</span>
            <input
              type="password"
              value={password()}
              onInput={(e) => setPassword(e.currentTarget.value)}
              data-testid="egress-password"
              autocomplete="new-password"
              // The placeholder is the ONLY place the stored credential is ever
              // alluded to, and it is not a value: it never becomes one, because
              // the request key is omitted whenever this field is empty.
              placeholder={editingId() !== null ? t('admin-egress-password-unchanged') : ''}
            />
          </label>
        </div>
        <label>
          <input
            type="checkbox"
            checked={allowPlaintext()}
            onChange={(e) => setAllowPlaintext(e.currentTarget.checked)}
            data-testid="egress-allow-plaintext"
          />
          <span>{t('admin-egress-allow-plaintext')}</span>
        </label>
        <p class={css.note}>{t('admin-egress-allow-plaintext-note')}</p>
        <div>
          <button type="submit" class={`btn btn--primary ${a11y.focusable}`} data-testid="egress-save">
            {editingId() !== null ? t('common-save') : t('common-add')}
          </button>
          <Show when={editingId() !== null}>
            <button
              type="button"
              class={`btn btn--ghost ${a11y.focusable}`}
              data-testid="egress-cancel"
              onClick={() => resetForm()}
            >
              {t('common-cancel')}
            </button>
          </Show>
        </div>
      </form>
    </section>
  );
}
