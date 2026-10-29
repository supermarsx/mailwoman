import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AnalyticsTag, cspPreview } from './AnalyticsTag.tsx';
import { renderInShell, holding } from './shell/testkit.tsx';
import type { AnalyticsTagApi, AnalyticsTagConfig, AnalyticsTagView } from '../../api/admin/analyticsTag.ts';

const config: AnalyticsTagConfig = { enabled: false, provider: 'plausible', siteId: '', host: '', region: '', customKind: 'url', scriptUrl: '', snippet: '', scriptOrigins: [], connectOrigins: [], imgOrigins: [] };
const view: AnalyticsTagView = { config, preview: { valid: false, error: 'incomplete' }, maxSnippetBytes: 32768 };
function api(): AnalyticsTagApi {
  return {
    read: vi.fn(async () => ({ data: view, etag: '"one"' })),
    save: vi.fn(async next => ({ data: { ...view, config: (({ acknowledgeFullAccess: _a, ...rest }) => rest)(next) }, etag: '"two"' })),
    test: vi.fn(async () => ({ path: '/assets/analytics/abc.js', sha256: 'abc', bytes: 512, served: false, fetches: [{ url: 'https://plausible.io/js/script.js', ok: true, bytes: 1200, contentType: 'application/javascript' }] })),
  };
}
const cspText = () => screen.getByTestId('analytics-tag-csp').textContent ?? '';

describe('browser analytics tag', () => {
  it('previews preset CSP additions exactly', () => {
    expect(cspPreview({ ...config, provider: 'plausible' })).toEqual({ scriptSrc: ['https://plausible.io'], connectSrc: ['https://plausible.io'], imgSrc: [] });
    expect(cspPreview({ ...config, provider: 'matomo', host: 'https://m.example.org/matomo/' })).toEqual({ scriptSrc: ['https://m.example.org'], connectSrc: ['https://m.example.org'], imgSrc: ['https://m.example.org'] });
    expect(cspPreview({ ...config, provider: 'matomo' })).toBeNull();
    expect(cspPreview({ ...config, provider: 'posthog', region: 'eu' })?.scriptSrc).toEqual(['https://eu-assets.i.posthog.com', 'https://eu.i.posthog.com']);
    expect(cspPreview({ ...config, provider: 'custom', scriptOrigins: ['https://CDN.example.com/', 'https://cdn.example.com'] })?.scriptSrc).toEqual(['https://cdn.example.com']);
  });

  it('configures a Plausible preset with a live CSP preview, tests it and saves with the revision', async () => {
    const transport = api();
    renderInShell(() => <AnalyticsTag api={transport} />);
    const provider = await screen.findByLabelText('Tag provider');
    await waitFor(() => expect(provider).toBeEnabled());
    expect(cspText()).toContain('script-src + https://plausible.io');
    expect(cspText()).toContain('img-src no additions');
    fireEvent.input(screen.getByLabelText(/^Site domain/), { target: { value: 'mail.example.com' } });
    fireEvent.input(screen.getByLabelText(/^Provider address/), { target: { value: 'https://stats.example.org' } });
    expect(cspText()).toContain('connect-src + https://stats.example.org');
    expect(screen.getByRole('button', { name: /Test/ })).toHaveAttribute('data-tooltip', "Check that the tag's script can be fetched, without saving");
    fireEvent.click(screen.getByRole('button', { name: /Test/ }));
    const results = await screen.findByRole('list', { name: 'Test results' });
    expect(within(results).getByText(/Reachable: https:\/\/plausible.io\/js\/script.js/)).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('Load this tag for consenting users'));
    // Presets run provider code with full page access too (t63 F3).
    expect(screen.getByRole('button', { name: 'Save browser tag' })).toBeDisabled();
    fireEvent.click(screen.getByLabelText("I understand this provider's script runs with full access to users' webmail pages."));
    fireEvent.click(screen.getByRole('button', { name: 'Save browser tag' }));
    await waitFor(() => expect(transport.save).toHaveBeenCalledWith({ ...config, enabled: true, siteId: 'mail.example.com', host: 'https://stats.example.org', acknowledgeFullAccess: true }, '"one"'));
    expect(await screen.findByRole('status')).toHaveTextContent('Browser tag saved.');
  });

  it('gates a custom tag behind the full-access warning and acknowledgement', async () => {
    const transport = api();
    renderInShell(() => <AnalyticsTag api={transport} />);
    const provider = await screen.findByLabelText('Tag provider');
    await waitFor(() => expect(provider).toBeEnabled());
    fireEvent.change(provider, { target: { value: 'custom' } });
    expect(screen.getByRole('note')).toHaveTextContent(/admin API of anyone signed in to the console/);
    fireEvent.click(screen.getByLabelText('JavaScript snippet', { selector: 'input' }));
    fireEvent.input(screen.getByLabelText(/^JavaScript snippet/, { selector: 'textarea' }), { target: { value: 'window.t=1;' } });
    expect(screen.getByText(/11 of 32,768 bytes/)).toBeInTheDocument();
    const origins = screen.getByLabelText(/^Connection origins/);
    fireEvent.input(origins, { target: { value: 'https://collect.example.com\n\nhttps://b.example.com\n' } });
    expect(origins).toHaveValue('https://collect.example.com\n\nhttps://b.example.com\n');
    expect(cspText()).toContain('connect-src + https://collect.example.com https://b.example.com');
    fireEvent.click(screen.getByLabelText('Load this tag for consenting users'));
    const save = screen.getByRole('button', { name: 'Save browser tag' });
    expect(save).toBeDisabled();
    fireEvent.click(screen.getByLabelText("I understand this provider's script runs with full access to users' webmail pages."));
    expect(save).toBeEnabled();
    fireEvent.click(save);
    await waitFor(() => expect(transport.save).toHaveBeenCalledWith(expect.objectContaining({ provider: 'custom', customKind: 'inline', snippet: 'window.t=1;', connectOrigins: ['https://collect.example.com', 'https://b.example.com'], acknowledgeFullAccess: true }), '"one"'));
  });

  it('is read-only without integrations.write', async () => {
    renderInShell(() => <AnalyticsTag api={api()} />, { loadMe: holding(['integrations.read']) });
    const enabled = await screen.findByLabelText('Load this tag for consenting users');
    expect(enabled).toBeDisabled();
    expect(screen.getByRole('button', { name: /Test/ })).toBeDisabled();
  });
});
