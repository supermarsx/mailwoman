// The add / edit form of one SSO backend, on its dedicated route page
// (SettingsFormPage with `frame: 'none'`, t32-cfg UX3-UX7).
//
// One form for both protocols: the OIDC group shows while the protocol is OIDC,
// the SAML group while it is SAML (`when`). Every field has one sentence of help
// and a link into docs/admin/sso.md. Addresses this server answers on (the OIDC
// callback, the SAML ACS and entity id) may be left empty: the form fills in
// this server's own address when it saves, and the help says which one.
//
// Secrets are write-only: the client secret (OIDC) and the service provider key
// (SAML) show whether one is stored and are sent only when something is typed.

import { createMemo, createSignal, Show, type JSX } from 'solid-js';
import { t, isolate } from '../../../i18n';
import { basePath } from '../../../api/basePath.ts';
import type { ClaimMap, FirstLoginPolicy, SsoBackendInput, SsoBackendRow, SsoConfig, SsoKind } from '../../../modules/sso';
import { SettingsFormPage, type FieldRenderContext, type SettingField, type SettingsGroup, type SettingsLoad } from '../shell/templates/SettingsFormPage.tsx';
import * as ux from '../shell/ux.css.ts';
import { ListEditor } from '../shell/ListEditor.tsx';
import { adminDocsHref } from '../shell/docsLink.ts';
import { CertList } from './CertList.tsx';
import { MetadataImport } from './MetadataImport.tsx';

const AREA = 'sso';
export const DEFAULT_NAMEID = 'urn:oasis:names:tc:SAML:2.0:nameid-format:persistent';
const NAMEID_FORMATS = [
  DEFAULT_NAMEID,
  'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress',
  'urn:oasis:names:tc:SAML:2.0:nameid-format:transient',
  'urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified',
];
const ID_PATTERN = '[a-z0-9][a-z0-9-]{0,62}';
const DOMAIN = /^(?=.{1,253}$)(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,63}$/;

/** This server's address for a backend path (`/api/sso/<id>/<leaf>`). */
export function ownAddress(id: string, leaf: 'callback' | 'acs' | 'metadata', origin: string = location.origin, base: string = basePath()): string {
  return `${origin}${base}/api/sso/${encodeURIComponent(id)}/${leaf}`;
}

/** The form values of a backend (or of a new one). */
export function ssoValues(row: SsoBackendRow | null): Record<string, unknown> {
  const c = row?.config ?? null;
  const oidc = c?.kind === 'oidc' ? c : null;
  const saml = c?.kind === 'saml' ? c : null;
  const scope = row?.scope ?? 'deployment';
  const claims: Partial<ClaimMap> = row?.claimMap ?? {};
  const secret = row?.hasSecret ? { set: true } : undefined;
  return {
    id: row?.id ?? '',
    displayName: row?.displayName ?? '',
    kind: c?.kind ?? 'oidc',
    scopeKind: scope.startsWith('domain:') ? 'domain' : 'deployment',
    domain: scope.startsWith('domain:') ? scope.slice('domain:'.length) : '',
    enabled: row?.enabled ?? true,
    firstLoginPolicy: c?.firstLoginPolicy ?? 'allowlist',
    issuerUrl: oidc?.issuerUrl ?? '',
    clientId: oidc?.clientId ?? '',
    oidcSecret: oidc ? secret : undefined,
    redirectUrl: oidc?.redirectUrl ?? '',
    oidcScopes: oidc?.scopes ?? ['openid', 'email', 'profile'],
    spEntityId: saml?.spEntityId ?? '',
    acsUrl: saml?.acsUrl ?? '',
    idpMetadataXml: saml?.idpMetadataXml ?? '',
    idpSsoUrl: saml?.idpSsoUrl ?? '',
    idpSloUrl: saml?.idpSloUrl ?? '',
    idpCerts: saml?.idpSigningCertsPem ?? [],
    idpMetadataUrl: saml?.idpMetadataUrl ?? '',
    wantAssertionsSigned: saml?.wantAssertionsSigned ?? true,
    wantEncrypted: saml?.wantEncrypted ?? false,
    nameidFormat: saml?.nameidFormat ?? DEFAULT_NAMEID,
    samlKey: saml ? secret : undefined,
    claimEmail: claims.email ?? 'email',
    claimUsername: claims.username ?? 'preferred_username',
    claimDisplay: claims.display ?? 'name',
    claimGroups: row === null ? 'groups' : (claims.groups ?? ''),
  };
}

const str = (v: unknown): string => (typeof v === 'string' ? v.trim() : '');
const orNull = (v: unknown): string | null => (str(v) === '' ? null : str(v));

/** The `POST /admin/sso` body from the form values. */
export function ssoInput(v: Readonly<Record<string, unknown>>, origin?: string, base?: string): SsoBackendInput {
  const id = str(v.id);
  const kind = v.kind as SsoKind;
  const firstLoginPolicy = v.firstLoginPolicy as FirstLoginPolicy;
  const config: SsoConfig =
    kind === 'oidc'
      ? {
          kind: 'oidc',
          issuerUrl: str(v.issuerUrl),
          clientId: str(v.clientId),
          redirectUrl: str(v.redirectUrl) || ownAddress(id, 'callback', origin, base),
          scopes: [...((v.oidcScopes as string[] | undefined) ?? [])],
          firstLoginPolicy,
        }
      : {
          kind: 'saml',
          spEntityId: str(v.spEntityId) || ownAddress(id, 'metadata', origin, base),
          acsUrl: str(v.acsUrl) || ownAddress(id, 'acs', origin, base),
          idpMetadataUrl: orNull(v.idpMetadataUrl),
          idpMetadataXml: orNull(v.idpMetadataXml),
          idpSsoUrl: str(v.idpSsoUrl),
          idpSloUrl: orNull(v.idpSloUrl),
          idpSigningCertsPem: [...((v.idpCerts as string[] | undefined) ?? [])],
          wantAssertionsSigned: v.wantAssertionsSigned === true,
          wantEncrypted: v.wantEncrypted === true,
          nameidFormat: str(v.nameidFormat) || DEFAULT_NAMEID,
          firstLoginPolicy,
        };
  const input: SsoBackendInput = {
    id,
    displayName: str(v.displayName),
    scope: v.scopeKind === 'domain' ? `domain:${str(v.domain).toLowerCase()}` : 'deployment',
    enabled: v.enabled === true,
    config,
    claimMap: {
      email: str(v.claimEmail),
      username: str(v.claimUsername),
      display: str(v.claimDisplay),
      groups: orNull(v.claimGroups),
    },
  };
  const secret = kind === 'oidc' ? v.oidcSecret : v.samlKey;
  if (typeof secret === 'string' && secret.trim() !== '') input.secret = secret.trim();
  return input;
}

export interface SsoEditorProps {
  /** The backend, or null for a new one. */
  row: SsoBackendRow | null;
  /** Ids already in use (a new backend may not take one). */
  takenIds: readonly string[];
  /** Domains to offer for a domain backend. */
  domains: readonly string[];
  save: (input: SsoBackendInput) => Promise<SsoBackendRow | null>;
  labelledBy: string;
  onCancel?: () => void;
  onSaving?: (saving: boolean) => void;
  now?: Date | undefined;
}

export function SsoEditor(props: SsoEditorProps): JSX.Element {
  const [saving, setSaving] = createSignal(false);
  const isNew = (): boolean => props.row === null;
  const docs = (id: string): string => adminDocsHref(AREA, id);
  const oidc = (v: Readonly<Record<string, unknown>>): boolean => v.kind === 'oidc';
  const saml = (v: Readonly<Record<string, unknown>>): boolean => v.kind === 'saml';
  const required = (label: string): NonNullable<SettingField['validate']> => (v) => (str(v) === '' ? t('admin-sso-err-required', { field: label }) : null);

  const groups = createMemo((): SettingsGroup[] => [
    {
      id: 'basics',
      heading: t('admin-sso-group-basics'),
      fields: [
        {
          id: 'id',
          label: t('admin-sso-id'),
          help: isNew() ? t('admin-sso-id-help') : t('admin-sso-id-fixed'),
          docsHref: docs('id'),
          kind: 'text',
          required: true,
          maxLength: 63,
          pattern: ID_PATTERN,
          patternHint: t('admin-sso-id-pattern'),
          placeholder: t('admin-sso-id-placeholder'),
          // An existing backend keeps its id (it is in the provider's registered addresses).
          ...(isNew()
            ? {}
            : {
                render: (ctx: FieldRenderContext) => (
                  <input id={ctx.id} class={ux.control} type="text" readOnly value={str(ctx.value)} aria-describedby={ctx.describedBy} />
                ),
              }),
          validate: (v) => (isNew() && props.takenIds.includes(str(v)) ? t('admin-sso-id-taken', { id: isolate(str(v)) }) : null),
        },
        {
          id: 'displayName',
          label: t('admin-sso-display-name'),
          help: t('admin-sso-display-name-help'),
          docsHref: docs('displayName'),
          kind: 'text',
          required: true,
          maxLength: 80,
          placeholder: t('admin-sso-display-name-placeholder'),
        },
        {
          id: 'kind',
          label: t('admin-sso-kind'),
          help: t('admin-sso-kind-help'),
          docsHref: docs('kind'),
          kind: 'select',
          options: [
            { value: 'oidc', label: t('admin-sso-kind-oidc') },
            { value: 'saml', label: t('admin-sso-kind-saml') },
          ],
        },
        {
          id: 'scopeKind',
          label: t('admin-sso-scope'),
          help: t('admin-sso-scope-help'),
          docsHref: docs('scope'),
          kind: 'select',
          options: [
            { value: 'deployment', label: t('admin-sso-scope-deployment') },
            { value: 'domain', label: t('admin-sso-scope-domain') },
          ],
        },
        props.domains.length > 0
          ? {
              id: 'domain',
              label: t('admin-sso-domain'),
              help: t('admin-sso-domain-help'),
              docsHref: docs('scope'),
              kind: 'select',
              required: true,
              options: [{ value: '', label: t('admin-sso-domain-choose') }, ...props.domains.map((d) => ({ value: d, label: d }))],
              when: (v) => v.scopeKind === 'domain',
              validate: required(t('admin-sso-domain')),
            }
          : {
              id: 'domain',
              label: t('admin-sso-domain'),
              help: t('admin-sso-domain-help'),
              docsHref: docs('scope'),
              kind: 'text',
              required: true,
              placeholder: t('admin-sso-domain-placeholder'),
              when: (v) => v.scopeKind === 'domain',
              validate: (v) => (DOMAIN.test(str(v).toLowerCase()) ? null : t('admin-sso-domain-invalid')),
            },
        {
          id: 'firstLoginPolicy',
          label: t('admin-sso-first-login'),
          help: t('admin-sso-first-login-help'),
          docsHref: docs('firstLoginPolicy'),
          kind: 'select',
          options: [
            { value: 'allowlist', label: t('admin-sso-policy-allowlist') },
            { value: 'autocreate', label: t('admin-sso-policy-autocreate') },
          ],
          revert: { defaultDisplay: t('admin-sso-policy-allowlist'), value: 'allowlist' },
        },
        { id: 'enabled', label: t('admin-sso-enabled'), help: t('admin-sso-enabled-help'), docsHref: docs('enabled'), kind: 'boolean' },
      ],
    },
    {
      id: 'oidc',
      heading: t('admin-sso-group-oidc'),
      when: oidc,
      fields: [
        {
          id: 'issuerUrl',
          label: t('admin-sso-issuer'),
          help: t('admin-sso-issuer-help'),
          docsHref: docs('issuerUrl'),
          kind: 'url',
          placeholder: t('admin-sso-issuer-placeholder'),
          validate: required(t('admin-sso-issuer')),
        },
        { id: 'clientId', label: t('admin-sso-client-id'), help: t('admin-sso-client-id-help'), docsHref: docs('clientId'), kind: 'text', validate: required(t('admin-sso-client-id')) },
        { id: 'oidcSecret', label: t('admin-sso-client-secret'), help: t('admin-sso-client-secret-help'), docsHref: docs('clientId'), kind: 'secret', maxLength: 4096 },
        {
          id: 'redirectUrl',
          label: t('admin-sso-redirect'),
          help: t('admin-sso-redirect-help'),
          docsHref: docs('redirectUrl'),
          kind: 'url',
        },
        {
          id: 'oidcScopes',
          label: t('admin-sso-scopes'),
          help: t('admin-sso-scopes-help'),
          docsHref: docs('scopes'),
          kind: 'text',
          revert: { defaultDisplay: 'openid email profile', value: ['openid', 'email', 'profile'] },
          validate: (v) => (Array.isArray(v) && v.includes('openid') ? null : t('admin-sso-scopes-openid')),
          render: (ctx) => (
            <ListEditor
              id={ctx.id}
              label={t('admin-sso-scopes')}
              items={(ctx.value as string[] | undefined) ?? []}
              onChange={(list) => ctx.setValue(list)}
              describedBy={ctx.describedBy}
              invalid={ctx.invalid}
              disabled={ctx.disabled}
              mono
              validate={(s) => (/^[\x21\x23-\x5b\x5d-\x7e]+$/.test(s) ? null : t('admin-sso-scopes-invalid'))}
            />
          ),
        },
      ],
    },
    {
      id: 'saml-idp',
      heading: t('admin-sso-group-saml-idp'),
      help: t('admin-sso-group-saml-idp-help'),
      when: saml,
      fields: [
        {
          id: 'idpMetadataXml',
          label: t('admin-sso-meta'),
          help: t('admin-sso-meta-help'),
          docsHref: docs('idpMetadata'),
          kind: 'text',
          render: (ctx) => (
            <MetadataImport
              id={ctx.id}
              describedBy={ctx.describedBy}
              disabled={ctx.disabled}
              stored={str(ctx.value)}
              now={props.now}
              onUse={(m, xml) => ctx.setValues({ idpMetadataXml: xml, idpSsoUrl: m.ssoUrl ?? '', idpSloUrl: m.sloUrl ?? '', idpCerts: m.certsPem })}
            />
          ),
        },
        {
          id: 'idpSsoUrl',
          label: t('admin-sso-idp-sso-url'),
          help: t('admin-sso-idp-sso-url-help'),
          docsHref: docs('idpSsoUrl'),
          kind: 'url',
          validate: required(t('admin-sso-idp-sso-url')),
        },
        { id: 'idpSloUrl', label: t('admin-sso-idp-slo-url'), help: t('admin-sso-idp-slo-url-help'), docsHref: docs('idpSloUrl'), kind: 'url' },
        {
          id: 'idpCerts',
          label: t('admin-sso-idp-certs'),
          help: t('admin-sso-idp-certs-help'),
          docsHref: docs('idpCerts'),
          kind: 'text',
          validate: (v) => (Array.isArray(v) && v.length > 0 ? null : t('admin-sso-cert-required')),
          render: (ctx) => (
            <CertList
              id={ctx.id}
              describedBy={ctx.describedBy}
              invalid={ctx.invalid}
              disabled={ctx.disabled}
              certs={(ctx.value as string[] | undefined) ?? []}
              onChange={(list) => ctx.setValue(list)}
              now={props.now}
            />
          ),
        },
        {
          id: 'idpMetadataUrl',
          label: t('admin-sso-idp-metadata-url'),
          help: t('admin-sso-idp-metadata-url-help'),
          docsHref: docs('idpMetadata'),
          kind: 'url',
          placeholder: t('admin-sso-idp-metadata-url-placeholder'),
        },
      ],
    },
    {
      id: 'saml-sp',
      heading: t('admin-sso-group-saml-sp'),
      when: saml,
      fields: [
        { id: 'spEntityId', label: t('admin-sso-sp-entity-id'), help: t('admin-sso-sp-entity-id-help'), docsHref: docs('spEntityId'), kind: 'text' },
        { id: 'acsUrl', label: t('admin-sso-acs-url'), help: t('admin-sso-acs-url-help'), docsHref: docs('acsUrl'), kind: 'url' },
        {
          id: 'nameidFormat',
          label: t('admin-sso-nameid-format'),
          help: t('admin-sso-nameid-format-help'),
          docsHref: docs('nameidFormat'),
          kind: 'select',
          options: NAMEID_FORMATS.map((f) => ({ value: f, label: f })),
          revert: { defaultDisplay: DEFAULT_NAMEID, value: DEFAULT_NAMEID },
        },
        {
          id: 'wantAssertionsSigned',
          label: t('admin-sso-want-signed'),
          help: t('admin-sso-want-signed-help'),
          docsHref: docs('wantAssertionsSigned'),
          kind: 'boolean',
          revert: { defaultDisplay: t('admin-sso-on'), value: true },
        },
        { id: 'wantEncrypted', label: t('admin-sso-want-encrypted'), help: t('admin-sso-want-encrypted-help'), docsHref: docs('wantEncrypted'), kind: 'boolean' },
        { id: 'samlKey', label: t('admin-sso-sp-key'), help: t('admin-sso-sp-key-help'), docsHref: docs('wantEncrypted'), kind: 'secret', maxLength: 16384 },
      ],
    },
    {
      id: 'claims',
      heading: t('admin-sso-claims'),
      help: t('admin-sso-claims-help'),
      fields: [
        { id: 'claimEmail', label: t('admin-sso-claim-email'), help: t('admin-sso-claim-email-help'), docsHref: docs('claims'), kind: 'text', required: true },
        { id: 'claimUsername', label: t('admin-sso-claim-username'), help: t('admin-sso-claim-username-help'), docsHref: docs('claims'), kind: 'text', required: true },
        { id: 'claimDisplay', label: t('admin-sso-claim-display'), help: t('admin-sso-claim-display-help'), docsHref: docs('claims'), kind: 'text' },
        { id: 'claimGroups', label: t('admin-sso-claim-groups'), help: t('admin-sso-claim-groups-help'), docsHref: docs('claims'), kind: 'text' },
      ],
    },
  ]);

  async function load(): Promise<SettingsLoad> {
    return { values: ssoValues(props.row) };
  }

  async function save(_changed: Record<string, unknown>, ctx: { values: Readonly<Record<string, unknown>> }): Promise<{ values: Record<string, unknown> } | void> {
    setSaving(true); props.onSaving?.(true);
    try {
      const stored = await props.save(ssoInput(ctx.values));
      if (stored) return { values: ssoValues(stored) };
    } finally { setSaving(false); props.onSaving?.(false); }
  }

  return (
    <>
    <SettingsFormPage
      title={isNew() ? t('admin-sso-add') : t('admin-sso-edit-title')}
      frame="none"
      labelledBy={props.labelledBy}
      groups={groups()}
      load={load}
      save={save}
      saveLabel={isNew() ? t('admin-sso-create') : t('admin-sso-update')}
      testId="sso-editor"
    />
    <Show when={props.onCancel}><div class={ux.pageActions}><button type="button" class="btn btn--ghost" disabled={saving()} onClick={() => props.onCancel?.()}>{t('admin-sso-cancel')}</button></div></Show>
    </>
  );
}
