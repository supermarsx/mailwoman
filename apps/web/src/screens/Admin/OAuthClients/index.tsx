// Admin › OAuth clients: dynamic client registration (t32-admin §3.1 e16).
//
// One settings form over `GET/PUT /admin/oauth-dcr` (crates/mw-server/src/oauth.rs):
// whether apps may register themselves as OAuth clients (RFC 7591, off unless an
// administrator turns it on), the redirect host suffixes such an app may use (an
// empty list refuses every registration), whether registration needs the shared
// `MW_DCR_INITIAL_ACCESS_TOKEN`, and the scope every self-registered client gets.
// The scope is a ceiling: a client's requested scope never widens it.
//
// The scope members the form has no control for (account and folder selectors,
// IP allowlist, expiry, MCP tools) are sent back as read, so saving never resets
// them. The server records when the policy last changed; the page shows it with
// the History link (UX8).

import { For, createSignal, onMount, type JSX } from 'solid-js';
import { loadCatalog, t, isolate } from '../../../i18n';
import { loadDcrPolicy, noScope, readScope, saveDcrPolicy, type DcrPolicy, type DcrPolicyWrite, type OAuthScope } from '../../../api/admin/policy.ts';
import { SettingsFormPage, type SettingsGroup, type SettingsLoad } from '../shell/templates/SettingsFormPage.tsx';
import { ListEditor } from '../shell/ListEditor.tsx';
import { FieldHistory } from '../shell/FieldHistory.tsx';
import { adminDocsHref } from '../shell/docsLink.ts';
import * as shell from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from '../shell/extras.css.ts';

export interface DcrPolicyApi {
  load(): Promise<{ kind: 'ok'; data: DcrPolicy } | { kind: 'unavailable' }>;
  save(policy: DcrPolicyWrite): Promise<DcrPolicy>;
}

export const httpDcrPolicyApi: DcrPolicyApi = {
  async load() {
    const r = await loadDcrPolicy();
    return r.kind === 'ok' ? { kind: 'ok', data: r.data } : { kind: 'unavailable' };
  },
  save: (policy) => saveDcrPolicy(policy),
};

const AREA = 'oauth-clients';
const PERMISSIONS = ['read', 'send', 'delete'] as const;
const SURFACES = ['mail', 'pim'] as const;
type ScopeFlag = (typeof PERMISSIONS)[number] | (typeof SURFACES)[number];

/** A redirect host suffix: a host name or a domain (`example.org`, `app.example.org`). */
const HOST_SUFFIX = /^(?=.{1,253}$)(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)*[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/;

export function dcrValues(p: DcrPolicy): Record<string, unknown> {
  return {
    enabled: p.enabled,
    requireInitialAccessToken: p.requireInitialAccessToken,
    allowedRedirectHostSuffixes: [...p.allowedRedirectHostSuffixes],
    defaultScope: p.defaultScope === null ? noScope() : readScope(p.defaultScope),
  };
}

export function dcrWrite(v: Readonly<Record<string, unknown>>): DcrPolicyWrite {
  return {
    enabled: v.enabled === true,
    requireInitialAccessToken: v.requireInitialAccessToken === true,
    allowedRedirectHostSuffixes: [...((v.allowedRedirectHostSuffixes as string[] | undefined) ?? [])],
    defaultScope: (v.defaultScope as OAuthScope | undefined) ?? noScope(),
  };
}

export interface AdminOAuthClientsProps {
  api?: DcrPolicyApi;
}

export function AdminOAuthClients(props: AdminOAuthClientsProps): JSX.Element {
  const api = props.api ?? httpDcrPolicyApi;
  onMount(() => void loadCatalog('admin-policy'));
  const [updatedAt, setUpdatedAt] = createSignal<string | null>(null);

  const groups = (): SettingsGroup[] => [
    {
      id: 'registration',
      heading: t('admin-oauth-group-registration'),
      fields: [
        {
          id: 'enabled',
          label: t('admin-oauth-enabled'),
          help: t('admin-oauth-enabled-help'),
          docsHref: adminDocsHref(AREA, 'enabled'),
          kind: 'boolean',
          propagation: 'hot',
          revert: { defaultDisplay: t('admin-oauth-off'), value: false },
          validate: (v, all) =>
            v === true && ((all.allowedRedirectHostSuffixes as string[] | undefined) ?? []).length === 0 ? t('admin-oauth-enabled-needs-hosts') : null,
        },
        {
          id: 'allowedRedirectHostSuffixes',
          label: t('admin-oauth-hosts'),
          help: t('admin-oauth-hosts-help'),
          docsHref: adminDocsHref(AREA, 'allowedRedirectHostSuffixes'),
          kind: 'text',
          propagation: 'hot',
          render: (ctx) => (
            <ListEditor
              id={ctx.id}
              label={t('admin-oauth-hosts')}
              items={(ctx.value as string[] | undefined) ?? []}
              onChange={(list) => ctx.setValue(list)}
              describedBy={ctx.describedBy}
              invalid={ctx.invalid}
              disabled={ctx.disabled}
              lowerCase
              mono
              placeholder={t('admin-oauth-hosts-placeholder')}
              emptyText={t('admin-oauth-hosts-none')}
              validate={(h) => (HOST_SUFFIX.test(h) ? null : t('admin-oauth-hosts-invalid', { host: isolate(h) }))}
            />
          ),
        },
        {
          id: 'requireInitialAccessToken',
          label: t('admin-oauth-token'),
          technical: 'MW_DCR_INITIAL_ACCESS_TOKEN',
          help: t('admin-oauth-token-help'),
          docsHref: adminDocsHref(AREA, 'requireInitialAccessToken'),
          kind: 'boolean',
          propagation: 'hot',
          revert: { defaultDisplay: t('admin-oauth-off'), value: false },
        },
      ],
    },
    {
      id: 'scope',
      heading: t('admin-oauth-group-scope'),
      help: t('admin-oauth-group-scope-help'),
      fields: [
        {
          id: 'defaultScope',
          label: t('admin-oauth-scope'),
          help: t('admin-oauth-scope-help'),
          docsHref: adminDocsHref(AREA, 'defaultScope'),
          kind: 'text',
          propagation: 'hot',
          revert: { defaultDisplay: t('admin-oauth-scope-nothing'), value: noScope() },
          validate: (v) => {
            const s = v as OAuthScope;
            if ((s.read || s.send || s.delete) && !s.mail && !s.pim) return t('admin-oauth-scope-needs-surface');
            if (s.rate_limit !== null && (!Number.isInteger(s.rate_limit) || s.rate_limit < 1)) return t('admin-oauth-scope-rate-invalid');
            return null;
          },
          render: (ctx) => {
            const scope = (): OAuthScope => (ctx.value as OAuthScope | undefined) ?? noScope();
            const set = (patch: Partial<OAuthScope>): void => ctx.setValue({ ...scope(), ...patch });
            const flag = (name: ScopeFlag): JSX.Element => (
              <div class={css.checkRow}>
                <input
                  id={`${ctx.id}-${name}`}
                  type="checkbox"
                  checked={scope()[name]}
                  disabled={ctx.disabled}
                  onChange={(e) => set({ [name]: e.currentTarget.checked } as Partial<OAuthScope>)}
                />
                <label for={`${ctx.id}-${name}`}>{t(`admin-oauth-scope-${name}`)}</label>
              </div>
            );
            return (
              <div class={css.listEditor} id={ctx.id} role="group" aria-label={t('admin-oauth-scope')} aria-describedby={ctx.describedBy}>
                <fieldset class={css.checks}>
                  <legend class={shell.fieldLabel}>{t('admin-oauth-scope-permissions')}</legend>
                  <For each={PERMISSIONS}>{(p) => flag(p)}</For>
                </fieldset>
                <fieldset class={css.checks}>
                  <legend class={shell.fieldLabel}>{t('admin-oauth-scope-surfaces')}</legend>
                  <For each={SURFACES}>{(p) => flag(p)}</For>
                </fieldset>
                <label class={shell.fieldLabel} for={`${ctx.id}-rate`}>
                  {t('admin-oauth-scope-rate')}
                </label>
                <input
                  id={`${ctx.id}-rate`}
                  class={ux.control}
                  type="number"
                  min="1"
                  step="1"
                  inputMode="numeric"
                  disabled={ctx.disabled}
                  value={scope().rate_limit === null ? '' : String(scope().rate_limit)}
                  aria-describedby={`${ctx.id}-rate-help`}
                  onInput={(e) => set({ rate_limit: e.currentTarget.value === '' ? null : Number(e.currentTarget.value) })}
                />
                <p id={`${ctx.id}-rate-help`} class={shell.help}>
                  {t('admin-oauth-scope-rate-help')}
                </p>
              </div>
            );
          },
        },
      ],
    },
  ];

  async function load(): Promise<SettingsLoad> {
    const r = await api.load();
    if (r.kind === 'unavailable') return 'unavailable';
    setUpdatedAt(r.data.updatedAt);
    return { values: dcrValues(r.data) };
  }

  async function save(_changed: Record<string, unknown>, ctx: { values: Readonly<Record<string, unknown>> }): Promise<{ values: Record<string, unknown> }> {
    const next = await api.save(dcrWrite(ctx.values));
    setUpdatedAt(next.updatedAt);
    return { values: dcrValues(next) };
  }

  return (
    <SettingsFormPage
      title={t('admin-oauth-title')}
      description={t('admin-oauth-intro')}
      groups={groups()}
      load={load}
      save={save}
      unavailableReason={t('admin-oauth-unavailable')}
      actions={<FieldHistory target="oauth_dcr:policy" updatedAt={updatedAt()} />}
      testId="admin-oauth-clients"
    />
  );
}

export default AdminOAuthClients;
