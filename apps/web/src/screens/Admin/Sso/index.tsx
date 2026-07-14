// Admin › SSO (t9 e4, SPEC §18.3 / §19). Add / edit / enable / delete OIDC +
// SAML login backends, per-deployment or per-domain. Edits the OIDC fields
// (issuer, client id, write-only client secret, scopes) and the SAML fields
// (IdP metadata URL or pasted XML, entity ids, signing certs, flags), the
// claim-map (email / username / display / groups), and the first-login policy
// (DEFAULT Allowlist = deny; AutoCreate provisions unknown subjects).
//
// It talks to `/admin/sso` via its OWN injected `SsoAdminApi` (a self-contained
// client — it does not extend the shared `AdminApi`), so it stays inside this
// executor's ownership boundary. Tests inject a mock; production defaults to the
// same-origin HTTP client.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import {
  createHttpSsoAdminApi,
  ssoMetadataPath,
  type ClaimMap,
  type FirstLoginPolicy,
  type SsoAdminApi,
  type SsoBackendInput,
  type SsoBackendRow,
  type SsoConfig,
  type SsoKind,
} from '../../../modules/sso';
import { t, loadCatalog } from '../../../i18n';
import * as css from '../admin.css.ts';

export interface AdminSsoProps {
  /** Tests inject a mock; production defaults to the same-origin HTTP client. */
  api?: SsoAdminApi;
}

/** Split a textarea into a trimmed, non-empty line/comma list. */
function parseList(raw: string): string[] {
  return raw
    .split(/[\n,]/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

/** Split PEM certs on a blank line between `-----END-----`/`-----BEGIN-----`. */
function parseCerts(raw: string): string[] {
  return raw
    .split(/\n\s*\n/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

const KINDS: readonly SsoKind[] = ['oidc', 'saml'];
const POLICIES: readonly FirstLoginPolicy[] = ['allowlist', 'autocreate'];
const DEFAULT_NAMEID = 'urn:oasis:names:tc:SAML:2.0:nameid-format:persistent';

export function AdminSso(props: AdminSsoProps): JSX.Element {
  const api = props.api ?? createHttpSsoAdminApi();
  const [backends, setBackends] = createSignal<SsoBackendRow[]>([]);
  const [error, setError] = createSignal<string | null>(null);

  // ── Form state ────────────────────────────────────────────────────────────
  const [editingId, setEditingId] = createSignal<string | null>(null);
  const [id, setId] = createSignal('');
  const [displayName, setDisplayName] = createSignal('');
  const [kind, setKind] = createSignal<SsoKind>('oidc');
  const [scopeKind, setScopeKind] = createSignal<'deployment' | 'domain'>('deployment');
  const [domain, setDomain] = createSignal('');
  const [enabled, setEnabled] = createSignal(true);
  const [policy, setPolicy] = createSignal<FirstLoginPolicy>('allowlist');
  const [secret, setSecret] = createSignal('');

  // OIDC
  const [issuerUrl, setIssuerUrl] = createSignal('');
  const [clientId, setClientId] = createSignal('');
  const [redirectUrl, setRedirectUrl] = createSignal('');
  const [oidcScopes, setOidcScopes] = createSignal('openid email profile');

  // SAML
  const [spEntityId, setSpEntityId] = createSignal('');
  const [acsUrl, setAcsUrl] = createSignal('');
  const [idpMetadataUrl, setIdpMetadataUrl] = createSignal('');
  const [idpMetadataXml, setIdpMetadataXml] = createSignal('');
  const [idpSsoUrl, setIdpSsoUrl] = createSignal('');
  const [idpSloUrl, setIdpSloUrl] = createSignal('');
  const [certs, setCerts] = createSignal('');
  const [wantAssertionsSigned, setWantAssertionsSigned] = createSignal(true);
  const [wantEncrypted, setWantEncrypted] = createSignal(false);
  const [nameidFormat, setNameidFormat] = createSignal(DEFAULT_NAMEID);

  // Claim map
  const [claimEmail, setClaimEmail] = createSignal('email');
  const [claimUsername, setClaimUsername] = createSignal('preferred_username');
  const [claimDisplay, setClaimDisplay] = createSignal('name');
  const [claimGroups, setClaimGroups] = createSignal('groups');

  async function reload(): Promise<void> {
    try {
      setBackends(await api.list());
      setError(null);
    } catch {
      setError(t('admin-sso-load-error'));
    }
  }
  onMount(() => void loadCatalog('admin'));
  onMount(() => void reload());

  function resetForm(): void {
    setEditingId(null);
    setId('');
    setDisplayName('');
    setKind('oidc');
    setScopeKind('deployment');
    setDomain('');
    setEnabled(true);
    setPolicy('allowlist');
    setSecret('');
    setIssuerUrl('');
    setClientId('');
    setRedirectUrl('');
    setOidcScopes('openid email profile');
    setSpEntityId('');
    setAcsUrl('');
    setIdpMetadataUrl('');
    setIdpMetadataXml('');
    setIdpSsoUrl('');
    setIdpSloUrl('');
    setCerts('');
    setWantAssertionsSigned(true);
    setWantEncrypted(false);
    setNameidFormat(DEFAULT_NAMEID);
    setClaimEmail('email');
    setClaimUsername('preferred_username');
    setClaimDisplay('name');
    setClaimGroups('groups');
  }

  /** Load an existing backend into the form for editing (secret stays blank —
   *  it is write-only and never returned; leaving it blank preserves it). */
  function edit(b: SsoBackendRow): void {
    setEditingId(b.id);
    setId(b.id);
    setDisplayName(b.displayName);
    setKind(b.config.kind);
    setEnabled(b.enabled);
    setSecret('');
    if (b.scope.startsWith('domain:')) {
      setScopeKind('domain');
      setDomain(b.scope.slice('domain:'.length));
    } else {
      setScopeKind('deployment');
      setDomain('');
    }
    setPolicy(b.config.firstLoginPolicy);
    setClaimEmail(b.claimMap.email);
    setClaimUsername(b.claimMap.username);
    setClaimDisplay(b.claimMap.display);
    setClaimGroups(b.claimMap.groups ?? '');
    if (b.config.kind === 'oidc') {
      setIssuerUrl(b.config.issuerUrl);
      setClientId(b.config.clientId);
      setRedirectUrl(b.config.redirectUrl);
      setOidcScopes(b.config.scopes.join(' '));
    } else {
      setSpEntityId(b.config.spEntityId);
      setAcsUrl(b.config.acsUrl);
      setIdpMetadataUrl(b.config.idpMetadataUrl ?? '');
      setIdpMetadataXml(b.config.idpMetadataXml ?? '');
      setIdpSsoUrl(b.config.idpSsoUrl);
      setIdpSloUrl(b.config.idpSloUrl ?? '');
      setCerts(b.config.idpSigningCertsPem.join('\n\n'));
      setWantAssertionsSigned(b.config.wantAssertionsSigned);
      setWantEncrypted(b.config.wantEncrypted);
      setNameidFormat(b.config.nameidFormat);
    }
  }

  function buildConfig(): SsoConfig {
    const firstLoginPolicy = policy();
    if (kind() === 'oidc') {
      return {
        kind: 'oidc',
        issuerUrl: issuerUrl().trim(),
        clientId: clientId().trim(),
        redirectUrl: redirectUrl().trim(),
        scopes: parseList(oidcScopes().replace(/\s+/g, ' ')),
        firstLoginPolicy,
      };
    }
    return {
      kind: 'saml',
      spEntityId: spEntityId().trim(),
      acsUrl: acsUrl().trim(),
      idpMetadataUrl: idpMetadataUrl().trim() === '' ? null : idpMetadataUrl().trim(),
      idpMetadataXml: idpMetadataXml().trim() === '' ? null : idpMetadataXml().trim(),
      idpSsoUrl: idpSsoUrl().trim(),
      idpSloUrl: idpSloUrl().trim() === '' ? null : idpSloUrl().trim(),
      idpSigningCertsPem: parseCerts(certs()),
      wantAssertionsSigned: wantAssertionsSigned(),
      wantEncrypted: wantEncrypted(),
      nameidFormat: nameidFormat().trim(),
      firstLoginPolicy,
    };
  }

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    if (id().trim() === '' || displayName().trim() === '') return;
    const claimMap: ClaimMap = {
      email: claimEmail().trim(),
      username: claimUsername().trim(),
      display: claimDisplay().trim(),
      groups: claimGroups().trim() === '' ? null : claimGroups().trim(),
    };
    const input: SsoBackendInput = {
      id: id().trim(),
      displayName: displayName().trim(),
      scope: scopeKind() === 'domain' ? `domain:${domain().trim()}` : 'deployment',
      enabled: enabled(),
      config: buildConfig(),
      claimMap,
    };
    if (secret().trim() !== '') input.secret = secret().trim();
    try {
      await api.save(input);
      resetForm();
      await reload();
    } catch {
      setError(t('admin-sso-save-error'));
    }
  }

  async function onToggle(b: SsoBackendRow): Promise<void> {
    try {
      await api.save({ ...b, enabled: !b.enabled });
      await reload();
    } catch {
      setError(t('admin-sso-save-error'));
    }
  }

  async function onDelete(backendId: string): Promise<void> {
    try {
      await api.remove(backendId);
      if (editingId() === backendId) resetForm();
      await reload();
    } catch {
      setError(t('admin-sso-delete-error'));
    }
  }

  return (
    <section class={css.section} aria-label={t('admin-sso-title')}>
      <h2 class={css.heading}>{t('admin-sso-title')}</h2>
      <p class={css.note}>{t('admin-sso-intro')}</p>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <form
        class={css.card}
        onSubmit={(e) => void onSubmit(e)}
        aria-label={editingId() !== null ? t('admin-sso-edit') : t('admin-sso-add')}
      >
        <div class={css.grid}>
          <label class="field">
            <span>{t('admin-sso-id')}</span>
            <input
              type="text"
              value={id()}
              disabled={editingId() !== null}
              placeholder={t('admin-sso-id-placeholder')}
              onInput={(e) => setId(e.currentTarget.value)}
            />
          </label>
          <label class="field">
            <span>{t('admin-sso-display-name')}</span>
            <input
              type="text"
              value={displayName()}
              placeholder={t('admin-sso-display-name-placeholder')}
              onInput={(e) => setDisplayName(e.currentTarget.value)}
            />
          </label>
        </div>

        <div class={css.grid}>
          <label class="field">
            <span>{t('admin-sso-kind')}</span>
            <select value={kind()} onChange={(e) => setKind(e.currentTarget.value as SsoKind)}>
              <For each={KINDS}>{(k) => <option value={k}>{t(`admin-sso-kind-${k}`)}</option>}</For>
            </select>
          </label>
          <label class="field">
            <span>{t('admin-sso-scope')}</span>
            <select
              value={scopeKind()}
              onChange={(e) => setScopeKind(e.currentTarget.value as 'deployment' | 'domain')}
            >
              <option value="deployment">{t('admin-sso-scope-deployment')}</option>
              <option value="domain">{t('admin-sso-scope-domain')}</option>
            </select>
          </label>
        </div>
        <Show when={scopeKind() === 'domain'}>
          <label class="field">
            <span>{t('admin-sso-domain')}</span>
            <input
              type="text"
              value={domain()}
              placeholder={t('admin-sso-domain-placeholder')}
              onInput={(e) => setDomain(e.currentTarget.value)}
            />
          </label>
        </Show>

        {/* OIDC fields */}
        <Show when={kind() === 'oidc'}>
          <label class="field">
            <span>{t('admin-sso-issuer')}</span>
            <input
              type="url"
              value={issuerUrl()}
              placeholder={t('admin-sso-issuer-placeholder')}
              onInput={(e) => setIssuerUrl(e.currentTarget.value)}
            />
          </label>
          <div class={css.grid}>
            <label class="field">
              <span>{t('admin-sso-client-id')}</span>
              <input type="text" value={clientId()} onInput={(e) => setClientId(e.currentTarget.value)} />
            </label>
            <label class="field">
              <span>{t('admin-sso-client-secret')}</span>
              <input
                type="password"
                autocomplete="off"
                value={secret()}
                placeholder={editingId() !== null ? t('admin-sso-secret-unchanged') : ''}
                onInput={(e) => setSecret(e.currentTarget.value)}
              />
            </label>
          </div>
          <label class="field">
            <span>{t('admin-sso-redirect')}</span>
            <input type="url" value={redirectUrl()} onInput={(e) => setRedirectUrl(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>{t('admin-sso-scopes')}</span>
            <input type="text" value={oidcScopes()} onInput={(e) => setOidcScopes(e.currentTarget.value)} />
          </label>
        </Show>

        {/* SAML fields */}
        <Show when={kind() === 'saml'}>
          <div class={css.grid}>
            <label class="field">
              <span>{t('admin-sso-sp-entity-id')}</span>
              <input type="text" value={spEntityId()} onInput={(e) => setSpEntityId(e.currentTarget.value)} />
            </label>
            <label class="field">
              <span>{t('admin-sso-acs-url')}</span>
              <input type="url" value={acsUrl()} onInput={(e) => setAcsUrl(e.currentTarget.value)} />
            </label>
          </div>
          <label class="field">
            <span>{t('admin-sso-idp-metadata-url')}</span>
            <input
              type="url"
              value={idpMetadataUrl()}
              placeholder={t('admin-sso-idp-metadata-url-placeholder')}
              onInput={(e) => setIdpMetadataUrl(e.currentTarget.value)}
            />
          </label>
          <label class="field">
            <span>{t('admin-sso-idp-metadata-xml')}</span>
            <textarea
              rows={3}
              value={idpMetadataXml()}
              placeholder={t('admin-sso-idp-metadata-xml-placeholder')}
              onInput={(e) => setIdpMetadataXml(e.currentTarget.value)}
            />
          </label>
          <div class={css.grid}>
            <label class="field">
              <span>{t('admin-sso-idp-sso-url')}</span>
              <input type="url" value={idpSsoUrl()} onInput={(e) => setIdpSsoUrl(e.currentTarget.value)} />
            </label>
            <label class="field">
              <span>{t('admin-sso-idp-slo-url')}</span>
              <input type="url" value={idpSloUrl()} onInput={(e) => setIdpSloUrl(e.currentTarget.value)} />
            </label>
          </div>
          <label class="field">
            <span>{t('admin-sso-idp-certs')}</span>
            <textarea
              rows={3}
              value={certs()}
              placeholder={t('admin-sso-idp-certs-placeholder')}
              onInput={(e) => setCerts(e.currentTarget.value)}
            />
          </label>
          <label class="field">
            <span>{t('admin-sso-nameid-format')}</span>
            <input type="text" value={nameidFormat()} onInput={(e) => setNameidFormat(e.currentTarget.value)} />
          </label>
          <label class={css.listRow}>
            <span>{t('admin-sso-want-signed')}</span>
            <input
              type="checkbox"
              checked={wantAssertionsSigned()}
              onChange={(e) => setWantAssertionsSigned(e.currentTarget.checked)}
            />
          </label>
          <label class={css.listRow}>
            <span>{t('admin-sso-want-encrypted')}</span>
            <input
              type="checkbox"
              checked={wantEncrypted()}
              onChange={(e) => setWantEncrypted(e.currentTarget.checked)}
            />
          </label>
        </Show>

        {/* Claim map */}
        <fieldset class="field">
          <legend>{t('admin-sso-claims')}</legend>
          <div class={css.grid}>
            <label class="field">
              <span>{t('admin-sso-claim-email')}</span>
              <input type="text" value={claimEmail()} onInput={(e) => setClaimEmail(e.currentTarget.value)} />
            </label>
            <label class="field">
              <span>{t('admin-sso-claim-username')}</span>
              <input type="text" value={claimUsername()} onInput={(e) => setClaimUsername(e.currentTarget.value)} />
            </label>
            <label class="field">
              <span>{t('admin-sso-claim-display')}</span>
              <input type="text" value={claimDisplay()} onInput={(e) => setClaimDisplay(e.currentTarget.value)} />
            </label>
            <label class="field">
              <span>{t('admin-sso-claim-groups')}</span>
              <input type="text" value={claimGroups()} onInput={(e) => setClaimGroups(e.currentTarget.value)} />
            </label>
          </div>
        </fieldset>

        <div class={css.grid}>
          <label class="field">
            <span>{t('admin-sso-first-login')}</span>
            <select value={policy()} onChange={(e) => setPolicy(e.currentTarget.value as FirstLoginPolicy)}>
              <For each={POLICIES}>{(p) => <option value={p}>{t(`admin-sso-policy-${p}`)}</option>}</For>
            </select>
          </label>
          <label class={css.listRow}>
            <span>{t('admin-sso-enabled')}</span>
            <input type="checkbox" checked={enabled()} onChange={(e) => setEnabled(e.currentTarget.checked)} />
          </label>
        </div>

        <div class={css.listRow}>
          <button type="submit" class="btn btn--primary">
            {editingId() !== null ? t('admin-sso-update') : t('admin-sso-create')}
          </button>
          <Show when={editingId() !== null}>
            <button type="button" class="btn btn--ghost" onClick={() => resetForm()}>
              {t('admin-sso-cancel')}
            </button>
          </Show>
        </div>
      </form>

      <div class={css.card}>
        <Show when={backends().length > 0} fallback={<p class={css.note}>{t('admin-sso-empty')}</p>}>
          <For each={backends()}>
            {(b) => (
              <div class={css.listRow} data-sso-id={b.id}>
                <div>
                  <strong dir="auto">{b.displayName}</strong>{' '}
                  <span class={css.badge}>{t(`admin-sso-kind-${b.config.kind}`)}</span>{' '}
                  <span class={b.enabled ? css.badge : css.badgeDeferred}>
                    {b.enabled ? t('admin-sso-badge-enabled') : t('admin-sso-badge-disabled')}
                  </span>{' '}
                  <span class={css.note}>{b.scope}</span>
                </div>
                <div class={css.listRow}>
                  <Show when={b.config.kind === 'saml'}>
                    <a class="btn btn--ghost" href={ssoMetadataPath(b.id)}>
                      {t('admin-sso-metadata')}
                    </a>
                  </Show>
                  <button type="button" class="btn btn--ghost" onClick={() => edit(b)}>
                    {t('admin-sso-edit')}
                  </button>
                  <button
                    type="button"
                    class="btn btn--ghost"
                    aria-label={
                      b.enabled
                        ? t('admin-sso-disable-for', { name: b.displayName })
                        : t('admin-sso-enable-for', { name: b.displayName })
                    }
                    onClick={() => void onToggle(b)}
                  >
                    {b.enabled ? t('admin-sso-disable') : t('admin-sso-enable')}
                  </button>
                  <button
                    type="button"
                    class="btn btn--ghost"
                    aria-label={t('admin-sso-delete-for', { name: b.displayName })}
                    onClick={() => void onDelete(b.id)}
                  >
                    {t('admin-sso-delete')}
                  </button>
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>
    </section>
  );
}

export default AdminSso;
