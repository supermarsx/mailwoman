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
// ⚠ ORDERING HAZARD, live as of `2eede09`. `Store::put_egress_proxy` seals
// `password.unwrap_or("")` and writes `sealed_password = excluded.sealed_password`
// unconditionally, so an OMITTED password overwrites the stored one with empty —
// editing a route's host or port would silently destroy its credentials, and the
// failure would surface much later as an auth error at fetch time. The fix is
// server-side (`None` ⇒ carry the sealed value forward, as `admin_sso.rs:176`
// already does, with the comment "renaming/enabling a backend must not silently
// wipe its secret"). This screen is written for the CORRECTED server and must not
// ship ahead of it.

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
 *
 * ── The "test this route" seam ───────────────────────────────────────────────
 * There is deliberately **no `test` method here yet, and no Test control renders**.
 *
 * `t22-e12` ships list/put/delete and nothing else, so a Test button today could
 * only report success without having asked anything — the same shape as a retry
 * that cannot recover, and precisely the defect this screen is meant to avoid.
 *
 * When the endpoint is designed, it attaches HERE as one method, and the outcome
 * type is defined by that endpoint rather than invented at this end. It has to
 * distinguish at least **connected**, **authenticated**, **refused by policy**,
 * and **name did not resolve**. Note that the refusal alone cannot carry that:
 * `t22-e11`'s `ProxyRefusal` has seven variants, but `Refusal::Blocked` is
 * documented as deliberately coarse and does not separate "private address" from
 * "does not resolve" — so the endpoint must say, and the UI must not guess.
 */
export interface EgressAdminApi {
  list(): Promise<EgressProxyView[]>;
  put(input: PutProxyInput): Promise<void>;
  remove(id: string): Promise<void>;
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
  };
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
                          data-testid={`egress-delete-${row.id}`}
                          onClick={() => void remove(row)}
                        >
                          {t('common-delete')}
                        </button>
                      </td>
                    </tr>
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
