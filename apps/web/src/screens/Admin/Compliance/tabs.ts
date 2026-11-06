// The Compliance hub's tab registry (`#/compliance/<id>`).
//
// A tab is added with one entry in COMPLIANCE_TABS. Reserved ids that other
// work adds: breaches, restrictions, policy, processing-records, dpia,
// erasure-coverage, encryption (plus the ones below). `status` is optional: when
// present the Overview shows it on the tab's card.

import type { Component } from 'solid-js';
import { t } from '../../../i18n';
import type { ComplianceSummary } from '../../../api/admin/compliance.ts';
import type { Can, Capability } from '../shell/can.ts';
import type { IconName } from '../shell/icons.tsx';

export interface ComplianceStatus {
  state: 'ok' | 'attention' | 'unknown';
  /** Already translated, one short line. */
  text: string;
}

export interface ComplianceTab {
  id: string;
  /** Message id of the tab label. */
  label: string;
  /** Message id of the one-line description on the Overview card. */
  description: string;
  icon: IconName;
  capability: Capability;
  /** Needs an all-domains admin. */
  unscoped?: boolean;
  load: () => Promise<{ default: Component }>;
  status?: () => Promise<ComplianceStatus>;
}

const summary = async (): Promise<ComplianceSummary> => (await import('../../../api/admin/compliance.ts')).complianceApi.summary();

async function policyStatus(): Promise<ComplianceStatus> {
  const s = (await summary()).policy;
  if (s.missing.length > 0) return { state: 'attention', text: t('compliance-status-policy-missing', { count: s.missing.length }) };
  if (s.pending > 0) return { state: 'attention', text: t('compliance-status-policy-pending', { count: s.pending }) };
  return { state: 'ok', text: s.version === null ? t('compliance-policy-none') : t('compliance-status-policy-ok', { version: s.version }) };
}
async function recordsStatus(): Promise<ComplianceStatus> {
  const s = (await summary()).records;
  return { state: 'ok', text: t('compliance-status-records', { activities: s.builtin + s.custom, processors: s.processors }) };
}
async function dpiaStatus(): Promise<ComplianceStatus> {
  const s = (await summary()).dpia;
  if (s.due > 0) return { state: 'attention', text: t('compliance-status-dpia-due', { count: s.due }) };
  if (s.suggestions > 0) return { state: 'attention', text: t('compliance-status-dpia-suggested', { count: s.suggestions }) };
  return { state: 'ok', text: t('compliance-status-dpia', { count: s.total }) };
}

export const COMPLIANCE_TABS: readonly ComplianceTab[] = [
  { id: 'data-requests', label: 'admin-nav-data-requests', description: 'compliance-card-data-requests', icon: 'download', capability: 'data_export.run', load: () => import('../DataRequests/index.tsx') },
  { id: 'cookies', label: 'privacy-tab-cookies', description: 'compliance-card-cookies', icon: 'preferences', capability: 'settings.read', unscoped: true, load: () => import('./CookiesHost.tsx') },
  { id: 'retention', label: 'admin-nav-retention', description: 'compliance-card-retention', icon: 'clock', capability: 'settings.read', load: () => import('../Retention/index.tsx') },
  { id: 'restrictions', label: 'admin-nav-restrictions', description: 'compliance-card-restrictions', icon: 'lock', capability: 'data_export.run', load: () => import('../Restrictions/index.tsx') },
  { id: 'consents', label: 'admin-nav-consents', description: 'compliance-card-consents', icon: 'check', capability: 'settings.read', unscoped: true, load: () => import('../Consents/index.tsx') },
  { id: 'policy', label: 'compliance-tab-policy', description: 'compliance-card-policy', icon: 'configuration', capability: 'settings.read', unscoped: true, load: () => import('./PolicyTab.tsx'), status: policyStatus },
  { id: 'processing-records', label: 'compliance-tab-processing-records', description: 'compliance-card-processing-records', icon: 'table', capability: 'settings.read', unscoped: true, load: () => import('./RecordsTab.tsx'), status: recordsStatus },
  { id: 'dpia', label: 'compliance-tab-dpia', description: 'compliance-card-dpia', icon: 'shield', capability: 'settings.read', unscoped: true, load: () => import('./DpiaTab.tsx'), status: dpiaStatus },
  { id: 'breaches', label: 'admin-nav-breaches', description: 'compliance-card-breaches', icon: 'alert', capability: 'privacy.read', unscoped: true, load: () => import('../Breaches/index.tsx') },
  { id: 'encryption', label: 'admin-nav-encryption', description: 'compliance-card-encryption', icon: 'key', capability: 'settings.read', unscoped: true, load: () => import('../Encryption/index.tsx') },
  { id: 'erasure-coverage', label: 'admin-nav-erasure-coverage', description: 'compliance-card-erasure-coverage', icon: 'sweep', capability: 'privacy.read', unscoped: true, load: () => import('../ErasureCoverage/index.tsx') },
  { id: 'breaches', label: 'admin-nav-breaches', description: 'compliance-card-breaches', icon: 'alert', capability: 'privacy.read', unscoped: true, load: () => import('../Breaches/index.tsx') },
  { id: 'erasure-coverage', label: 'admin-nav-erasure-coverage', description: 'compliance-card-erasure-coverage', icon: 'sweep', capability: 'privacy.read', unscoped: true, load: () => import('../ErasureCoverage/index.tsx') },
  // Registry tail: other areas append one line each, above this comment.
];

export function canOpenComplianceTab(can: Can, tab: ComplianceTab): boolean {
  const grants = can.grants();
  return can.can(tab.capability) && (!tab.unscoped || grants.kind === 'all' || grants.me.domainScope === null);
}

/** Old bookmarks that now live under Compliance. */
export const LEGACY_COMPLIANCE_ROUTES: Readonly<Record<string, string>> = {
  'data-requests': 'data-requests',
  retention: 'retention',
};
