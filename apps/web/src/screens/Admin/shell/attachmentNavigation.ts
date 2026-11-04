import type { Can, Capability } from './can.ts';
import type { AdminRoute } from './router.ts';

/** Attachment policy tabs (`#/attachments/<id>`) and the registry keys each edits in place. */
export const ATTACHMENT_TABS: readonly { id: string; label: string; capability: Capability; keys: readonly string[] }[] = [
  { id: 'lists', label: 'admin-attachments-tab-lists', capability: 'settings.read', keys: ['attachments.allowedExtensions', 'attachments.allowedMimeTypes', 'attachments.blockedExtensions', 'attachments.blockedMimeTypes'] },
  { id: 'limits', label: 'admin-attachments-tab-limits', capability: 'settings.read', keys: ['attachments.maxBytes'] },
  { id: 'delivery', label: 'admin-attachments-tab-delivery', capability: 'settings.read', keys: ['attachments.downloadAllowed', 'attachments.previewAllowed'] },
  { id: 'warnings', label: 'admin-attachments-tab-warnings', capability: 'settings.read', keys: ['attachments.warnExtensions', 'attachments.warnMimeTypes'] },
  { id: 'test', label: 'admin-attachments-tab-test', capability: 'settings.read', keys: [] },
];

export const canOpenAttachmentTab = (can: Can, tab: (typeof ATTACHMENT_TABS)[number]): boolean => can.can(tab.capability);

/** The tab a registry key belongs to; an `attachments.*` key the tabs do not name goes to Lists. */
export function attachmentTabForKey(key: string): string | undefined {
  const tab = ATTACHMENT_TABS.find((candidate) => candidate.keys.includes(key));
  return tab?.id ?? (key.startsWith('attachments.') ? 'lists' : undefined);
}

/** `#/attachments/<tab>` or the older per-key link `#/attachments/<key>` (focuses that row). */
export function attachmentRouteTab(route: AdminRoute): { tab: string | undefined; focusKey: string | undefined } {
  const first = route.rest[0];
  if (first?.startsWith('attachments.')) return { tab: attachmentTabForKey(first), focusKey: first };
  return { tab: first, focusKey: undefined };
}
