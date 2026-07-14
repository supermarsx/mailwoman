import { test, expect } from '@playwright/test';

/**
 * V6 live E2E — MCP (plan §3 e13): a real MCP client handshake over the mounted
 * Streamable-HTTP transport → tools/list (the frozen 10 tools) → untrusted
 * provenance on mail-content tools → mail.send is enumerated but GATED (an
 * unauthenticated tools/call is refused — never a raw transmit).
 *
 * /mcp is mounted in ENGINE mode. If the `v6` project points at a proxy-mode
 * server, the endpoint is absent and this spec skips LOUDLY (no silent skip). The
 * gated mail.send→Outbox with a transmitted()==0 assertion is hard-proven by
 * mw-mcp's e4 unit suite (13 tests) and the Rust harness.
 */
async function rpc(request: import('@playwright/test').APIRequestContext, body: unknown) {
  return request.post('/mcp', { data: body, headers: { 'content-type': 'application/json' } });
}

test.describe('v6 MCP server (live)', () => {
  test('handshake → tools/list(10) → untrusted provenance → gated send', async ({ request }) => {
    const initResp = await rpc(request, { jsonrpc: '2.0', id: 1, method: 'initialize', params: {} });
    test.skip(
      initResp.status() === 404,
      'MCP (/mcp) not mounted — the v6 project targets a proxy-mode server; ENGINE mode required.',
    );

    const init = await initResp.json();
    expect(init.result.serverInfo.name).toBe('mailwoman-mcp');

    const list = await (await rpc(request, {
      jsonrpc: '2.0',
      id: 2,
      method: 'tools/list',
      params: {},
    })).json();
    const tools = list.result.tools as Array<{ name: string; _meta?: { untrustedOutput?: boolean } }>;
    expect(tools.length, 'frozen 10-tool set').toBe(10);

    const names = tools.map((t) => t.name);
    for (const expected of [
      'mail.search', 'mail.read', 'folders.list', 'drafts.create', 'mail.send',
      'calendar.read', 'calendar.propose', 'tasks.read', 'tasks.write', 'contacts.read',
    ]) {
      expect(names, `tool ${expected} enumerated`).toContain(expected);
    }

    // Prompt-injection posture: mail-content tools declare untrusted output.
    expect(tools.find((t) => t.name === 'mail.search')?._meta?.untrustedOutput).toBe(true);
    expect(tools.find((t) => t.name === 'mail.read')?._meta?.untrustedOutput).toBe(true);

    // Send is enumerated but not open: an unauthenticated tools/call is refused.
    const unauth = await (await rpc(request, {
      jsonrpc: '2.0',
      id: 3,
      method: 'tools/call',
      params: { name: 'mail.send', arguments: { to: ['x@y.test'], subject: 's', bodyText: 'b' } },
    })).json();
    const refused = unauth.error !== undefined || unauth.result?.isError === true;
    expect(refused, 'unauthenticated mail.send is refused (gated)').toBe(true);
  });
});
