// The effective on/off value of each mail feature setting (`mail.allowSend`,
// `mail.allowReply`, ...). Every key is a global/domain/user boolean, so the
// answer is the shared settings map's value: the administrator's lock or
// default, else the user's own choice. Until the map has loaded, or against a
// server that does not know a key, a feature keeps its default (on).

import {settings} from '../api/settings.ts';
import {t} from '../i18n/index.ts';

export const MAIL_FEATURE_DEFAULTS = {
  allowSend: true,
  allowReply: true,
  allowForward: true,
  allowDownload: true,
  allowPrint: true,
  showSpam: true,
  showAuthStatus: true,
  closeOnDelete: true,
  // Opt-in: the message list's Go to top button is off until an administrator or the user turns it on.
  showGoToTop: false,
  // Opt-in: an icon-only Create folder button next to Compose in the sidebar.
  showCreateFolderButton: false,
} as const;

export type MailFeature = keyof typeof MAIL_FEATURE_DEFAULTS;

/** The registry key of a feature: `allowSend` -> `mail.allowSend`. */
export function mailFeatureKey(feature: MailFeature): string {
  return `mail.${feature}`;
}

/** Reactive: whether `feature` is on for the signed-in user. */
export function mailFeature(feature: MailFeature): boolean {
  const value = settings()[mailFeatureKey(feature)]?.value;
  return typeof value === 'boolean' ? value : MAIL_FEATURE_DEFAULTS[feature];
}

/** `mail.rowActionsMode`: how the per-row quick actions in the message list are presented. */
export const ROW_ACTIONS_MODES = ['open', 'hover', 'click', 'off'] as const;
export type RowActionsMode = (typeof ROW_ACTIONS_MODES)[number];
export const ROW_ACTIONS_MODE_DEFAULT: RowActionsMode = 'open';
export const ROW_ACTIONS_MODE_KEY = 'mail.rowActionsMode';

export function isRowActionsMode(value: unknown): value is RowActionsMode {
  return typeof value === 'string' && (ROW_ACTIONS_MODES as readonly string[]).includes(value);
}

/** Reactive: the administrator's lock or default, else the user's own choice; 'open' until loaded or unknown. */
export function rowActionsMode(): RowActionsMode {
  const value = settings()[ROW_ACTIONS_MODE_KEY]?.value;
  return isRowActionsMode(value) ? value : ROW_ACTIONS_MODE_DEFAULT;
}

/** Tooltip for a control a feature setting turns off. */
export function mailFeatureOffTitle(): string {
  return t('mail-feature-off');
}

/** Whether `mail.showSpam` hides this folder from navigation (the Junk folder; nothing is deleted). */
export function spamFolderHidden(folder: { role?: string | null }): boolean {
  return folder.role === 'junk' && !mailFeature('showSpam');
}

/** What `mail.closeOnDelete` closes when `deleted` messages are gone: the reading pane and the message tabs/panes that show one of them. */
export function viewsClosedByDelete(
  deleted: ReadonlySet<string>,
  open: { previewId: string | null; panes: readonly { id: string; kind: string; reference?: string | undefined }[] },
): { preview: boolean; paneIds: string[] } {
  if (!mailFeature('closeOnDelete') || deleted.size === 0) return { preview: false, paneIds: [] };
  return {
    preview: open.previewId !== null && deleted.has(open.previewId),
    paneIds: open.panes.filter((pane) => pane.kind === 'messages' && pane.reference !== undefined && deleted.has(pane.reference)).map((pane) => pane.id),
  };
}

/** One settled view of the folder list, to tell a deletion from paging. */
export interface ListSnapshot { key: string; ids: ReadonlySet<string>; total: number }

/** Ids another client (or a rule) deleted between two snapshots of the same folder: gone from the list while the folder's total fell by at least as many. Paging moves the window without changing the total, so it is not read as a deletion. */
export function vanishedIds(before: ListSnapshot | null, now: ListSnapshot): Set<string> {
  if (!before || before.key !== now.key || now.total >= before.total) return new Set();
  const gone = new Set([...before.ids].filter((id) => !now.ids.has(id)));
  return gone.size <= before.total - now.total ? gone : new Set();
}
