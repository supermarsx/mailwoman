// V7 Nextcloud module (SPEC §18.4, plan §2.6 / §3 e7). Attach-from / save-to / share-link.
// Lazily importable; NOT routed by this module (ownership boundary — e14 wires the
// components into the composer + attachment menus). CalDAV/CardDAV/tasks are core
// (`mw-dav`); this module is files + share links only.
//
// e14 WIRE-UP (import paths):
//   import { NextcloudAttach }    from './modules/nextcloud/index.ts'  — attach menu
//   import { SaveToNextcloud }    from './modules/nextcloud/index.ts'  — attachment menu
//   import { ShareLinkComposer }  from './modules/nextcloud/index.ts'  — large-attachment path
// Endpoints this module calls (e9 to satisfy, e14 to mount):
//   GET  /api/nextcloud/list?path=      → { entries: WebDavEntry[] }
//   POST /api/nextcloud/attach          → { attachments: AttachedFile[] }
//   POST /api/nextcloud/save            → { entry: WebDavEntry }
//   POST /api/nextcloud/share-link      → ShareLink

export { NextcloudAttach, type NextcloudAttachProps } from './NextcloudAttach.tsx';
export { SaveToNextcloud, type SaveToNextcloudProps } from './SaveToNextcloud.tsx';
export { ShareLinkComposer, type ShareLinkComposerProps } from './ShareLinkComposer.tsx';
export { FilePicker, type FilePickerProps } from './FilePicker.tsx';
export {
  NextcloudService,
  type Fetcher,
  type WebDavEntry,
  type AttachedFile,
  type ShareLink,
  type ShareLinkOptions,
} from './service.ts';
