// Labels of the consequences a settings lock reports in its dry run (worker f01).

import { t } from '../../../i18n';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('admin-settings');

/** `{domainValues, userValues, adminUserValues, profileValues, groupValues}` → text with the count. */
export function impactLabels(): Record<string, (count: number) => string> {
  return {
    domainValues: (count) => t('admin-settings-impact-domain', { count }),
    userValues: (count) => t('admin-settings-impact-user', { count }),
    adminUserValues: (count) => t('admin-settings-impact-admin-user', { count }),
    profileValues: (count) => t('admin-settings-impact-profile', { count }),
    groupValues: (count) => t('admin-settings-impact-group', { count }),
  };
}
