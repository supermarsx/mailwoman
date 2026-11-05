// The Compliance hub's tab registry (`#/compliance/<id>`).
//
// A tab is added with one entry in COMPLIANCE_TABS. Reserved ids that other
// work adds: breaches, restrictions, policy, processing-records, dpia,
// erasure-coverage, encryption (plus the ones below). `status` is optional: when
// present the Overview shows it on the tab's card.

import type { Component } from 'solid-js';
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

export const COMPLIANCE_TABS: readonly ComplianceTab[] = [
  { id: 'data-requests', label: 'admin-nav-data-requests', description: 'compliance-card-data-requests', icon: 'download', capability: 'data_export.run', load: () => import('../DataRequests/index.tsx') },
  { id: 'cookies', label: 'privacy-tab-cookies', description: 'compliance-card-cookies', icon: 'preferences', capability: 'settings.read', unscoped: true, load: () => import('./CookiesHost.tsx') },
  { id: 'retention', label: 'admin-nav-retention', description: 'compliance-card-retention', icon: 'clock', capability: 'settings.read', load: () => import('../Retention/index.tsx') },
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
