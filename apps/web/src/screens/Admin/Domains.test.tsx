import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { Domains } from './Domains.tsx';
import { renderInShell } from './shell/testkit.tsx';
import { expectNoAxeViolations } from './shell/uxTestkit.ts';
import { AdminProblemError } from './shell/adminApi.ts';
import type { DomainsApi, DomainDetail, DomainChange } from '../../api/admin/domains.ts';

const domain: DomainDetail = { name: 'example.com', enabled: true, preferred: false, upstream: { imap: { host: 'mail.example.com', port: 993, tls: 'implicit', auth: 'password' } }, settings: { discovery: 'off', retainedSetting: true }, allowlist: [], blocklist: [], updatedAt: 1, mode: 'engine', aliases: [], hosts: [], etag: '"v1"', mappingEtag: '"m1"', preferenceEtag: '"p1"' };
function service(overrides: Partial<DomainsApi> = {}): DomainsApi {
  return {
    status: vi.fn(async () => ({ mode: 'engine' as const, blockedAccounts: { total: 0, byDomain: [] }, upstreamChoices: [] })),
    probe: vi.fn(async () => ({ domain: domain.name, results: [], ranAt: '2026-10-25T16:00:00Z', credentialsSent: false as const })), dns: vi.fn(async () => ({ domain: domain.name, mx: { records: [], error: null }, spf: { present: false, records: [], error: null }, dmarc: { present: false, records: [], error: null }, srv: [] })),
    list: vi.fn(async () => [domain]), detail: vi.fn(async () => structuredClone(domain)),
    capabilities: vi.fn(async () => ['domains.read', 'domains.write', 'domains.delete', 'domains.upstream_write', 'domains.signin_policy', 'diagnostics.run']),
    save: vi.fn(async (_name: string, changes: DomainChange) => changes.dryRun ? { dryRun: true, before: domain, after: { ...domain, ...changes } } : domain),
    remove: vi.fn(async () => {}), enable: vi.fn(async () => {}), prefer: vi.fn(async () => {}), clearPreferred: vi.fn(async () => {}), mapping: vi.fn(async () => {}), renameAlias: vi.fn(async () => {}),
    detect: vi.fn(async () => ({ name: domain.name, upstream: { imap: { host: 'detected.example.com', port: 993, tls: 'implicit' as const, auth: 'password' as const } }, source: 'ISPDB', requiresOAuth: false, saved: false as const })),
    ...overrides,
  };
}
function renderDomains(api: DomainsApi) { const [route, setRoute] = createSignal({ section: 'domains', rest: [] as string[] }); return renderInShell(() => <Domains service={api} />, { route, navigate: path => setRoute({ section: 'domains', rest: (path.split('#/')[1] ?? path).split('/').filter(Boolean).slice(1).map(decodeURIComponent) }) }); }
async function page(name: string) { fireEvent.click(screen.getByRole('link', { name })); await waitFor(() => expect(screen.getByRole('link', { name })).toHaveAttribute('aria-current', 'page')); }
async function open(api: DomainsApi) { const result = renderDomains(api); fireEvent.click(await screen.findByRole('button', { name: domain.name })); await screen.findByRole('link', { name: 'Connections and discovery' }); await page('Connections and discovery'); await screen.findByRole('textbox', { name: 'IMAP Host' }); await waitFor(() => expect(screen.getByRole('button', { name: 'Reload server version' })).toBeEnabled()); return result; }

describe('Admin Domains editor', () => {
  it('edits an alias on its own page using the atomic rename API', async () => {
    const api=service({detail:vi.fn(async()=>({...domain,aliases:['old.example.net']}))});
    await open(api);await page('Domain aliases');
    fireEvent.click(screen.getByRole('button',{name:'Edit alias'}));
    expect(screen.queryByRole('button',{name:'Remove old.example.net'})).toBeNull();
    const input=screen.getByRole('textbox',{name:'Alias domain'});
    fireEvent.input(input,{target:{value:'new.example.net'}});fireEvent.submit(input.closest('form')!);
    await waitFor(()=>expect(api.renameAlias).toHaveBeenCalledWith(domain.name,'old.example.net','new.example.net','"m1"'));
    expect(api.mapping).not.toHaveBeenCalled();
  });
  it('keeps compact alias actions inside the named row and opens the dedicated clone editor',async()=>{
    const api=service({detail:vi.fn(async()=>({...domain,aliases:['first.example.net','second.example.net']}))});
    await open(api);await page('Domain aliases');
    const table=screen.getByRole('table',{name:'Domain aliases'});
    const row=within(table).getByRole('rowheader',{name:'second.example.net'}).closest('tr')!;
    expect(within(row).getByRole('button',{name:'Edit alias'})).toBeEnabled();
    expect(within(row).getByRole('button',{name:'Remove second.example.net'})).toBeEnabled();
    fireEvent.click(within(row).getByRole('button',{name:'Clone alias'}));
    expect(screen.getByRole('textbox',{name:'Alias domain'})).toHaveValue('second.example.net');
    expect(screen.queryByRole('table',{name:'Domain aliases'})).toBeNull();
    expect(api.mapping).not.toHaveBeenCalled();
  });
  it('keeps alias names visible without management actions for read-only administrators',async()=>{
    const api=service({capabilities:vi.fn(async()=>['domains.read']),detail:vi.fn(async()=>({...domain,aliases:['readonly.example.net']}))});
    await open(api);await page('Domain aliases');
    const table=screen.getByRole('table',{name:'Domain aliases'});
    expect(within(table).getByRole('rowheader',{name:'readonly.example.net'})).toBeVisible();
    expect(within(table).queryByRole('button')).toBeNull();
    expect(screen.queryByRole('button',{name:'Add alias'})).toBeNull();
  });
  it('shows aliases in a searchable domain table and keeps actions within their row', async () => {
    const api = service({ list: vi.fn(async () => [{ ...domain, aliases: ['brand.example.net'] }]) });
    renderDomains(api);
    const table = await screen.findByRole('table', { name: 'Domains' });
    expect(within(table).getByRole('columnheader', { name: 'Domain aliases' })).toBeVisible();
    expect(within(table).getByText('brand.example.net')).toBeVisible();
    const row = within(table).getByRole('button', { name: domain.name }).closest('tr')!;
    expect(within(row).getByRole('link', { name: 'Edit' })).toBeVisible();
    const search = screen.getByRole('searchbox', { name: 'Find a domain' });
    fireEvent.input(search, { target: { value: 'BRAND.EXAMPLE' } });
    await waitFor(() => expect(within(table).getByRole('button', { name: domain.name })).toBeVisible());
    fireEvent.input(search, { target: { value: 'missing.test' } });
    await screen.findByText('No domains match this search.');
    expect(screen.queryByRole('button', { name: domain.name })).toBeNull();
  });
  it('previews a structured upstream change and applies it with the read ETag', async () => {
    const api = service(); await open(api);
    fireEvent.input(screen.getByRole('textbox', { name: 'IMAP Host' }), { target: { value: 'new.example.com' } });
    fireEvent.submit(screen.getByRole('form', { name: 'Configure example.com' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(api.save).toHaveBeenCalledTimes(1);
    expect(api.save).toHaveBeenCalledWith(domain.name, { upstream: { imap: { ...domain.upstream.imap, host: 'new.example.com' } }, dryRun: true }, '"v1"');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Apply changes' }));
    await waitFor(() => expect(api.save).toHaveBeenCalledTimes(2));
    expect(api.save).toHaveBeenLastCalledWith(domain.name, { upstream: { imap: { ...domain.upstream.imap, host: 'new.example.com' } } }, '"v1"');
  });
  it('preserves a draft on stale preview failure and offers reload', async () => {
    const api = service({ save: vi.fn(async () => { throw new AdminProblemError(412, { code: 'stale' }, '"v2"'); }) }); await open(api);
    fireEvent.input(screen.getByRole('textbox', { name: 'IMAP Host' }), { target: { value: 'draft.example.com' } });
    fireEvent.submit(screen.getByRole('form', { name: 'Configure example.com' }));
    await waitFor(() => expect(api.save).toHaveBeenCalledOnce());
    expect(screen.getByRole('textbox', { name: 'IMAP Host' })).toHaveValue('draft.example.com');
    await waitFor(() => expect(screen.getByRole('button', { name: 'Reload server version' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Reload server version' }));
    const dialog = await screen.findByRole('alertdialog'); fireEvent.click(within(dialog).getByRole('button', { name: 'Continue' }));
    await waitFor(() => expect(screen.getByRole('textbox', { name: 'IMAP Host' })).toHaveValue('mail.example.com'));
  });
  it('only sends policy changes and retains unknown settings', async () => {
    const api = service(); await open(api);
    await page('Sign-in policy');
    fireEvent.input(screen.getByRole('textbox', { name: 'Allowlist' }), { target: { value: ' alice@example.com\nalice@example.com\n' } });
    fireEvent.submit(screen.getByRole('form', { name: 'Configure example.com' })); await screen.findByRole('alertdialog');
    expect(api.save).toHaveBeenCalledWith(domain.name, { allowlist: ['alice@example.com'], dryRun: true }, '"v1"');
  });
  it('renders proxy mode with JMAP fields and no engine protocol switch', async () => {
    const api = service({ detail: vi.fn(async () => ({ ...domain, mode: 'proxy' as const, upstream: { jmap: { sessionUrl: 'https://mail.example.com/jmap' } } })) });
    renderDomains(api); fireEvent.click(await screen.findByRole('button', { name: domain.name }));
    await screen.findByRole('link', { name: 'Connections and discovery' }); await page('Connections and discovery');
    expect(await screen.findByRole('textbox', { name: 'JMAP session URL' })).toHaveValue('https://mail.example.com/jmap');
    expect(screen.queryByRole('combobox', { name: 'Incoming mail protocol' })).toBeNull();
  });
  it('keeps read-only domain configuration accessible without mutation controls', async () => {
    const api = service({ capabilities: vi.fn(async () => ['domains.read']) }); const { container } = await open(api);
    expect(screen.getByRole('textbox', { name: 'IMAP Host' })).toBeDisabled();
    await page('Sign-in policy');
    expect(screen.getByRole('textbox', { name: 'Allowlist' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Review changes' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Delete' })).toBeNull();
    await expectNoAxeViolations(container);
  });
  it('does not save discovery proposals and refuses OAuth-only proposals', async () => {
    const api = service(); await open(api); fireEvent.click(screen.getByRole('button', { name: 'Detect connection settings' }));
    const apply = await screen.findByRole('button', { name: 'Use proposal in editor' }); expect(api.save).not.toHaveBeenCalled();
    fireEvent.click(apply); expect(screen.getByRole('textbox', { name: 'IMAP Host' })).toHaveValue('detected.example.com'); expect(api.save).not.toHaveBeenCalled();
    vi.mocked(api.detect).mockResolvedValueOnce({ name: domain.name, upstream: {}, source: 'ISPDB', requiresOAuth: true, saved: false });
    await waitFor(() => expect(screen.getByRole('button', { name: 'Detect connection settings' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Detect connection settings' }));
    expect(await screen.findByRole('button', { name: 'Use proposal in editor' })).toBeDisabled();
  });
  it('requires typed confirmation before deleting and defaults to protected deletion', async () => {
    const api = service(); await open(api); await page('Overview'); fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    const dialog = await screen.findByRole('alertdialog'); const button = within(dialog).getByRole('button', { name: 'Delete' });
    expect(button).toBeDisabled(); expect(api.remove).not.toHaveBeenCalled();
    fireEvent.input(within(dialog).getByRole('textbox'), { target: { value: domain.name } }); fireEvent.click(button);
    await waitFor(() => expect(api.remove).toHaveBeenCalledWith(domain.name, '"v1"', false));
  });
  it('creates a named domain before loading its server-owned mode', async () => {
    const api = service(); renderDomains(api);
    fireEvent.click(await screen.findByRole('button', { name: 'Quick add' }));
    const form = await screen.findByRole('form', { name: 'Add domain' });
    fireEvent.input(within(form).getByRole('textbox'), { target: { value: ' new.example ' } });
    fireEvent.submit(form);
    await waitFor(() => expect(api.save).toHaveBeenCalledWith('new.example', {}, null));
    await waitFor(() => expect(api.detail).toHaveBeenCalledWith('new.example'));
  });
  it('shows a plaintext warning only for explicit unencrypted transport', async () => {
    await open(service());
    expect(screen.queryByText(/without transport encryption/)).toBeNull();
    fireEvent.change(screen.getByRole('combobox', { name: 'IMAP Connection security' }), { target: { value: 'none' } });
    expect(screen.getByText(/without transport encryption/)).toBeInTheDocument();
    expect(screen.getByRole('checkbox', { name: 'Allow private network addresses for IMAP' })).not.toBeChecked();
  });
  it('keeps draft version fencing when a mapping refresh observes a concurrent edit', async () => {
    const api = service(); await open(api);
    fireEvent.input(screen.getByRole('textbox', { name: 'IMAP Host' }), { target: { value: 'draft.example.com' } });
    vi.mocked(api.detail).mockResolvedValueOnce({ ...domain, aliases: ['alias.example.com'], etag: '"v2"' });
    await page('Domain aliases');
    fireEvent.click(screen.getByRole('button',{name:'Add alias'}));
    fireEvent.input(screen.getByRole('textbox', { name: 'Alias domain' }), { target: { value: 'alias.example.com' } });
    fireEvent.submit(screen.getByRole('textbox', { name: 'Alias domain' }).closest('form')!);
    await waitFor(() => expect(api.mapping).toHaveBeenCalledWith(domain.name, 'aliases', 'alias.example.com', false, '"m1"'));
    await screen.findByText('alias.example.com');
    await page('Connections and discovery');
    await waitFor(() => expect(screen.getByRole('button', { name: 'Review changes' })).toBeEnabled());
    fireEvent.submit(screen.getByRole('form', { name: 'Configure example.com' }));
    await screen.findByRole('alertdialog');
    expect(api.save).toHaveBeenCalledWith(domain.name, expect.objectContaining({ dryRun: true }), '"v1"');
    expect(screen.getByRole('textbox', { name: 'IMAP Host' })).toHaveValue('draft.example.com');
  });

  it('uses the global preference version for selecting and clearing the default', async () => {
    const api = service(); await open(api);
    vi.mocked(api.detail).mockResolvedValue({ ...domain, preferred: true, preferenceEtag: '"p2"' });
    await page('Overview');
    fireEvent.click(screen.getByRole('button', { name: 'Set preferred domain' }));
    await waitFor(() => expect(api.prefer).toHaveBeenCalledWith(domain.name, '"p1"'));
    const clear = await screen.findByRole('button', { name: 'Clear preferred domain' });
    await waitFor(() => expect(clear).toBeEnabled());
    fireEvent.click(clear);
    await waitFor(() => expect(api.clearPreferred).toHaveBeenCalledWith('"p2"'));
  });
  it('refreshes only the mapping version after a successful mapping update', async () => {
    const api = service(); await open(api);
    vi.mocked(api.detail).mockResolvedValue({ ...domain, aliases: ['alias.example.com'], mappingEtag: '"m2"' });
    await page('Domain aliases');
    fireEvent.click(screen.getByRole('button',{name:'Add alias'}));
    fireEvent.input(screen.getByRole('textbox', { name: 'Alias domain' }), { target: { value: 'alias.example.com' } });
    fireEvent.submit(screen.getByRole('textbox', { name: 'Alias domain' }).closest('form')!);
    await screen.findByText('alias.example.com');
    await waitFor(() => expect(screen.getByRole('button', { name: 'Remove alias.example.com' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Remove alias.example.com' }));
    await waitFor(() => expect(api.mapping).toHaveBeenLastCalledWith(domain.name, 'aliases', 'alias.example.com', true, '"m2"'));
  });

  it('probes only saved settings and displays real failures without invented TLS evidence', async () => {
    const api = service({ probe: vi.fn(async () => ({ domain: domain.name, results: [{ protocol: 'imap', host: 'mail.example.com', port: 993, tls: 'implicit', reachable: false, latencyMs: 12, tlsVersion: null, certNotAfter: null, certDaysLeft: null, greeting: null, capabilities: [], error: 'Certificate is not trusted' }], ranAt: '2026-10-25T16:00:00Z', credentialsSent: false as const })) });
    await open(api); await page('Diagnostics'); fireEvent.click(screen.getByRole('button', { name: 'Test saved connections' }));
    expect(await screen.findByText('Certificate is not trusted')).toBeInTheDocument();
    expect(screen.getByText('Connection check failed · 12 ms')).toBeInTheDocument();
    expect(screen.queryByText(/Certificate expires/)).toBeNull();
    expect(api.probe).toHaveBeenCalledWith(domain.name); expect(api.save).not.toHaveBeenCalled();
    await page('Connections and discovery');
    fireEvent.input(screen.getByRole('textbox', { name: 'IMAP Host' }), { target: { value: 'unsaved.example.com' } });
    await page('Diagnostics');
    expect(screen.getByRole('button', { name: 'Test saved connections' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Check DNS' })).toBeDisabled();
  });
  it('distinguishes missing DNS records from resolver failures', async () => {
    const api = service({ dns: vi.fn(async () => ({ domain: domain.name, mx: { records: [{ preference: 10, host: 'mx.example.com.' }], error: null }, spf: { present: true, records: ['v=spf1 -all'], error: null }, dmarc: { present: false, records: [], error: 'DNS lookup failed or timed out' }, srv: [{ service: '_imaps._tcp', records: [], error: null }] })) });
    await open(api); await page('Diagnostics'); fireEvent.click(screen.getByRole('button', { name: 'Check DNS' }));
    expect(await screen.findByText('v=spf1 -all')).toBeInTheDocument();
    expect(screen.getByText('10 · mx.example.com.')).toBeInTheDocument();
    expect(screen.getByText('DNS lookup failed or timed out')).toBeInTheDocument();
    expect(screen.getByText('No matching DNS records were found.')).toBeInTheDocument();
  });

  it('links blocked accounts to domain configuration and shows the imported host', async () => {
    const api = service({ status: vi.fn(async () => ({ mode: 'engine' as const, blockedAccounts: { total: 2, byDomain: [{ domain: 'missing.test', accounts: 2, seenHost: 'old.example.com' }] }, upstreamChoices: [] })) });
    renderDomains(api);
    const link = await screen.findByRole('link', { name: 'Configure missing.test' });
    expect(link.getAttribute('href')).toContain('#/domains/missing.test');
    expect(screen.getByText('Previously used server: old.example.com')).toBeInTheDocument();
  });
  it('offers migrated proxy origins as unsaved proposals and tolerates null protocol entries', async () => {
    const api = service({ status: vi.fn(async () => ({ mode: 'proxy' as const, blockedAccounts: { total: 0, byDomain: [] }, upstreamChoices: ['https://legacy.example.com'] })), detail: vi.fn(async () => ({ ...domain, mode: 'proxy' as const, upstream: { imap: null, jmap: { sessionUrl: 'https://current.example.com/jmap' } } })) });
    vi.mocked(api.save).mockImplementation(async (_name, changes) => ({ dryRun: true, before: { ...domain, upstream: { imap: null, jmap: { sessionUrl: 'https://current.example.com/jmap' } } }, after: { ...domain, ...changes } }));
    renderDomains(api); fireEvent.click(await screen.findByRole('button', { name: domain.name }));
    await screen.findByRole('link', { name: 'Connections and discovery' }); await page('Connections and discovery');
    await screen.findByRole('combobox', { name: 'Previously configured JMAP origins' });
    await waitFor(() => expect(screen.getByRole('button', { name: 'Reload server version' })).toBeEnabled());
    fireEvent.change(screen.getByRole('combobox', { name: 'Previously configured JMAP origins' }), { target: { value: 'https://legacy.example.com' } });
    expect(screen.getByRole('textbox', { name: 'JMAP session URL' })).toHaveValue('https://legacy.example.com/.well-known/jmap');
    expect(api.save).not.toHaveBeenCalled();
    fireEvent.submit(screen.getByRole('form', { name: 'Configure example.com' }));
    const dialog = await screen.findByRole('alertdialog'); expect(within(dialog).getByText(/current.example.com/)).toBeInTheDocument();
  });

  it('keeps list actions on the list and confirms deletion against a fresh revision', async () => {
    const api = service(); renderDomains(api);
    await screen.findByRole('button', { name: domain.name });
    expect(screen.queryByRole('textbox')).toBeNull();
    expect(screen.getByRole('link', { name: 'Edit' })).toHaveAttribute('href', expect.stringContaining('/connections'));
    expect(screen.getByRole('link', { name: 'Clone' })).toHaveAttribute('href', expect.stringContaining('/clone'));
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(api.detail).toHaveBeenCalledWith(domain.name);
    expect(within(dialog).getByRole('button', { name: 'Delete' })).toBeDisabled();
    fireEvent.input(within(dialog).getByRole('textbox'), { target: { value: domain.name } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    await waitFor(() => expect(api.remove).toHaveBeenCalledWith(domain.name, '"v1"', false));
    expect(screen.queryByRole('form')).toBeNull();
  });
  it('opens cloning on its own page and copies configuration into a disabled domain', async () => {
    const api = service(); renderDomains(api);
    fireEvent.click(await screen.findByRole('link', { name: 'Clone' }));
    const form = await screen.findByRole('form', { name: 'Clone' });
    expect(screen.queryByRole('button', { name: domain.name })).toBeNull();
    expect(api.save).not.toHaveBeenCalled();
    fireEvent.input(within(form).getByRole('textbox'), { target: { value: 'copy.example.com' } });
    fireEvent.submit(form);
    await waitFor(() => expect(api.save).toHaveBeenCalledWith('copy.example.com', { enabled: false, upstream: domain.upstream, settings: domain.settings, allowlist: [], blocklist: [] }, null));
  });
  it('opens policy separately from connection editors and keeps overview read-only', async () => {
    const api = service(); renderDomains(api);
    fireEvent.click(await screen.findByRole('button', { name: domain.name }));
    await screen.findByRole('link', { name: 'Sign-in policy' });
    expect(screen.queryByRole('textbox')).toBeNull();
    await page('Sign-in policy');
    expect(screen.getByRole('textbox', { name: 'Allowlist' })).toBeVisible();
    expect(screen.queryByRole('textbox', { name: 'IMAP Host' })).toBeNull();
    expect(screen.queryByRole('textbox', { name: 'Alias domain' })).toBeNull();
  });

});
