// Where the policy pages are: the administrator's own address when set, else the
// pages this server generates (`/privacy`, `/cookies-policy`) while they are on.

import { basePath } from '../api/basePath.ts';
import type { ConsentConfig } from './policy.ts';

export function policyLinks(config: ConsentConfig | null | undefined): { cookies: string | null; privacy: string | null } {
  const pages = config?.policyPages === true;
  return {
    cookies: config?.policyUrl || (pages ? `${basePath()}/cookies-policy` : null),
    privacy: config?.privacyUrl || (pages ? `${basePath()}/privacy` : null),
  };
}
