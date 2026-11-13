// Which icon each action wears. The menus, toolbars and bars import ACTION_ICONS
// instead of naming an icon inline, so one action reads the same everywhere and
// icons.actions.test.ts can check that no two different actions in one menu share
// a glyph (ACTION_MENUS lists what each menu shows). Drawing lives in
// components/Icon.tsx; docs/ui-icons.md has the audit table.

import type { IconName } from './components/Icon.tsx';

export const ACTION_ICONS = {
  // Message actions
  open: 'mail',
  saveSender: 'personAdd',
  saveContacts: 'teamAdd',
  openContact: 'person',
  markRead: 'mailRead',
  markUnread: 'markUnread',
  markAllRead: 'markAllRead',
  flag: 'flag',
  unflag: 'flagOff',
  pin: 'pin',
  unpin: 'pinOff',
  reply: 'reply',
  replyAll: 'replyAll',
  forward: 'forward',
  forwardAttachment: 'forwardAttachment',
  openReplyDraft: 'compose',
  goReply: 'goReply',
  goForward: 'goForward',
  markAnswered: 'markAnswered',
  markForwarded: 'markForwarded',
  archive: 'archiveDown',
  spam: 'spam',
  trash: 'delete',
  moveTo: 'moveTo',
  copyTo: 'copyTo',
  label: 'tag',
  followUp: 'followUp',
  conversation: 'conversation',
  links: 'link',
  export: 'exportFile',
  print: 'print',
  sweep: 'sweep',
  headers: 'serverMetadata',
  // Copy family: the two-sheet base with a small badge
  copy: 'copy',
  copySender: 'copyAddress',
  copySubject: 'copySubject',
  // Attachments
  attachment: 'attachment',
  downloadAll: 'downloadZip',
  download: 'download',
  focusFocused: 'star',
  focusOther: 'inbox',
  // Search
  search: 'search',
  searchSender: 'person',
  searchDomain: 'domains',
  searchSubject: 'mail',
  searchFolder: 'folder',
  // Snooze family
  snooze: 'snooze',
  snoozeLater: 'snoozeLater',
  snoozeEvening: 'snoozeEvening',
  snoozeTomorrow: 'snoozeTomorrow',
  snoozeWeekend: 'snoozeWeekend',
  snoozeNextWeek: 'snoozeNextWeek',
  snoozeMonth: 'snoozeMonth',
  snoozePick: 'snoozePick',
  unsnooze: 'unsnooze',
  snoozedFolder: 'snoozedFolder',
  remindNoReply: 'remindNoReply',
  // Folder actions
  openFolder: 'folder',
  openFolderTab: 'openTab',
  markFolderRead: 'markAllRead',
  markSelectedFoldersRead: 'mailRead',
  folderLayout: 'preferences',
  folderStar: 'star',
  folderUnstar: 'starOutline',
  arrangeFolders: 'grip',
  refresh: 'refresh',
  newFolder: 'folderPlus',
  renameFolder: 'edit',
  cleanupFolder: 'cleanup',
  emptyFolder: 'emptyFolder',
  folderSettings: 'settings',
  retention: 'archive',
  hideFolder: 'eyeOff',
  sharing: 'users',
  moreFolderActions: 'more',
  deleteFolder: 'delete',
  copyAllMessages: 'copyTo',
  moveAllMessages: 'moveTo',
  moveFolderUp: 'chevronUp',
  moveFolderDown: 'chevronDown',
  // Outbox
  outboxOpen: 'mail',
  outboxSendNow: 'sent',
  outboxSendAgain: 'refresh',
  outboxEditTime: 'clock',
  outboxRecall: 'undo',
  outboxEdit: 'edit',
  outboxDelete: 'delete',
  // Not yet in a menu; drawn so the next action that needs one has its own glyph
  duplicate: 'duplicate',
  copyLink: 'copyLink',
  copyId: 'copyId',
  copyText: 'copyText',
  copyPath: 'copyPath',
  copyHeaders: 'copyHeaders',
  openWindow: 'openWindow',
  openSide: 'openSide',
  viewSource: 'fileCode',
  deletePermanently: 'deletePermanently',
  redirect: 'redirect',
  mute: 'mute',
  blockSender: 'block',
  reportSpam: 'shieldAlert',
  notSpam: 'notSpam',
  reportPhishing: 'phishing',
  createFilter: 'createFilter',
  properties: 'info',
  expandAll: 'expandAll',
  collapseAll: 'collapseAll',
  selectAll: 'selectAll',
  clearSelection: 'clearSelection',
  // Composer Format tab
  fontGrow: 'fontGrow',
  fontShrink: 'fontShrink',
  indentMore: 'indentMore',
  indentLess: 'indentLess',
  copyFormat: 'brushCopy',
  applyFormat: 'brushApply',
  textLtr: 'textLtr',
  textRtl: 'textRtl',
  lineSpacing: 'lineSpacing',
  checklist: 'checklist',
  clearFormatting: 'eraser',
  bulletList: 'bulletList',
  numberList: 'numberList',
  alignLeft: 'alignLeft',
  alignCenter: 'alignCenter',
  alignRight: 'alignRight',
  alignJustify: 'alignJustify',
} as const satisfies Record<string, IconName>;

export type ActionId = keyof typeof ACTION_ICONS;

const MESSAGE_CONTEXT: readonly ActionId[] = ['open', 'markRead', 'flag', 'pin', 'reply', 'replyAll', 'forward', 'openReplyDraft', 'goReply', 'goForward', 'attachment', 'downloadAll', 'copy', 'print', 'search', 'focusFocused', 'focusOther', 'archive', 'spam', 'trash', 'markAnswered', 'markForwarded', 'snooze', 'unsnooze', 'followUp', 'label', 'export', 'saveSender', 'saveContacts'];
/** An action's other state, shown when it is already on. */
const TOGGLED: Partial<Record<ActionId, ActionId>> = { markRead: 'markUnread', flag: 'unflag', pin: 'unpin', saveSender: 'openContact' };

/** The actions each menu, toolbar or bar shows at once. Two actions listed together never share an icon (a toggle's two states count as one action). */
export const ACTION_MENUS: Readonly<Record<string, readonly ActionId[]>> = {
  'message-context': MESSAGE_CONTEXT,
  'message-context-toggled': MESSAGE_CONTEXT.map((id) => TOGGLED[id] ?? id),
  'folder-mark-read': ['markRead', 'markFolderRead'],
  'compose-format': ['fontGrow', 'fontShrink', 'bulletList', 'numberList', 'checklist', 'indentMore', 'indentLess', 'clearFormatting', 'copyFormat', 'applyFormat', 'alignLeft', 'alignCenter', 'alignRight', 'alignJustify', 'lineSpacing', 'textLtr', 'textRtl'],
  'message-copy': ['copyTo', 'copySender', 'copySubject'],
  'message-search': ['searchSender', 'searchDomain', 'searchSubject', 'searchFolder'],
  'snooze': ['snoozeLater', 'snoozeTomorrow', 'snoozeNextWeek', 'unsnooze'],
  'snooze-full': ['snoozeLater', 'snoozeEvening', 'snoozeTomorrow', 'snoozeWeekend', 'snoozeNextWeek', 'snoozeMonth', 'snoozePick', 'unsnooze'],
  'message-row': ['pin', 'snooze', 'label', 'followUp', 'archive', 'trash'],
  'reader-toolbar': ['reply', 'replyAll', 'forward', 'openReplyDraft', 'conversation', 'links', 'search', 'pin', 'markRead', 'flag', 'archive', 'moveTo', 'trash', 'spam', 'export', 'print', 'sweep', 'headers'],
  'mail-selection-bar': ['archive', 'trash', 'moveTo', 'markRead', 'flag', 'spam', 'label', 'forwardAttachment', 'download', 'downloadAll'],
  'folder-bulk-bar': ['markRead', 'archive', 'trash', 'moveTo'],
  'folder-context': ['openFolder', 'openFolderTab', 'markSelectedFoldersRead', 'markFolderRead', 'pin', 'folderStar', 'folderLayout', 'arrangeFolders', 'refresh', 'newFolder', 'renameFolder', 'cleanupFolder', 'emptyFolder', 'folderSettings', 'retention', 'hideFolder', 'sharing', 'moreFolderActions'],
  'folder-more': ['deleteFolder', 'copyAllMessages', 'moveAllMessages', 'moveFolderUp', 'moveFolderDown'],
  'outbox-context': ['outboxOpen', 'outboxSendNow', 'outboxSendAgain', 'outboxEditTime', 'outboxRecall', 'outboxEdit', 'outboxDelete'],
  // The set drawn but not yet in a menu: it must still be pairwise distinct.
  'reserve': ['duplicate', 'copy', 'copyLink', 'copyId', 'copyText', 'copyPath', 'copyHeaders', 'copySender', 'copySubject', 'openFolderTab', 'openWindow', 'openSide', 'viewSource', 'trash', 'deletePermanently', 'emptyFolder', 'cleanupFolder', 'sweep', 'archive', 'moveTo', 'copyTo', 'export', 'download', 'downloadAll', 'forwardAttachment', 'redirect', 'markAllRead', 'markUnread', 'pin', 'unpin', 'folderStar', 'flag', 'unflag', 'mute', 'blockSender', 'reportSpam', 'notSpam', 'reportPhishing', 'createFilter', 'properties', 'refresh', 'expandAll', 'collapseAll', 'selectAll', 'clearSelection', 'print', 'snooze', 'snoozeLater', 'snoozeEvening', 'snoozeTomorrow', 'snoozeWeekend', 'snoozeNextWeek', 'snoozeMonth', 'snoozePick', 'unsnooze', 'snoozedFolder', 'remindNoReply'],
};

/** The snooze presets' icons, by the catalog id snoozePresets() gives each. */
export const SNOOZE_PRESET_ICONS: Readonly<Record<string, IconName>> = {
  'mail-snooze-later': ACTION_ICONS.snoozeLater,
  'mail-snooze-evening': ACTION_ICONS.snoozeEvening,
  'mail-snooze-tomorrow': ACTION_ICONS.snoozeTomorrow,
  'mail-snooze-weekend': ACTION_ICONS.snoozeWeekend,
  'mail-snooze-next-week': ACTION_ICONS.snoozeNextWeek,
  'mail-snooze-month': ACTION_ICONS.snoozeMonth,
  'mail-snooze-pick': ACTION_ICONS.snoozePick,
  'snooze-kind-hours': ACTION_ICONS.snoozeLater,
  'snooze-kind-days': ACTION_ICONS.snoozeTomorrow,
  'mail-snooze-end-week': ACTION_ICONS.snoozeWeekend,
  'mail-snooze-end-month': ACTION_ICONS.snoozeMonth,
  'mail-snooze-next-month': ACTION_ICONS.snoozeMonth,
};

export function snoozePresetIcon(labelId: string): IconName {
  return SNOOZE_PRESET_ICONS[labelId] ?? ACTION_ICONS.snooze;
}
