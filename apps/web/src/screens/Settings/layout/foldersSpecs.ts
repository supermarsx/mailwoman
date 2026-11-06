// Search-index entries for the Folders preference cards (static: the components
// are heavy and the folder list needs a mailbox to render). The ids are the ids
// of the elements that carry each row's label / control.
import type { CardSpec } from './rows.tsx';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('folder-dnd');

export const FOLDER_PREFERENCE_CARDS: readonly CardSpec[] = [
  { id: 'folders-card-display', section: 'folders', title: 'settings-folders-title', help: 'settings-folders-help', rows: [
    { id: 'folders-collapse-user', label: 'settings-folders-collapse-user', help: 'settings-folders-hidden-help' },
  ] },
  { id: 'folders-arrange-card', section: 'folders', title: 'folder-dnd-arrange-title', help: 'folder-dnd-arrange-help', rows: [
    { id: 'folders-arrange-enabled', label: 'folder-dnd-reorder-enable' },
    { id: 'folders-arrange-nest', label: 'folder-dnd-nest-mode' },
    { id: 'folders-arrange-placement', label: 'folder-dnd-new-placement' },
    { id: 'folders-arrange-handles', label: 'folder-dnd-handles' },
  ] },
  { id: 'folders-card-roles', section: 'folders', title: 'settings-folder-role-title', help: 'settings-folder-role-help', rows: (['inbox', 'drafts', 'sent', 'trash', 'junk', 'archive'] as const).map(role => ({ id: `folder-role-${role}`, label: `settings-folder-role-${role}` })) },
  { id: 'folders-card-list', section: 'folders', title: 'settings-folders-list-card', help: 'settings-folders-order-help', rows: [] },
];
