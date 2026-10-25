import type { AdminRole } from '../../../api/admin/admins.ts';
import { t } from '../../../i18n';

const builtins = new Set(['super_admin', 'domain_admin', 'helpdesk', 'auditor', 'security_officer']);

/** Built-in purposes describe capabilities.rs. Custom roles never inherit an
 * authority description merely because their display name matches a built-in. */
export function rolePurpose(role: Pick<AdminRole, 'id' | 'builtin' | 'capabilities' | 'description'>): string {
  return role.builtin && builtins.has(role.id)
    ? t(`admin-role-purpose-${role.id}`)
    : role.description?.trim() || t('admin-role-purpose-custom', { count: role.capabilities.length });
}

export function permissionPurpose(capability: string): string {
  const action = capability.split('.').at(-1);
  if (action === 'read') return t('admin-role-purpose-read');
  if (action === 'export') return t('admin-role-purpose-export');
  if (action === 'revoke' || action === 'revoke_sessions') return t('admin-role-purpose-revoke');
  if (action === 'delete') return t('admin-role-purpose-delete');
  if (action === 'disable') return t('admin-role-purpose-disable');
  if (action === 'create') return t('admin-role-purpose-create');
  return t('admin-role-purpose-specific');
}
