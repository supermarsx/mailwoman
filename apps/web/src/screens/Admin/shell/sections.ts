// The admin console's section table (t32-admin.md §2.1).
//
// One row per section: id (the URL fragment segment, `/admin#/<id>`), sidebar
// group, icon, label, the capability that shows it, and the lazily loaded screen.
// The shell, the command palette and the breadcrumbs all read this table, so a
// later stage adds a section by adding a row here (through the `t32-admin-index`
// mini-slot, which covers this file with index.tsx).
//
// Only screens that exist are listed. Sections the plan adds later (access
// control, settings, audit log, admins and roles, logs, branding) get their row
// when their screen lands, so the sidebar never offers an empty page.
//
// Capability choices where the catalogue has no `.read` for the area: the
// two-factor policy and the security policy are shown with `security.read` (their
// writes need `twofa_policy.write` / `security_policy.write`), maintenance with
// `maintenance.run`, server metadata with `server_metadata.write` and OAuth clients
// with `oauth_clients.write` (the screens are editors).

import { createComponent, lazy, type Component } from 'solid-js';
import { t } from '../../../i18n';
import type { AdminApi } from '../../../state/slices/admin.ts';
import type { Capability, Can } from './can.ts';
import type { IconName } from './icons.tsx';
import type { AdminRoute } from './router.ts';

export type GroupId = 'overview' | 'people' | 'security' | 'configuration' | 'operations' | 'personal';

export const GROUPS: readonly { id: GroupId; label: () => string }[] = [
  { id: 'personal', label: () => t('admin-personal-group') },
  { id: 'overview', label: () => t('admin-shell-group-overview') },
  { id: 'people', label: () => t('admin-shell-group-people') },
  { id: 'security', label: () => t('admin-shell-group-security') },
  { id: 'configuration', label: () => t('admin-shell-group-configuration') },
  { id: 'operations', label: () => t('admin-shell-group-operations') },
];

/** What the shell hands every section screen. */
export interface SectionProps {
  api: AdminApi;
  route: AdminRoute;
}

export interface SectionDef {
  id: string;
  group: GroupId;
  icon: IconName;
  label: () => string;
  capability?: Capability;
  anyCapabilities?: readonly Capability[];
  component: Component<SectionProps>;
}

/** Wrap a screen as a lazy section. With `passApi` the screen gets the console's
 *  shared admin client as its `api` prop; otherwise it is rendered with no props
 *  and uses its own production default. */
function screen<P extends object>(load: () => Promise<Component<P>>, passApi: boolean): Component<SectionProps> {
  return lazy(async () => {
    const C = await load();
    const wrapped: Component<SectionProps> = (props) =>
      createComponent(C, (passApi ? { get api() { return props.api; } } : {}) as P);
    return { default: wrapped };
  });
}

export const SECTIONS: readonly SectionDef[] = [
  { id: 'configuration-transfer', group: 'configuration', icon: 'restore', label: () => t('admin-transfer-title'), capability: 'config.read', component: screen(() => import('../ConfigurationTransfer.tsx').then(m => m.ConfigurationTransfer), false) },
  { id: 'package-tracking', group: 'configuration', icon: 'archive', label: () => t('admin-nav-package-tracking'), capability: 'integrations.read', component: screen(() => import('../PackageTracking.tsx').then(m => m.PackageTracking), false) },
  { id: 'identity-provider', group: 'security', icon: 'identityProvider', label: () => t('admin-identity-title'), capability: 'integrations.read', component: screen(() => import('../IdentityProvider/index.tsx').then(m => m.default), false) },
  { id: 'attachments', group: 'security', icon: 'attachment', label: () => t('admin-attachments-title'), capability: 'settings.read', component: screen(() => import('../AttachmentPolicy.tsx').then(m => m.AdminAttachmentPolicy), false) },
  { id: 'nextcloud', group: 'configuration', icon: 'nextcloud', label: () => t('admin-nav-nextcloud'), capability: 'integrations.read', component: screen(() => import('../Nextcloud.tsx').then(m => m.AdminNextcloud), false) },
  { id: 'analytics', group: 'configuration', icon: 'analytics', label: () => t('admin-analytics-title'), capability: 'integrations.read', component: screen(() => import('../Analytics.tsx').then(m => m.Analytics), false) },
  { id: 'branding', group: 'configuration', icon: 'branding', label: () => t('admin-branding-title'), capability: 'branding.read', component: screen(() => import('../Branding/index.tsx').then(m => m.BrandingSettings), false) },
  { id: 'logs', group: 'operations', icon: 'logs', label: () => t('admin-logs-title'), capability: 'logs.read', component: screen(() => import('../Logs.tsx').then((m) => m.Logs), false) },
  { id: 'preferences', group: 'personal', icon: 'preferences', label: () => t('admin-personal-title'), component: screen(() => import('../PersonalPreferences.tsx').then((m) => m.PersonalPreferences), false) },
  {
    id: 'diagnostics',
    group: 'operations',
    icon: 'diagnostics',
    label: () => t('admin-diag-title'),
    capability: 'diagnostics.read',
    component: screen(() => import('../Diagnostics.tsx').then((m) => m.Diagnostics), false),
  },
  {
    id: 'dashboard',
    group: 'overview',
    icon: 'dashboard',
    label: () => t('admin-shell-nav-dashboard'),
    capability: 'dashboard.read',
    component: screen(() => import('../Dashboard/Dashboard.tsx').then((m) => m.Dashboard), false),
  },
  {
    id: 'users',
    group: 'people',
    icon: 'users',
    label: () => t('admin-nav-users'),
    capability: 'users.read',
    component: screen(() => import('../Users.tsx').then((m) => m.Users), false),
  },
  {
    id: 'domains',
    group: 'people',
    icon: 'domains',
    label: () => t('admin-nav-domains'),
    capability: 'domains.read',
    component: screen(() => import('../Domains.tsx').then((m) => m.Domains), false),
  },
  {
    id: 'security',
    group: 'security',
    icon: 'twofactor',
    label: () => t('admin-security-center-title'),
    anyCapabilities: ['security.read','bans.read','diagnostics.read','sessions.read'],
    component: screen(() => import('../SecurityCenter.tsx').then((m) => m.AdminSecurityCenter), true),
  },
  {
    id: 'sso',
    group: 'security',
    icon: 'sso',
    label: () => t('admin-nav-sso'),
    capability: 'sso.read',
    component: screen(() => import('../Sso/index.tsx').then((m) => m.AdminSso), false),
  },
  {
    id: 'admins',
    group: 'security',
    icon: 'admins',
    label: () => t('admin-accounts-title'),
    capability: 'admins.read',
    component: screen(() => import('../Admins/index.tsx').then((m) => m.AdminAdmins), false),
  },
  {
    id: 'roles',
    group: 'security',
    icon: 'roles',
    label: () => t('admin-accounts-roles'),
    capability: 'admins.read',
    component: screen(() => import('../Roles/index.tsx').then((m) => m.AdminRoles), false),
  },
  {
    id: 'oauth-clients',
    group: 'security',
    icon: 'oauthClients',
    label: () => t('admin-nav-oauth-clients'),
    capability: 'oauth_clients.write',
    component: screen(() => import('../OAuthClients/index.tsx').then((m) => m.AdminOAuthClients), false),
  },
  {
    id: 'settings',
    group: 'configuration',
    icon: 'settings',
    label: () => t('admin-shell-nav-settings'),
    capability: 'settings.read',
    component: screen(() => import('../Settings/index.tsx').then((m) => m.AdminSettings), false),
  },
  {
    id: 'config',
    group: 'configuration',
    icon: 'configuration',
    label: () => t('admin-config-title'),
    capability: 'config.read',
    component: screen(() => import('../Configuration.tsx').then((m) => m.Configuration), false),
  },
  {
    id: 'ai-assistant',
    group: 'configuration',
    icon: 'assistant',
    label: () => t('admin-nav-assist'),
    capability: 'ai_assistant.read',
    component: screen(() => import('../Assist/index.tsx').then((m) => m.AdminAssist), false),
  },
  {
    id: 'integrations',
    group: 'configuration',
    icon: 'integrations',
    label: () => t('admin-nav-integrations'),
    capability: 'integrations.read',
    component: screen(() => import('../IntegrationsRouter.tsx').then((m) => m.IntegrationsRouter), false),
  },
  {
    id: 'plugins',
    group: 'configuration',
    icon: 'plugins',
    label: () => t('admin-nav-plugins'),
    capability: 'plugins.read',
    component: screen(() => import('../Plugins/index.tsx').then((m) => m.AdminPlugins), false),
  },
  {
    id: 'ui-plugins',
    group: 'configuration',
    icon: 'uiPlugins',
    label: () => t('plugins-admin-ui-nav'),
    capability: 'plugins.read',
    component: screen(() => import('../UiPlugins/index.tsx').then((m) => m.AdminUiPlugins), false),
  },
  {
    id: 'privacy',
    group: 'security',
    icon: 'privacy',
    label: () => t('privacy-title'),
    capability: 'egress.read',
    component: screen(() => import('../Privacy.tsx').then((m) => m.AdminPrivacy), false),
  },
  {
    id: 'egress',
    group: 'configuration',
    icon: 'egress',
    label: () => t('privacy-outbound-title'),
    capability: 'egress.read',
    component: screen(() => import('../Outbound.tsx').then((m) => m.AdminOutbound), false),
  },
  {
    id: 'observability',
    group: 'operations',
    icon: 'observability',
    label: () => t('admin-nav-observability'),
    anyCapabilities: ['audit.read','observability.read'],
    component: screen(() => import('../Observability.tsx').then((m) => m.Observability), false),
  },
  {
    id: 'maintenance',
    group: 'operations',
    icon: 'maintenance',
    label: () => t('admin-nav-rethread'),
    anyCapabilities: ['maintenance.read', 'maintenance.run'],
    component: screen(() => import('../Maintenance.tsx').then((m) => m.Maintenance), true),
  },
  {
    id: 'server-metadata',
    group: 'operations',
    icon: 'serverMetadata',
    label: () => t('admin-nav-servermeta'),
    capability: 'server_metadata.write',
    component: screen(() => import('../ServerMetadata.tsx').then((m) => m.ServerMetadata), true),
  },
];

/** The section for an id, or undefined. */
export function findSection(id: string, sections: readonly SectionDef[] = SECTIONS): SectionDef | undefined {
  return sections.find((s) => s.id === id);
}

/** Shared sidebar and route admission; child screens retain their own gates. */
export function canOpenSection(section: SectionDef, can: Can): boolean {
  return (section.capability === undefined || can.can(section.capability)) && (section.anyCapabilities === undefined || section.anyCapabilities.some(cap => can.can(cap)));
}
