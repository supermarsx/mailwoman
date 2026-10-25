import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { AdminSso, worstCert } from './index.tsx';
import { ssoInput, ssoValues, ownAddress } from './SsoEditor.tsx';
import { certInfo, certState, parseIdpMetadata, splitPem } from './saml.ts';
import type { SsoAdminApi, SsoBackendRow } from '../../../modules/sso';
import { sectionPath, type AdminRoute } from '../shell/router.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { expectNoAxeViolations, expectNoInlineStyle } from '../shell/uxTestkit.ts';
import { stripIsolates } from '../../../i18n';

// openssl req -x509 -newkey rsa:1024 -subj "/CN=idp.example.org/O=Example IdP" -days 3650 (2026-10-25 → 2036-10-22).
const CERT_B64 =
  'MIICKTCCAZKgAwIBAgIBBzANBgkqhkiG9w0BAQsFADAwMRgwFgYDVQQDDA9pZHAuZXhhbXBsZS5vcmcxFDASBgNVBAoMC0V4YW1wbGUgSWRQMB4XDTI2MTAyNTEzMjk1' +
  'MFoXDTM2MTAyMjEzMjk1MFowMDEYMBYGA1UEAwwPaWRwLmV4YW1wbGUub3JnMRQwEgYDVQQKDAtFeGFtcGxlIElkUDCBnzANBgkqhkiG9w0BAQEFAAOBjQAwgYkCgYEA' +
  '0gamFB7YTPqqx+5LRsrNbtpdH347lFvavyrH5TSJ3y2eRalfPhxASM5Ea2enyxCempztHl8UC5kRFne2t0sEYCHLPhZ64tFxWejG4wujd5gQEncs81LMxfxE7Td7Umwx' +
  'EXPhJRY4of/TLWqAySopc+EAEdXaadmKQ8rdZTpJogcCAwEAAaNTMFEwHQYDVR0OBBYEFB6SVGpXZUjgxOotc1EGEkmb9OMkMB8GA1UdIwQYMBaAFB6SVGpXZUjgxOot' +
  'c1EGEkmb9OMkMA8GA1UdEwEB/wQFMAMBAf8wDQYJKoZIhvcNAQELBQADgYEAbBezH7e837tmwdvhLerXvid/2DSTYQb27exudHZp346ss4GFTbvI/VfpGdM9fkZRYNko' +
  'CLQHxzHql4nYlXR3cOh+Ew5NAdMGlRCErXJGbklkhFnVEMvkRkGDZXWJXQ6kRfrA8Odu9A8yHMm3GIRHCiM0mmwWv1Xssc8jG5LYy3s=';
const CERT = `-----BEGIN CERTIFICATE-----\n${CERT_B64.match(/.{1,64}/g)!.join('\n')}\n-----END CERTIFICATE-----`;
const NOW = new Date('2026-11-01T00:00:00Z');

const METADATA = `<?xml version="1.0"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="http://www.w3.org/2000/09/xmldsig#" entityID="https://idp.example.org/saml">
  <md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:KeyDescriptor use="signing"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>${CERT_B64}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>
    <md:KeyDescriptor use="encryption"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>AAAA</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>
    <md:SingleLogoutService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" Location="https://idp.example.org/saml/slo"/>
    <md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="https://idp.example.org/saml/post"/>
    <md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" Location="https://idp.example.org/saml/sso"/>
  </md:IDPSSODescriptor>
</md:EntityDescriptor>`;

const OIDC_ROW: SsoBackendRow = {
  id: 'corp-oidc',
  displayName: 'Acme SSO',
  scope: 'deployment',
  enabled: true,
  config: {
    kind: 'oidc',
    issuerUrl: 'https://idp.example.org',
    clientId: 'client',
    redirectUrl: 'https://mail.example.org/api/sso/corp-oidc/callback',
    scopes: ['openid', 'email', 'profile'],
    firstLoginPolicy: 'allowlist',
  },
  claimMap: { email: 'email', username: 'preferred_username', display: 'name', groups: 'groups' },
  hasSecret: true,
  updatedAt: '2026-10-20T08:00:00Z',
};

const SAML_ROW: SsoBackendRow = {
  id: 'corp-saml',
  displayName: 'Contoso SAML',
  scope: 'domain:example.org',
  enabled: false,
  config: {
    kind: 'saml',
    spEntityId: 'https://mail.example.org/sp',
    acsUrl: 'https://mail.example.org/api/sso/corp-saml/acs',
    idpMetadataUrl: null,
    idpMetadataXml: null,
    idpSsoUrl: 'https://idp.example.org/saml/sso',
    idpSloUrl: null,
    idpSigningCertsPem: [CERT],
    wantAssertionsSigned: true,
    wantEncrypted: false,
    nameidFormat: 'urn:oasis:names:tc:SAML:2.0:nameid-format:persistent',
    firstLoginPolicy: 'allowlist',
  },
  claimMap: { email: 'email', username: 'uid', display: 'displayName', groups: null },
};

function setup(rows: SsoBackendRow[] = [OIDC_ROW, SAML_ROW], opts: { rest?: string[]; domains?: string[]; now?: Date } = {}) {
  let stored = [...rows];
  const api: SsoAdminApi = {
    list: vi.fn(async (scope?: string) => stored.filter((r) => r.scope === (scope ?? 'deployment'))),
    save: vi.fn(async (input) => {
      const { secret: _secret, ...row } = input;
      stored = [...stored.filter((r) => r.id !== input.id), row];
    }),
    remove: vi.fn(async (id: string) => {
      stored = stored.filter((r) => r.id !== id);
    }),
  };
  const [route, setRoute] = createSignal<AdminRoute>({ section: 'sso', rest: opts.rest ?? [] });
  const navigate = vi.fn((path: string) => {
    const rest = (path.split('#/sso')[1] ?? '').split('/').filter(Boolean);
    setRoute({ section: 'sso', rest });
  });
  const utils = renderInShell(() => (
    <AdminSso api={api} listDomains={async () => opts.domains ?? ['example.org']} route={route()} navigate={navigate} now={opts.now ?? NOW} />
  ));
  return { ...utils, api, navigate, setRoute };
}

describe('Admin › SSO list', () => {
  it('lists the providers of every scope with protocol, scope, state and certificate state', async () => {
    const { api } = setup();
    const table = await screen.findByRole('table', { name: 'Single sign-on' });
    expect(table.querySelector('[data-row-key="corp-oidc"]')).not.toBeNull();
    expect(table.querySelector('[data-row-key="corp-saml"]')).not.toBeNull();
    expect(api.list).toHaveBeenCalledWith('deployment');
    expect(api.list).toHaveBeenCalledWith('domain:example.org');
    const saml = table.querySelector('[data-row-key="corp-saml"]')!;
    expect(stripIsolates(saml.textContent ?? '')).toMatch(/Domain example\.org/);
    expect(within(saml as HTMLElement).getByText('Disabled')).toBeInTheDocument();
    expect(stripIsolates(saml.textContent ?? '')).toMatch(/Valid until/);
  });

  it('search and protocol chips filter the list', async () => {
    setup();
    const table = await screen.findByRole('table', { name: 'Single sign-on' });
    await waitFor(() => expect(table.querySelectorAll('tbody tr')).toHaveLength(2));
    fireEvent.click(screen.getByRole('button', { name: 'SAML 2.0', pressed: false }));
    await waitFor(() => expect([...table.querySelectorAll('tbody tr')].map((r) => r.getAttribute('data-row-key'))).toEqual(['corp-saml']));
    fireEvent.click(screen.getByRole('button', { name: 'SAML 2.0', pressed: true }));
    fireEvent.input(screen.getByRole('searchbox', { name: 'Search providers' }), { target: { value: 'acme' } });
    await waitFor(() => expect([...screen.getByRole('table').querySelectorAll('tbody tr')].map((r) => r.getAttribute('data-row-key'))).toEqual(['corp-oidc']));
  });

  it('the empty state offers Add', async () => {
    const { navigate } = setup([]);
    expect(await screen.findByText('No SSO backends configured.')).toBeInTheDocument();
    fireEvent.click(within(screen.getByTestId('table-empty')).getByRole('button', { name: 'Add a login backend' }));
    expect(navigate).toHaveBeenCalledWith(sectionPath('sso', ['new']));
  });

  it('turning a provider off asks first, naming it; Cancel sends nothing', async () => {
    const { api } = setup();
    fireEvent.click(await screen.findByRole('button', { name: 'Disable Acme SSO' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(stripIsolates(dialog.textContent ?? '')).toMatch(/Turn off Acme SSO\?/);
    fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    expect(api.save).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Disable Acme SSO' }));
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Disable' }));
    await waitFor(() => expect(api.save).toHaveBeenCalledTimes(1));
    expect((api.save as ReturnType<typeof vi.fn>).mock.calls[0]![0]).toEqual({
      id: 'corp-oidc',
      displayName: 'Acme SSO',
      scope: 'deployment',
      enabled: false,
      config: OIDC_ROW.config,
      claimMap: OIDC_ROW.claimMap,
    });
    expect(await screen.findByTestId('sso-notice')).toHaveTextContent(/is off/);
  });

  it('delete asks first and removes', async () => {
    const { api } = setup();
    fireEvent.click(await screen.findByRole('button', { name: 'Delete Contoso SAML' }));
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Delete' }));
    await waitFor(() => expect(api.remove).toHaveBeenCalledWith('corp-saml'));
    await waitFor(() => expect(screen.queryByText('Contoso SAML')).toBeNull());
  });
});

describe('Admin › SSO detail', () => {
  it('opens a provider at #/sso/<id> with its values, history, and a fixed id', async () => {
    setup([OIDC_ROW], { rest: ['corp-oidc'] });
    const detail = await screen.findByTestId('list-detail');
    expect(await within(detail).findByRole('heading', { name: 'Acme SSO' })).toBeInTheDocument();
    const id = await within(detail).findByRole('textbox', { name: 'Backend ID' });
    expect(id).toHaveValue('corp-oidc');
    expect(id).toHaveAttribute('readonly');
    expect(within(detail).getByRole('textbox', { name: 'Issuer URL' })).toHaveValue('https://idp.example.org');
    expect(stripIsolates(within(detail).getByTestId('field-history').textContent ?? '')).toMatch(/Last changed on/);
    // The SAML group is hidden for an OIDC provider.
    expect(within(detail).queryByRole('group', { name: 'Identity provider (SAML)' })).toBeNull();
    expect(within(detail).getByPlaceholderText('A value is stored. Type a new one to replace it.')).toBeInTheDocument();
  });

  it('adds an OIDC provider: validation first, then the body with this server callback filled in, then the new address', async () => {
    const { api, navigate } = setup([], { rest: ['new'] });
    const detail = await screen.findByTestId('list-detail');
    fireEvent.click(within(detail).getByRole('button', { name: 'Add backend' }));
    expect(within(detail).getByRole('button', { name: 'Add backend' })).toBeDisabled();
    fireEvent.input(within(detail).getByRole('textbox', { name: 'Backend ID' }), { target: { value: 'Bad Id' } });
    fireEvent.input(within(detail).getByRole('textbox', { name: 'Display name' }), { target: { value: 'New IdP' } });
    fireEvent.click(within(detail).getByRole('button', { name: 'Add backend' }));
    await waitFor(() => expect(within(detail).getByText(/Use lower-case letters, digits and hyphens/)).toBeInTheDocument());
    expect(within(detail).getByText('Issuer URL is required.')).toBeInTheDocument();
    expect(api.save).not.toHaveBeenCalled();
    fireEvent.input(within(detail).getByRole('textbox', { name: 'Backend ID' }), { target: { value: 'new-oidc' } });
    fireEvent.input(within(detail).getByRole('textbox', { name: 'Issuer URL' }), { target: { value: 'https://idp.test' } });
    fireEvent.input(within(detail).getByRole('textbox', { name: 'Client ID' }), { target: { value: 'mw' } });
    fireEvent.input(within(detail).getByLabelText('Client secret', { selector: 'input' }), { target: { value: 's3cret' } });
    fireEvent.click(within(detail).getByRole('button', { name: 'Add backend' }));
    await waitFor(() => expect(api.save).toHaveBeenCalledTimes(1));
    const body = (api.save as ReturnType<typeof vi.fn>).mock.calls[0]![0];
    expect(body).toMatchObject({
      id: 'new-oidc',
      displayName: 'New IdP',
      scope: 'deployment',
      enabled: true,
      secret: 's3cret',
      config: { kind: 'oidc', issuerUrl: 'https://idp.test', clientId: 'mw', redirectUrl: ownAddress('new-oidc', 'callback'), scopes: ['openid', 'email', 'profile'] },
    });
    await waitFor(() => expect(navigate).toHaveBeenCalledWith(sectionPath('sso', ['new-oidc'])));
  });

  it('a new provider may not reuse an id', async () => {
    const { api } = setup([OIDC_ROW], { rest: ['new'] });
    const detail = await screen.findByTestId('list-detail');
    await waitFor(() => expect(screen.getByRole('table').querySelector('[data-row-key="corp-oidc"]')).not.toBeNull());
    fireEvent.input(within(detail).getByRole('textbox', { name: 'Backend ID' }), { target: { value: 'corp-oidc' } });
    fireEvent.click(within(detail).getByRole('button', { name: 'Add backend' }));
    await waitFor(() => expect(within(detail).getByText(/Another provider already uses the id/)).toBeInTheDocument());
    expect(api.save).not.toHaveBeenCalled();
  });

  it('SAML: metadata upload shows a preview and fills the sign-in address, sign-out address and certificates', async () => {
    const { api } = setup([SAML_ROW], { rest: ['corp-saml'] });
    const detail = await screen.findByTestId('list-detail');
    const meta = await within(detail).findByTestId('sso-metadata');
    const file = new File([METADATA], 'idp.xml', { type: 'application/xml' });
    fireEvent.change(within(meta).getByLabelText('Metadata file'), { target: { files: [file] } });
    const preview = await within(meta).findByTestId('sso-metadata-preview');
    expect(preview).toHaveTextContent('https://idp.example.org/saml');
    expect(preview).toHaveTextContent('https://idp.example.org/saml/sso');
    expect(preview).toHaveTextContent('https://idp.example.org/saml/slo');
    expect(stripIsolates(preview.textContent ?? '')).toMatch(/idp\.example\.org.*Valid until/);
    fireEvent.click(within(preview).getByRole('button', { name: 'Use these values' }));
    expect(within(detail).getByRole('textbox', { name: 'IdP logout (SLO) URL' })).toHaveValue('https://idp.example.org/saml/slo');
    expect(within(detail).getAllByTestId('sso-cert')).toHaveLength(1);
    fireEvent.click(within(detail).getByRole('button', { name: 'Save changes' }));
    await waitFor(() => expect(api.save).toHaveBeenCalledTimes(1));
    const body = (api.save as ReturnType<typeof vi.fn>).mock.calls[0]![0];
    expect(body.config).toMatchObject({ kind: 'saml', idpSloUrl: 'https://idp.example.org/saml/slo', idpMetadataXml: METADATA.trim() });
    expect(body.scope).toBe('domain:example.org');
  });

  it('SAML: a file that is not metadata is refused and fills nothing', async () => {
    setup([SAML_ROW], { rest: ['corp-saml'] });
    const meta = await screen.findByTestId('sso-metadata');
    fireEvent.input(within(meta).getByRole('textbox', { name: 'Or paste the metadata' }), { target: { value: '<html><body/></html>' } });
    expect(await within(meta).findByRole('alert')).toHaveTextContent('no IDPSSODescriptor');
    expect(within(meta).queryByTestId('sso-metadata-preview')).toBeNull();
  });

  it('SAML: the certificate list shows expiry; an expired one says so; removing the last one blocks saving', async () => {
    const { api } = setup([SAML_ROW], { rest: ['corp-saml'], now: new Date('2037-01-01T00:00:00Z') });
    const certs = await screen.findByTestId('sso-certs');
    expect(stripIsolates(certs.textContent ?? '')).toMatch(/idp\.example\.org.*Expired on/);
    fireEvent.click(within(certs).getByRole('button', { name: /Remove the certificate/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));
    expect(await screen.findByText('Add at least one signing certificate.')).toBeInTheDocument();
    expect(api.save).not.toHaveBeenCalled();
  });

  it('has no axe violations and no inline style (list + SAML detail)', async () => {
    const { container } = setup([OIDC_ROW, SAML_ROW], { rest: ['corp-saml'] });
    await screen.findByTestId('sso-certs');
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });
});

describe('SAML helpers', () => {
  it('reads subject and validity from a certificate', () => {
    const c = certInfo(CERT);
    expect(c.subject).toBe('idp.example.org');
    expect(c.notBefore?.toISOString()).toBe('2026-10-25T13:29:50.000Z');
    expect(c.notAfter?.toISOString()).toBe('2036-10-22T13:29:50.000Z');
    expect(certState(c, NOW)).toBe('valid');
    expect(certState(c, new Date('2036-10-01T00:00:00Z'))).toBe('expiring');
    expect(certState(c, new Date('2036-10-23T00:00:00Z'))).toBe('expired');
    expect(certState(c, new Date('2026-10-01T00:00:00Z'))).toBe('not-yet-valid');
    expect(certState(certInfo('-----BEGIN CERTIFICATE-----\nAAA\n-----END CERTIFICATE-----'))).toBe('unreadable');
    expect(splitPem(`junk ${CERT}\n${CERT}`)).toHaveLength(2);
  });

  it('parses IdP metadata: redirect binding, signing certificates only', () => {
    const r = parseIdpMetadata(METADATA);
    expect(r.ok).toBe(true);
    if (!r.ok) return;
    expect(r.metadata.entityId).toBe('https://idp.example.org/saml');
    expect(r.metadata.ssoUrl).toBe('https://idp.example.org/saml/sso');
    expect(r.metadata.certsPem).toHaveLength(1);
    expect(parseIdpMetadata('not xml <').ok).toBe(false);
  });

  it('worstCert picks the expired one first', () => {
    expect(worstCert(OIDC_ROW)).toBeNull();
    expect(worstCert(SAML_ROW, NOW)?.subject).toBe('idp.example.org');
  });

  it('form values round-trip to the POST body; empty own addresses are filled in', () => {
    const body = ssoInput(ssoValues(SAML_ROW), 'https://mail.example.org', '');
    expect(body).toEqual({ id: 'corp-saml', displayName: 'Contoso SAML', scope: 'domain:example.org', enabled: false, config: SAML_ROW.config, claimMap: SAML_ROW.claimMap });
    const v = { ...ssoValues(null), id: 'x', kind: 'saml' };
    expect(ssoInput(v, 'https://m.example', '/mail').config).toMatchObject({ acsUrl: 'https://m.example/mail/api/sso/x/acs', spEntityId: 'https://m.example/mail/api/sso/x/metadata' });
  });
});
