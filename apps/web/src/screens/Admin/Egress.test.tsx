// Egress routes admin (26.20 t22-e16).
//
// The load-bearing assertions here are about what is NOT sent. `ProxyView` has no
// password field, so the screen has nothing to display and nothing to echo back —
// and the way that contract breaks is not the server returning a secret, it is the
// CLIENT sending a mask or a placeholder back as if it were one. That is invisible
// to a server test (the field arrives populated and well-formed) and it is exactly
// what these assert against: the request key is absent, not empty, not masked.

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import {
  AdminEgress,
  createHttpEgressAdminApi,
  isConnected,
  type EgressAdminApi,
  type EgressProxyView,
  type EgressTestResult,
  type PutProxyInput,
} from './Egress.tsx';

/** Every outcome the endpoint may return. Exactly one of them is a success. */
const ALL_OUTCOMES = [
  'connected',
  'authRejected',
  'refusedByPolicy',
  'dnsFailed',
  'unreachable',
  'originTlsFailed',
  'routeInvalid',
] as const;

function result(over: Partial<EgressTestResult> = {}): EgressTestResult {
  return {
    outcome: 'connected',
    stage: 'origin',
    endpoint: 'http://proxy.corp.example:3128',
    traversedProxy: true,
    detail: 'reached the origin',
    ...over,
  };
}

function view(over: Partial<EgressProxyView> = {}): EgressProxyView {
  return {
    id: 'corp',
    scheme: 'http',
    host: 'proxy.corp.example',
    port: 3128,
    username: 'svc',
    hasCredentials: true,
    allowPlaintext: false,
    createdAt: '2026-08-01T00:00:00Z',
    updatedAt: '2026-08-01T00:00:00Z',
    ...over,
  };
}

/** A fake admin API that records every request body it is handed. */
function fakeApi(rows: EgressProxyView[] = [view()], testResult?: EgressTestResult | Error) {
  const puts: PutProxyInput[] = [];
  const removed: string[] = [];
  const api: EgressAdminApi = {
    list: vi.fn(async () => rows),
    put: vi.fn(async (input: PutProxyInput) => {
      puts.push(input);
    }),
    remove: vi.fn(async (id: string) => {
      removed.push(id);
    }),
    test: vi.fn(async () => {
      if (testResult instanceof Error) throw testResult;
      return testResult ?? result();
    }),
  };
  return { api, puts, removed };
}

async function mounted(api: EgressAdminApi): Promise<void> {
  render(() => <AdminEgress api={api} />);
  await screen.findByTestId('admin-egress');
}

describe('egress routes — listing', () => {
  it('shows a configured route without ever showing a password', async () => {
    const { api } = fakeApi();
    await mounted(api);

    const row = await screen.findByTestId('egress-row-corp');
    expect(row.textContent).toContain('http://proxy.corp.example:3128');
    expect(row.textContent).toContain('svc');
    // The only credential fact the server states.
    expect(row.textContent).toContain('Set');
  });

  it('distinguishes a route with no credentials from one that has them', async () => {
    const { api } = fakeApi([view({ id: 'open', hasCredentials: false, username: '' })]);
    await mounted(api);
    const row = await screen.findByTestId('egress-row-open');
    expect(row.textContent).toContain('None');
  });

  it('says so when nothing is configured, rather than showing an empty table', async () => {
    const { api } = fakeApi([]);
    await mounted(api);
    expect(await screen.findByTestId('egress-empty')).toBeInTheDocument();
  });

  it('reports a failed load instead of rendering as empty', async () => {
    // An empty list and a failed fetch are different facts and must not look alike.
    const api: EgressAdminApi = {
      list: vi.fn(async () => {
        throw new Error('boom');
      }),
      put: vi.fn(),
      remove: vi.fn(),
      test: vi.fn(async () => result()),
    };
    await mounted(api);
    expect(await screen.findByRole('alert')).toBeInTheDocument();
    // And it must NOT also say "no routes configured, egress goes direct" — that
    // is a claim about the deployment, and a failed fetch did not establish it.
    expect(screen.queryByTestId('egress-empty')).toBeNull();
  });
});

describe('egress routes — credentials are write-only from this end too', () => {
  it('leaves the password field EMPTY when loading a route for editing', async () => {
    const { api } = fakeApi();
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-edit-corp'));

    const field = screen.getByTestId('egress-password') as HTMLInputElement;
    expect(field.value).toBe('');
    // A placeholder is allowed to allude to the stored value; it is not a value.
    expect(field.placeholder).not.toBe('');
    expect(field.type).toBe('password');
  });

  it('OMITS the password key entirely when editing without typing one', async () => {
    // THE assertion. Editing a port must not disturb the credential — and the way
    // that goes wrong is the client helpfully sending back what it was showing.
    const { api, puts } = fakeApi();
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-edit-corp'));

    fireEvent.input(screen.getByTestId('egress-port'), { target: { value: '8080' } });
    fireEvent.click(screen.getByTestId('egress-save'));

    await waitFor(() => expect(puts).toHaveLength(1));
    const sent = puts[0]!;
    expect(sent.port).toBe(8080);
    // Absent — not `''`, not the placeholder text, not `null`. `'password' in sent`
    // is the check that fails for every one of those.
    expect('password' in sent).toBe(false);
    // And nothing that looks like the placeholder leaked into any other field.
    expect(JSON.stringify(sent)).not.toContain('Leave blank');
  });

  it('sends exactly what was typed, when one is typed', async () => {
    // The control. Without it, "omits the password" also passes for a form that
    // can never send a password at all, which would be a different broken screen.
    const { api, puts } = fakeApi();
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-edit-corp'));

    fireEvent.input(screen.getByTestId('egress-password'), { target: { value: 'hunter2' } });
    fireEvent.click(screen.getByTestId('egress-save'));

    await waitFor(() => expect(puts).toHaveLength(1));
    expect(puts[0]!.password).toBe('hunter2');
  });

  it('clears the typed password after a save, so it cannot ride the next request', async () => {
    const { api, puts } = fakeApi();
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-edit-corp'));
    fireEvent.input(screen.getByTestId('egress-password'), { target: { value: 'hunter2' } });
    fireEvent.click(screen.getByTestId('egress-save'));
    await waitFor(() => expect(puts).toHaveLength(1));

    // Now edit again and change only the host.
    fireEvent.click(await screen.findByTestId('egress-edit-corp'));
    expect((screen.getByTestId('egress-password') as HTMLInputElement).value).toBe('');
    fireEvent.input(screen.getByTestId('egress-host'), { target: { value: 'other.example' } });
    fireEvent.click(screen.getByTestId('egress-save'));

    await waitFor(() => expect(puts).toHaveLength(2));
    expect('password' in puts[1]!).toBe(false);
  });

  it('refuses a password with no username, in place, rather than as a bare 400', async () => {
    const { api, puts } = fakeApi([view({ id: 'open', hasCredentials: false, username: '' })]);
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-edit-open'));
    fireEvent.input(screen.getByTestId('egress-password'), { target: { value: 'hunter2' } });
    fireEvent.click(screen.getByTestId('egress-save'));

    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(puts).toHaveLength(0);
  });
});

describe('egress routes — add and delete', () => {
  it('adds a route with the fields as entered', async () => {
    const { api, puts } = fakeApi([]);
    await mounted(api);

    fireEvent.input(screen.getByTestId('egress-id'), { target: { value: 'lab' } });
    fireEvent.change(screen.getByTestId('egress-scheme'), { target: { value: 'socks5' } });
    fireEvent.input(screen.getByTestId('egress-host'), { target: { value: 'socks.lab.example' } });
    fireEvent.input(screen.getByTestId('egress-port'), { target: { value: '1080' } });
    fireEvent.click(screen.getByTestId('egress-save'));

    await waitFor(() => expect(puts).toHaveLength(1));
    expect(puts[0]).toMatchObject({
      id: 'lab',
      scheme: 'socks5',
      host: 'socks.lab.example',
      port: 1080,
      allowPlaintext: false,
    });
    // A new route with nothing typed still sends no password key.
    expect('password' in puts[0]!).toBe(false);
  });

  it('refuses an incomplete route without calling the server', async () => {
    const { api, puts } = fakeApi([]);
    await mounted(api);
    fireEvent.input(screen.getByTestId('egress-id'), { target: { value: 'lab' } });
    fireEvent.click(screen.getByTestId('egress-save'));

    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(puts).toHaveLength(0);
  });

  it('keeps plaintext OFF unless it is explicitly turned on', async () => {
    // Deny-by-default: https is what stops a proxy reading what it carries, so
    // the opt-in must never be the default of a freshly typed route.
    const { api, puts } = fakeApi([]);
    await mounted(api);
    fireEvent.input(screen.getByTestId('egress-id'), { target: { value: 'lab' } });
    fireEvent.input(screen.getByTestId('egress-host'), { target: { value: 'h' } });
    fireEvent.input(screen.getByTestId('egress-port'), { target: { value: '1' } });
    fireEvent.click(screen.getByTestId('egress-save'));
    await waitFor(() => expect(puts).toHaveLength(1));
    expect(puts[0]!.allowPlaintext).toBe(false);
  });

  it('deletes a route by id', async () => {
    const { api, removed } = fakeApi();
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-delete-corp'));
    await waitFor(() => expect(removed).toEqual(['corp']));
  });

  it('does not let an edit rename a route into a second one', async () => {
    const { api } = fakeApi();
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-edit-corp'));
    // The id is the primary key: `put` is create-or-replace, so an editable id
    // would silently create a duplicate rather than rename.
    expect((screen.getByTestId('egress-id') as HTMLInputElement).disabled).toBe(true);
  });
});

describe('egress routes — the HTTP client', () => {
  it('talks to t22-e12 endpoints and reads its envelope', async () => {
    const calls: { url: string; init: RequestInit | undefined }[] = [];
    globalThis.fetch = vi.fn(async (url: string, init?: RequestInit) => {
      calls.push({ url, init });
      if (init?.method === 'POST') return new Response('{}', { status: 200 });
      return new Response(JSON.stringify({ proxies: [view()] }), { status: 200 });
    }) as unknown as typeof fetch;

    const api = createHttpEgressAdminApi('');
    expect(await api.list()).toHaveLength(1);
    await api.put({ id: 'corp', scheme: 'http', host: 'h', port: 1, username: '', allowPlaintext: false });
    await api.remove('corp');

    expect(calls.map((c) => c.url)).toEqual([
      '/admin/egress/proxies',
      '/admin/egress/proxies',
      '/admin/egress/proxies/corp/delete',
    ]);
    // Admin session is a cookie; a client-supplied account id is never trusted.
    expect(calls.every((c) => c.init?.credentials === 'same-origin' || c.init === undefined)).toBe(true);
  });

});

describe('egress routes — a test reports what happened, not whether a request succeeded', () => {
  it('shows a connected route as connected, with the stage it reached', async () => {
    const { api } = fakeApi([view()], result({ outcome: 'connected', stage: 'origin' }));
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-test-corp'));

    const row = await screen.findByTestId('egress-result-corp');
    expect(row.getAttribute('data-ok')).toBe('true');
    expect(screen.getByTestId('egress-verdict-corp').textContent).toContain('Connected');
    expect(screen.getByTestId('egress-stage-corp').textContent).toContain('origin');
  });

  it('CANNOT show success for any non-connected outcome, over the whole enum', async () => {
    // The assertion the endpoint was shaped for. Every one of these arrives as
    // HTTP 200 — a UI keying off the status calls all seven healthy, and a UI
    // using a denylist calls every future variant healthy too.
    for (const outcome of ALL_OUTCOMES) {
      const { api } = fakeApi([view()], result({ outcome }));
      const { unmount } = render(() => <AdminEgress api={api} />);
      await screen.findByTestId('admin-egress');
      fireEvent.click(await screen.findByTestId('egress-test-corp'));

      const row = await screen.findByTestId('egress-result-corp');
      expect(`${outcome}:${row.getAttribute('data-ok')}`).toBe(
        `${outcome}:${outcome === 'connected' ? 'true' : 'false'}`,
      );
      unmount();
    }
  });

  it('treats an unrecognised outcome as NOT success', async () => {
    // The enum is expected to grow — proxy auth rejection is being split out of a
    // bundled variant upstream. A value this build has never seen must not arrive
    // pre-approved as healthy.
    const { api } = fakeApi([view()], result({ outcome: 'proxyAuthRejected', stage: 'tunnel' }));
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-test-corp'));

    const row = await screen.findByTestId('egress-result-corp');
    expect(row.getAttribute('data-ok')).toBe('false');
    // And it is surfaced rather than swallowed, so the gap is visible.
    expect(screen.getByTestId('egress-verdict-corp').textContent).toContain('Unrecognised');
    expect(row.getAttribute('data-outcome')).toBe('proxyAuthRejected');
  });

  it('distinguishes two failures that differ only by stage', async () => {
    // "Refused by policy at the tunnel" and "refused by policy at the origin" are
    // different faults with different fixes; collapsing both to a red cross throws
    // away the reason the endpoint carries a stage at all.
    const a = fakeApi([view()], result({ outcome: 'refusedByPolicy', stage: 'tunnel' }));
    const first = render(() => <AdminEgress api={a.api} />);
    await screen.findByTestId('admin-egress');
    fireEvent.click(await screen.findByTestId('egress-test-corp'));
    const tunnelText = (await screen.findByTestId('egress-verdict-corp')).textContent ?? '';
    first.unmount();

    const b = fakeApi([view()], result({ outcome: 'refusedByPolicy', stage: 'origin' }));
    render(() => <AdminEgress api={b.api} />);
    await screen.findByTestId('admin-egress');
    fireEvent.click(await screen.findByTestId('egress-test-corp'));
    const originText = (await screen.findByTestId('egress-verdict-corp')).textContent ?? '';

    expect(tunnelText).not.toBe(originText);
    expect(tunnelText).toContain('Refused by policy');
    expect(originText).toContain('Refused by policy');
  });

  it('states whether the proxy was actually traversed', async () => {
    // From the transport's own progress, so it is a fact about this attempt rather
    // than a restatement of the configuration.
    const { api } = fakeApi([view()], result({ outcome: 'dnsFailed', stage: 'dns', traversedProxy: false }));
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-test-corp'));
    expect((await screen.findByTestId('egress-proxied-corp')).textContent).toContain('Did not traverse');
  });

  it('a test that could not RUN is not a verdict about the route', async () => {
    // 404/401/5xx mean we did not learn anything. Rendering that as a failed route
    // would be the mirror of rendering a failed route as success.
    const { api } = fakeApi([view()], new Error('could not run'));
    await mounted(api);
    fireEvent.click(await screen.findByTestId('egress-test-corp'));

    expect(await screen.findByTestId('egress-test-failed-corp')).toBeInTheDocument();
    expect(screen.queryByTestId('egress-result-corp')).toBeNull();
  });

  it('isConnected is an allowlist, not the absence of a known failure', async () => {
    expect(isConnected(result({ outcome: 'connected' }))).toBe(true);
    for (const outcome of ALL_OUTCOMES.filter((o) => o !== 'connected')) {
      expect(isConnected(result({ outcome }))).toBe(false);
    }
    expect(isConnected(result({ outcome: 'somethingAddedLater' }))).toBe(false);
    expect(isConnected(result({ outcome: '' }))).toBe(false);
  });

  it('the client posts to the test route and does NOT throw on a negative verdict', async () => {
    // The status carries only whether the test ran, so a 200 with a failing
    // outcome must resolve. Throwing here would collapse the distinction before
    // the UI ever sees it.
    const calls: string[] = [];
    globalThis.fetch = vi.fn(async (url: string) => {
      calls.push(url);
      return new Response(JSON.stringify(result({ outcome: 'authRejected', stage: 'connect' })), { status: 200 });
    }) as unknown as typeof fetch;

    const got = await createHttpEgressAdminApi('').test('corp');
    expect(calls).toEqual(['/admin/egress/proxies/corp/test']);
    expect(got.outcome).toBe('authRejected');
    expect(isConnected(got)).toBe(false);
  });

  it('the client DOES throw when the test itself could not run', async () => {
    globalThis.fetch = vi.fn(async () => new Response('{}', { status: 503 })) as unknown as typeof fetch;
    await expect(createHttpEgressAdminApi('').test('corp')).rejects.toThrow(/503/);
  });
});
