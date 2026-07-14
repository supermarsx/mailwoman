import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { AdminSso } from './index.tsx';
import type { SsoAdminApi, SsoBackendRow } from '../../../modules/sso';

function mockApi(overrides: Partial<SsoAdminApi> = {}): SsoAdminApi {
  return {
    list: vi.fn(async () => []),
    save: vi.fn(async () => undefined),
    remove: vi.fn(async () => undefined),
    ...overrides,
  };
}

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
    idpMetadataUrl: 'https://idp.example.org/saml/metadata',
    idpMetadataXml: null,
    idpSsoUrl: 'https://idp.example.org/saml/sso',
    idpSloUrl: null,
    idpSigningCertsPem: ['-----BEGIN CERTIFICATE-----\nAAA\n-----END CERTIFICATE-----'],
    wantAssertionsSigned: true,
    wantEncrypted: false,
    nameidFormat: 'urn:oasis:names:tc:SAML:2.0:nameid-format:persistent',
    firstLoginPolicy: 'allowlist',
  },
  claimMap: { email: 'email', username: 'uid', display: 'displayName', groups: null },
};

describe('Admin › SSO', () => {
  it('lists configured backends with kind + enabled badges', async () => {
    render(() => <AdminSso api={mockApi({ list: vi.fn(async () => [OIDC_ROW, SAML_ROW]) })} />);
    expect(await screen.findByText('Acme SSO')).toBeInTheDocument();
    expect(screen.getByText('Contoso SAML')).toBeInTheDocument();
    // SAML row exposes an SP-metadata link.
    expect(screen.getByRole('link', { name: 'SP metadata' })).toHaveAttribute(
      'href',
      '/api/sso/corp-saml/metadata',
    );
  });

  it('shows the empty state when there are no backends', async () => {
    render(() => <AdminSso api={mockApi()} />);
    expect(await screen.findByText('No SSO backends configured.')).toBeInTheDocument();
  });

  it('creates an OIDC backend from the form', async () => {
    const save = vi.fn(async () => undefined);
    render(() => <AdminSso api={mockApi({ save })} />);
    fireEvent.input(screen.getByPlaceholderText('corp-oidc'), { target: { value: 'new-oidc' } });
    fireEvent.input(screen.getByPlaceholderText('Sign in with Acme SSO'), { target: { value: 'New IdP' } });
    fireEvent.input(screen.getByPlaceholderText('https://idp.example.org/realms/acme'), {
      target: { value: 'https://idp.test' },
    });
    fireEvent.submit(screen.getByRole('form', { name: 'Add a login backend' }));
    await waitFor(() => expect(save).toHaveBeenCalled());
    const input = (save.mock.calls[0] as unknown[])[0] as {
      id: string;
      config: { kind: string; issuerUrl: string };
    };
    expect(input.id).toBe('new-oidc');
    expect(input.config.kind).toBe('oidc');
    expect(input.config.issuerUrl).toBe('https://idp.test');
  });

  it('toggles enabled via save with the flag flipped', async () => {
    const save = vi.fn(async () => undefined);
    render(() => <AdminSso api={mockApi({ list: vi.fn(async () => [OIDC_ROW]), save })} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Disable Acme SSO' }));
    await waitFor(() => expect(save).toHaveBeenCalled());
    expect(((save.mock.calls[0] as unknown[])[0] as SsoBackendRow).enabled).toBe(false);
  });

  it('deletes a backend', async () => {
    const remove = vi.fn(async () => undefined);
    render(() => <AdminSso api={mockApi({ list: vi.fn(async () => [OIDC_ROW]), remove })} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Delete Acme SSO' }));
    await waitFor(() => expect(remove).toHaveBeenCalledWith('corp-oidc'));
  });

  it('loads a backend into the form for editing and disables the id field', async () => {
    render(() => <AdminSso api={mockApi({ list: vi.fn(async () => [SAML_ROW]) })} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Edit' }));
    // Editing switches the submit label and locks the id.
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeInTheDocument();
    const idField = screen.getByPlaceholderText('corp-oidc') as HTMLInputElement;
    expect(idField.value).toBe('corp-saml');
    expect(idField.disabled).toBe(true);
    // SAML-specific field is populated.
    expect((screen.getByDisplayValue('https://idp.example.org/saml/metadata')).tagName).toBeTruthy();
  });
});
