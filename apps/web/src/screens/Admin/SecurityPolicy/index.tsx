// Admin › Security policy (t32-admin §3.1 e16; absorbs t29-e11; server t29-e6).
//
// One settings form over `GET/PUT /admin/security-policy`: the TLS floor for new
// connections, the global two-factor rule, the Argon2 cost of new password hashes,
// the reading floor and capture protection Mailwoman clients apply, and the
// stored DLP rules (edited as rules, `DlpRulesEditor`). The server re-reads the
// policy every 30 seconds on every process, so each field says so (UX18).
//
// A field an environment variable pins (`pinnedByEnv`) is shown disabled with its
// variable (UX9); a write that touches it anyway is refused by the server (409)
// and the refusal is shown at the field. A PUT carries all eight fields, so the
// form sends every value, not only the changed ones.
//
// The server keeps no per-field change record for this object; the page links the
// audit log filtered to it instead of claiming a last change (honest-data rule).

import { createSignal, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import type { DlpRule } from '../../../api/crypto-types.ts';
import {
  ARGON_BOUNDS,
  POLICY_PIN_VARS,
  loadSecurityPolicy,
  policyFieldFor,
  saveSecurityPolicy,
  type SecurityPolicy,
  type SecurityPolicyWrite,
  type SecurityFloor,
} from '../../../api/admin/policy.ts';
import { SettingsFormPage, type SettingField, type SettingsGroup, type SettingsLoad } from '../shell/templates/SettingsFormPage.tsx';
import type { SourceInfo } from '../shell/SourceBadge.tsx';
import { adminDocsHref } from '../shell/docsLink.ts';
import { historyPath } from '../shell/FieldHistory.tsx';
import { hrefFor } from '../shell/router.ts';
import { DlpRulesEditor, parseDlpRules, validateDlpRules } from './DlpRulesEditor.tsx';

export interface SecurityPolicyApi {
  load(): Promise<SettingsLoadResult>;
  save(policy: SecurityPolicyWrite): Promise<SecurityPolicy>;
}

type SettingsLoadResult = { kind: 'ok'; data: SecurityPolicy } | { kind: 'unavailable' };

export const httpSecurityPolicyApi: SecurityPolicyApi = {
  async load() {
    const r = await loadSecurityPolicy();
    return r.kind === 'ok' ? { kind: 'ok', data: r.data } : { kind: 'unavailable' };
  },
  save: (policy) => saveSecurityPolicy(policy),
};

/** What a deployment that never stored a policy runs with (`SecurityPolicy::default`, mw-admin). */
export const POLICY_DEFAULTS = {
  minTls: '1.2',
  require2fa: false,
  argon2MCost: ARGON_BOUNDS.minMemoryKib,
  argon2TCost: 2,
  argon2PCost: 1,
  capturePolicy: 'off',
  maxSecurityFloor: '',
} as const;

const AREA = 'security-policy';
const NO_FLOOR = '';

/** The form's values from the policy in force. */
export function policyValues(p: SecurityPolicy): Record<string, unknown> {
  const rules = parseDlpRules(p.dlpRulesJson);
  return {
    minTls: p.minTls,
    require2fa: p.require2fa,
    argon2MCost: p.argon2MCost,
    argon2TCost: p.argon2TCost,
    argon2PCost: p.argon2PCost,
    capturePolicy: p.capturePolicy,
    maxSecurityFloor: p.maxSecurityFloor ?? NO_FLOOR,
    // Stored text that is not a list of rules is kept as text so nothing is lost.
    dlpRules: rules,
    dlpRulesJson: rules === null ? p.dlpRulesJson : null,
  };
}

/** The PUT body from the form's values (all eight fields). */
export function policyWrite(v: Readonly<Record<string, unknown>>): SecurityPolicyWrite {
  const rules = v.dlpRules as DlpRule[] | null;
  return {
    minTls: v.minTls as SecurityPolicyWrite['minTls'],
    require2fa: v.require2fa === true,
    argon2MCost: Number(v.argon2MCost),
    argon2TCost: Number(v.argon2TCost),
    argon2PCost: Number(v.argon2PCost),
    capturePolicy: v.capturePolicy as SecurityPolicyWrite['capturePolicy'],
    maxSecurityFloor: v.maxSecurityFloor === NO_FLOOR ? null : (v.maxSecurityFloor as SecurityFloor),
    dlpRulesJson: rules === null ? String(v.dlpRulesJson ?? '[]') : JSON.stringify(rules),
  };
}

export interface AdminSecurityPolicyProps {
  api?: SecurityPolicyApi;
}

export function AdminSecurityPolicy(props: AdminSecurityPolicyProps): JSX.Element {
  const api = props.api ?? httpSecurityPolicyApi;
  onMount(() => void loadCatalog('admin-policy'));

  // The last policy read; the pins and env rules come from it.
  const [current, setCurrent] = createSignal<SecurityPolicy | null>(null);
  const pinned = (field: 'minTls' | 'require2fa'): boolean => current()?.pinnedByEnv.includes(field) ?? false;
  const source = (id: keyof typeof POLICY_DEFAULTS, value: unknown): SourceInfo => {
    if ((id === 'minTls' || id === 'require2fa') && pinned(id)) return { kind: 'env-pin', envVar: POLICY_PIN_VARS[id] };
    return { kind: value === POLICY_DEFAULTS[id] ? 'default' : 'stored' };
  };

  const field = (
    id: keyof typeof POLICY_DEFAULTS,
    rest: Omit<SettingField, 'id' | 'docsHref' | 'propagation' | 'revert' | 'source'> & { defaultDisplay: string },
  ): SettingField => {
    const { defaultDisplay, ...f } = rest;
    return {
      id,
      docsHref: adminDocsHref(AREA, id),
      propagation: 'hot',
      get source() {
        const p = current();
        return p ? source(id, policyValues(p)[id]) : undefined;
      },
      revert: { defaultDisplay, value: POLICY_DEFAULTS[id] },
      ...f,
    } as SettingField;
  };

  const groups = (): SettingsGroup[] => [
    {
      id: 'connections',
      heading: t('admin-policy-group-connections'),
      fields: [
        field('minTls', {
          label: t('admin-policy-min-tls'),
          technical: 'MW_ADMIN_MIN_TLS',
          help: t('admin-policy-min-tls-help'),
          kind: 'select',
          options: [
            { value: '1.2', label: t('admin-policy-min-tls-12') },
            { value: '1.3', label: t('admin-policy-min-tls-13') },
          ],
          defaultDisplay: t('admin-policy-min-tls-12'),
        }),
      ],
    },
    {
      id: 'sign-in',
      heading: t('admin-policy-group-sign-in'),
      help: t('admin-policy-group-sign-in-help'),
      fields: [
        field('require2fa', {
          label: t('admin-policy-require-2fa'),
          technical: 'MW_ADMIN_REQUIRE_2FA',
          help: t('admin-policy-require-2fa-help'),
          kind: 'boolean',
          defaultDisplay: t('admin-policy-off'),
        }),
        field('argon2MCost', {
          label: t('admin-policy-argon-memory'),
          help: t('admin-policy-argon-memory-help', { min: ARGON_BOUNDS.minMemoryKib, max: ARGON_BOUNDS.maxMemoryKib }),
          kind: 'number',
          required: true,
          min: ARGON_BOUNDS.minMemoryKib,
          max: ARGON_BOUNDS.maxMemoryKib,
          step: 1,
          validate: (v, all) =>
            Number(all.argon2TCost) === 1 && Number(v) < ARGON_BOUNDS.minMemoryKibSinglePass
              ? t('admin-policy-argon-single-pass', { min: ARGON_BOUNDS.minMemoryKibSinglePass })
              : null,
          defaultDisplay: String(POLICY_DEFAULTS.argon2MCost),
        }),
        field('argon2TCost', {
          label: t('admin-policy-argon-passes'),
          help: t('admin-policy-argon-passes-help', { max: ARGON_BOUNDS.maxPasses }),
          kind: 'number',
          required: true,
          min: 1,
          max: ARGON_BOUNDS.maxPasses,
          step: 1,
          defaultDisplay: String(POLICY_DEFAULTS.argon2TCost),
        }),
        field('argon2PCost', {
          label: t('admin-policy-argon-parallelism'),
          help: t('admin-policy-argon-parallelism-help', { max: ARGON_BOUNDS.maxParallelism }),
          kind: 'number',
          required: true,
          min: 1,
          max: ARGON_BOUNDS.maxParallelism,
          step: 1,
          defaultDisplay: String(POLICY_DEFAULTS.argon2PCost),
        }),
      ],
    },
    {
      id: 'reading',
      heading: t('admin-policy-group-reading'),
      help: t('admin-policy-group-reading-help'),
      fields: [
        field('maxSecurityFloor', {
          label: t('admin-policy-floor'),
          help: t('admin-policy-floor-help'),
          kind: 'select',
          options: [
            { value: NO_FLOOR, label: t('admin-policy-floor-none') },
            { value: 'full-sanitized', label: t('admin-policy-floor-full-sanitized') },
            { value: 'sanitized-no-media', label: t('admin-policy-floor-sanitized-no-media') },
            { value: 'plain-text', label: t('admin-policy-floor-plain-text') },
          ],
          defaultDisplay: t('admin-policy-floor-none'),
        }),
        field('capturePolicy', {
          label: t('admin-policy-capture'),
          help: t('admin-policy-capture-help'),
          kind: 'select',
          options: [
            { value: 'off', label: t('admin-policy-capture-off') },
            { value: 'protect', label: t('admin-policy-capture-protect') },
          ],
          defaultDisplay: t('admin-policy-capture-off'),
        }),
      ],
    },
    {
      id: 'dlp',
      heading: t('admin-policy-group-dlp'),
      help: t('admin-policy-group-dlp-help'),
      fields: [
        {
          id: 'dlpRules',
          label: t('admin-policy-dlp-rules'),
          help: t('admin-policy-dlp-rules-help'),
          docsHref: adminDocsHref(AREA, 'dlpRules'),
          kind: 'text',
          propagation: 'hot',
          when: (v) => v.dlpRules !== null,
          validate: (v) => (Array.isArray(v) ? validateDlpRules(v as DlpRule[]) : null),
          render: (ctx) => (
            <DlpRulesEditor
              id={ctx.id}
              label={t('admin-policy-dlp-rules')}
              describedBy={ctx.describedBy}
              disabled={ctx.disabled}
              rules={(ctx.value as DlpRule[] | null) ?? []}
              envRules={current()?.envDlpRules ?? []}
              onChange={(rules) => ctx.setValue(rules)}
            />
          ),
        },
        {
          id: 'dlpRulesJson',
          label: t('admin-policy-dlp-json'),
          help: t('admin-policy-dlp-json-help'),
          docsHref: adminDocsHref(AREA, 'dlpRules'),
          kind: 'textarea',
          when: (v) => v.dlpRules === null,
          validate: (v) => {
            try {
              return Array.isArray(JSON.parse(String(v ?? ''))) ? null : t('admin-policy-dlp-json-not-list');
            } catch {
              return t('admin-policy-dlp-json-not-json');
            }
          },
        },
      ],
    },
  ];

  async function load(): Promise<SettingsLoad> {
    const r = await api.load();
    if (r.kind === 'unavailable') return 'unavailable';
    setCurrent(r.data);
    return { values: policyValues(r.data) };
  }

  async function save(_changed: Record<string, unknown>, ctx: { values: Readonly<Record<string, unknown>> }): Promise<{ values: Record<string, unknown> }> {
    const next = await api.save(policyWrite(ctx.values));
    setCurrent(next);
    return { values: policyValues(next) };
  }

  return (
    <SettingsFormPage
      title={t('admin-policy-title')}
      description={t('admin-policy-intro')}
      groups={groups()}
      load={load}
      save={save}
      fieldFor={(pointer) => {
        const name = pointer.replace(/^\//, '');
        return name === 'dlpRulesJson' ? 'dlpRules' : (policyFieldFor(name) ?? name);
      }}
      unavailableReason={t('admin-policy-unavailable')}
      actions={
        <a class="btn btn--ghost" href={hrefFor(historyPath('security_policy:global'))}>
          {t('admin-policy-history')}
        </a>
      }
      testId="admin-security-policy"
    />
  );
}

export default AdminSecurityPolicy;
