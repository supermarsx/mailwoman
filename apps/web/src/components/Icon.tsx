// Admin sidebar and control icons (t32-admin.md §2.1, scope F).
//
// One file, no icon library. Each icon is a 16×16 inline SVG drawn from squares,
// circles, triangles and 2 px lines, in `currentColor`, so it takes the text
// colour of the item it sits in (including the selected and hover states).
// Icons are decorative: `aria-hidden="true"`, the text label next to them names
// the item. Glyphs that point a direction carry `icon--directional` and are
// mirrored under `[dir=rtl]` (shell.css.ts).

import type { JSX } from 'solid-js';

export type IconName =
  | 'tableRowBefore' | 'tableRowAfter' | 'tableRowDelete' | 'tableColumnBefore' | 'tableColumnAfter' | 'tableColumnDelete' | 'tableMerge' | 'tableSplit' | 'tableHeader'
  | 'undo' | 'redo'
  | 'cut' | 'paste' | 'table' | 'paragraph' | 'paint' | 'eyedropper'
  | 'home' | 'briefcase' | 'heart' | 'shopping' | 'education' | 'travel' | 'coffee' | 'gift' | 'bank' | 'book'
  | 'identityProvider'
  | 'shield'
  | 'fileAudio' | 'fileVideo' | 'filePdf' | 'fileCode'
  | 'print'
  | 'download'
  | 'preferences'
  | 'dashboard'
  | 'analytics'
  | 'nextcloud'
  | 'sessions'
  | 'admins'
  | 'roles'
  | 'users'
  | 'domains'
  | 'twofactor'
  | 'access'
  | 'sso'
  | 'settings'
  | 'configuration'
  | 'diagnostics'
  | 'assistant'
  | 'integrations'
  | 'plugins'
  | 'uiPlugins'
  | 'egress'
  | 'privacy'
  | 'observability'
  | 'audit'
  | 'maintenance'
  | 'serverMetadata'
  | 'logs'
  | 'branding'
  | 'signOut'
  | 'menu'
  | 'close'
  | 'search'
  | 'filter'
  | 'chevron'
  | 'check'
  | 'alert'
  | 'info'
  | 'eye'
  | 'eyeOff'
  | 'policy'
  | 'oauthClients'
  | 'restore'
  | 'edit'
  | 'delete'
  | 'clone'
  | 'enable'
  | 'disable'
  | 'inbox'
  | 'sent'
  | 'mail'
  | 'flag'
  | 'pin'
  | 'attachment'
  | 'archive'
  | 'clock'
  | 'tag'
  | 'link'
  | 'image'
  | 'folder'
  | 'file'
  | 'lock'
  | 'unlock'
  | 'star'
  | 'starOutline'
  | 'chevronDown'
  | 'chevronUp'
  | 'replyAll' | 'mailRead' | 'spam' | 'sweep'
  | 'reply'
  | 'forward'
  | 'calendar'
  | 'minus'
  | 'plus'
  | 'sync'
  | 'more'
  | 'conversation'
  | 'sort'
  | 'quote'
  | 'clearFormat'
  | 'bold'
  | 'italic'
  | 'underline'
  | 'strike'
  | 'bulletList'
  | 'numberList'
  | 'heading'
  | 'busy'
  | 'tentative'
  | 'chevronLeft'
  | 'back'
  | 'microphone'
  | 'refresh';

/** Icons whose meaning depends on reading direction. */
export const DIRECTIONAL: ReadonlySet<IconName> = new Set<IconName>(['signOut', 'egress', 'chevron', 'chevronLeft', 'back', 'reply', 'replyAll', 'forward', 'undo', 'redo']);

const S = { fill: 'none', stroke: 'currentColor', 'stroke-width': '2' } as const;
const F = { fill: 'currentColor' } as const;

const PATHS: Record<IconName, () => JSX.Element> = {
  filter: () => <path d="M2 2 H14 L9 8 V13 L7 14 V8 Z" {...S} />,
  home: () => <><path d="M1 7l7-6 7 6M3 6v9h10V6M6 15v-5h4v5" {...S}/></>,
  briefcase: () => <><rect x="1" y="5" width="14" height="10" {...S}/><path d="M5 5V2h6v3M1 9h14M7 8v3h2V8" {...S}/></>,
  heart: () => <path d="M8 14L2 8C-2 3 5-1 8 4c3-5 10-1 6 4z" {...S}/>,
  shopping: () => <><path d="M3 5h10l1 10H2zM5 6V4a3 3 0 016 0v2" {...S}/></>,
  education: () => <><path d="M1 5l7-4 7 4-7 4zM4 7v5c3 3 5 3 8 0V7M15 5v7" {...S}/></>,
  travel: () => <><path d="M1 8l5-1 3-6h2L9 7l6 2v2l-6-1-1 5H6l-1-5-4-1z" {...S}/></>,
  coffee: () => <><path d="M2 5h9v5a4 4 0 01-8 0V5M11 6h2a2 2 0 010 4h-2M1 15h13M4 1v2M8 1v2" {...S}/></>,
  gift: () => <><rect x="1" y="6" width="14" height="3" {...S}/><path d="M2 9v6h12V9M8 6v9M8 6C0 6 3-2 8 4c5-6 8 2 0 2z" {...S}/></>,
  bank: () => <><path d="M1 5l7-4 7 4zM3 7v5M8 7v5M13 7v5M1 15h14" {...S}/></>,
  book: () => <><path d="M8 3C5 1 2 1 1 2v12c2-1 5-1 7 1 2-2 5-2 7-1V2c-1-1-4-1-7 1zM8 3v12" {...S}/></>,
  identityProvider: () => <><rect x="1" y="2" width="14" height="12" {...S}/><circle cx="5" cy="6" r="2" {...S}/><path d="M2 12v-1a3 3 0 016 0v1M10 6h3M10 9h3" {...S}/></>,
  replyAll: () => <><path d="M6 3L2 7l4 4M10 3L6 7l4 4M6 7h5c3 0 4 2 4 6" {...S}/></>,
  mailRead: () => <><path d="M1 6l7-5 7 5v9H1zM1 6l7 5 7-5" {...S}/></>,
  spam: () => <><path d="M5 1h6l4 4v6l-4 4H5l-4-4V5zM8 4v5M8 11v1" {...S}/></>,
  sweep: () => <><path d="M10 6l5-5M2 9l5-5 5 5-5 5H1zM5 9l-3 3M8 10l-3 3" {...S}/></>,
  download: () => <path d="M8 1v9m-4-4 4 4 4-4M2 11v3h12v-3" {...S}/>,
  shield: () => <><path d="M8 1L2 3v5c0 3 3 5 6 7 3-2 6-4 6-7V3z" {...S}/><path d="M5 8l2 2 4-4" {...S}/></>,
  fileAudio: () => <><path d="M6 12V3l7-2v9M6 6l7-2" {...S}/><ellipse cx="3.5" cy="12" rx="2.5" ry="2" {...S}/><ellipse cx="10.5" cy="10" rx="2.5" ry="2" {...S}/></>,
  fileVideo: () => <><rect x="1" y="3" width="10" height="10" {...S}/><path d="M11 6l4-2v8l-4-2M5 6l3 2-3 2z" {...S}/></>,
  filePdf: () => <><path d="M3 1h7l3 3v11H3zM10 1v4h3M5 8h6M5 11h5" {...S}/></>,
  fileCode: () => <><path d="M5 4L1 8l4 4M11 4l4 4-4 4M9 2L7 14" {...S}/></>,
  refresh: () => <path d="M13 6 A5.5 5.5 0 1 0 13 11 M13 2 V6 H9" {...S} />,
  microphone: () => <path d="M5 4 A3 3 0 0 1 11 4 V7 A3 3 0 0 1 5 7 Z M2 6 V7 A6 6 0 0 0 14 7 V6 M8 13 V15 M5 15 H11" {...S} />,
  back: () => <path d="M7 2 L1 8 L7 14 M1 8 H15" {...S} />,
  chevronLeft: () => <path d="M10 3 L5 8 L10 13" {...S} />,
  more: () => <path d="M2 8 H4 M7 8 H9 M12 8 H14" {...S} />,
  conversation: () => <><path d="M1 2 H10 V8 H5 L2 10.5 V8 H1 Z" {...S}/><path d="M12.5 5 H15 V12 H14 V14.5 L11 12 H6 V10.5" {...S}/></>,
  sort: () => <path d="M3 5 L8 1 L13 5 M3 11 L8 15 L13 11" {...S} />,
  quote: () => <path d="M2 3 H7 V9 H4 L2 13 V3 M10 3 H15 V9 H12 L10 13 V3" {...S} />,
  clearFormat: () => <path d="M2 2 H14 M8 2 V12 M3 12 H10 M11 10 L15 14 M15 10 L11 14" {...S} />,
  bold: () => <path d="M4 1 H9 A3 3 0 0 1 9 7 H4 V1 M4 7 H10 A4 4 0 0 1 10 15 H4 Z" {...S} />,
  italic: () => <path d="M7 1 H14 M2 15 H9 M11 1 L5 15" {...S} />,
  underline: () => <path d="M3 1 V8 A5 5 0 0 0 13 8 V1 M2 15 H14" {...S} />,
  strike: () => <path d="M13 3 Q8 -1 4 3 Q1 6 8 8 Q15 10 12 13 Q8 17 3 13 M1 8 H15" {...S} />,
  bulletList: () => <path d="M1 3 H3 M6 3 H15 M1 8 H3 M6 8 H15 M1 13 H3 M6 13 H15" {...S} />,
  numberList: () => <path d="M1 2 H3 V6 M1 6 H4 M1 10 Q4 8 4 11 L1 14 H4 M7 3 H15 M7 12 H15" {...S} />,
  heading: () => <path d="M3 1 V15 M13 1 V15 M3 8 H13" {...S} />,
  busy: () => <path d="M8 1 A7 7 0 1 1 7.99 1 M3 8 H13" {...S} />,
  tentative: () => <path d="M8 1 A7 7 0 1 1 7.99 1 M8 4 V9 M8 11 V13" {...S} />,

  tableRowBefore: () => <path d="M1 6 H15 V15 H1 Z M1 10 H15 M8 6 V15 M5 2 H11 M8 0 V4" {...S}/>,
  tableRowAfter: () => <path d="M1 1 H15 V10 H1 Z M1 5 H15 M8 1 V10 M5 14 H11 M8 12 V16" {...S}/>,
  tableRowDelete: () => <path d="M1 2 H15 V14 H1 Z M1 6 H15 M1 10 H15 M6 7 L10 9 M10 7 L6 9" {...S}/>,
  tableColumnBefore: () => <path d="M6 1 H15 V15 H6 Z M10 1 V15 M6 8 H15 M2 5 V11 M0 8 H4" {...S}/>,
  tableColumnAfter: () => <path d="M1 1 H10 V15 H1 Z M5 1 V15 M1 8 H10 M14 5 V11 M12 8 H16" {...S}/>,
  tableColumnDelete: () => <path d="M2 1 H14 V15 H2 Z M6 1 V15 M10 1 V15 M7 6 L9 10 M9 6 L7 10" {...S}/>,
  tableMerge: () => <path d="M1 2 H15 V14 H1 Z M1 8 H15 M3 5 H7 L5 3 M7 5 L5 7 M13 5 H9 L11 3 M9 5 L11 7" {...S}/>,
  tableSplit: () => <path d="M1 2 H15 V14 H1 Z M8 2 V14 M3 8 H6 M3 8 L5 6 M3 8 L5 10 M13 8 H10 M13 8 L11 6 M13 8 L11 10" {...S}/>,
  tableHeader: () => <><path d="M1 2 H15 V14 H1 Z M1 7 H15 M6 7 V14 M11 7 V14" {...S}/><path d="M3 4 H13 V5 H3 Z" fill="currentColor"/></>,
  cut: () => <><circle cx="4" cy="12" r="2" {...S}/><circle cx="12" cy="12" r="2" {...S}/><path d="M5 10 L13 1 M11 10 L3 1" {...S}/></>,
  paste: () => <><path d="M5 3 H2 V15 H14 V3 H11 M5 7 H11 M5 10 H11" {...S}/><path d="M5 1 H11 V4 H5 Z" {...S}/></>,
  table: () => <path d="M1 2 H15 V14 H1 Z M1 6 H15 M6 2 V14 M11 6 V14 M1 10 H15" {...S}/>,
  paragraph: () => <path d="M13 2 H6 A3 3 0 0 0 6 8 H8 M8 2 V15 M12 2 V15" {...S}/>,
  paint: () => <path d="M3 1 H13 V5 H3 Z M13 3 H15 V8 H8 V11 M6 11 H10 V15 H6 Z" {...S}/>,
  // A pipette: the bulb at the top end, the glass tube down to the tip.
  eyedropper: () => <path d="M10 2 L14 6 M9 3 L13 7 M11 5 L4 12 L3 15 L6 14 L13 7" {...S}/>,
  delete: () => <path d="M3 4 H13 M6 1 H10 M4 4 L5 15 H11 L12 4 M7 7 V12 M9 7 V12" {...S} />,
  clone: () => <path d="M5 5 H14 V14 H5 Z M2 11 V2 H11" {...S} />,
  enable: () => <path d="M3 8 L6 11 L13 4" {...S} />,
  disable: () => <path d="M3 3 L13 13 M13 3 L3 13" {...S} />,
  inbox: () => <path d="M3 2 H13 L15 9 V14 H1 V9 Z M1 9 H5 L6 11 H10 L11 9 H15" {...S} />,
  sent: () => <path d="M1 2 L15 8 L1 14 L4 8 Z M4 8 H15" {...S} />,
  mail: () => <path d="M1 3 H15 V13 H1 Z M1 3 L8 9 L15 3" {...S} />,
  flag: () => <path d="M3 15 V1 H13 L10 4 L13 7 H3" {...S} />,
  pin: () => <path d="M5 1 H11 L10 6 L13 9 H9 V15 H7 V9 H3 L6 6 Z" {...S} />,
  attachment: () => <path d="M6 6 L10 2 A3 3 0 0 1 14 6 L7 13 A4 4 0 0 1 1 7 L8 0 M5 9 L11 3" {...S} />,
  archive: () => <path d="M1 2 H15 V5 H1 Z M3 5 V14 H13 V5 M6 8 H10" {...S} />,
  clock: () => <path d="M8 1 A7 7 0 1 1 7.99 1 M8 4 V8 L11 10" {...S} />,
  tag: () => <path d="M1 1 H8 L15 8 L8 15 L1 8 Z M4 4 H5" {...S} />,
  link: () => <path d="M6 10 L10 6 M5 8 L2 11 A2 2 0 0 0 5 14 L8 11 M8 5 L11 2 A2 2 0 0 1 14 5 L11 8" {...S} />,
  image: () => <path d="M1 1 H15 V15 H1 Z M1 12 L5 8 L9 12 L12 7 L15 10 M4 4 H6" {...S} />,
  folder: () => <path d="M1 3 H6 L8 5 H15 V14 H1 Z" {...S} />,
  file: () => <path d="M3 1 H10 L14 5 V15 H3 Z M10 1 V5 H14" {...S} />,
  lock: () => <path d="M3 7 H13 V15 H3 Z M5 7 V4 A3 3 0 0 1 11 4 V7" {...S} />,
  unlock: () => <path d="M3 7 H13 V15 H3 Z M5 7 V4 A3 3 0 0 1 11 4" {...S} />,
  star: () => <path d="M8 1 L10 6 H15 L11 9 L13 15 L8 12 L3 15 L5 9 L1 6 H6 Z" {...S} fill="currentColor" />,
  starOutline: () => <path d="M8 1 L10 6 H15 L11 9 L13 15 L8 12 L3 15 L5 9 L1 6 H6 Z" {...S} />,
  chevronDown: () => <path d="M3 5 L8 10 L13 5" {...S} />,
  // A hooked arrow back (undo) and forward (redo); mirrored under RTL with the other directional glyphs.
  undo: () => <path d="M5 2 L1 6 L5 10 M1 6 H10 A4 4 0 0 1 10 14 H6" {...S} />,
  redo: () => <path d="M11 2 L15 6 L11 10 M15 6 H6 A4 4 0 0 0 6 14 H10" {...S} />,
  chevronUp: () => <path d="M3 11 L8 6 L13 11" {...S} />,
  reply: () => <path d="M6 3 L1 8 L6 13 M1 8 H10 Q14 8 14 13" {...S} />,
  forward: () => <path d="M10 3 L15 8 L10 13 M15 8 H6 Q2 8 2 13" {...S} />,
  calendar: () => <path d="M2 3 H14 V15 H2 Z M2 7 H14 M5 1 V5 M11 1 V5" {...S} />,
  minus: () => <path d="M2 8 H14" {...S} />,
  plus: () => <path d="M2 8 H14 M8 2 V14" {...S} />,
  sync: () => <path d="M2 7 V2 H7 M2 2 L14 14 M9 14 H14 V9" {...S} />,

  restore: () => <><path d="M3 5 A6 6 0 1 1 2 10 M3 1 V5 H7" {...S} /></>,
  edit: () => <><path d="M2 11 L11 2 L14 5 L5 14 H2 Z M9 4 L12 7" {...S} /></>,
  print: () => <g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 5V1h8v4M4 11H2V5h12v6h-2" /><path d="M4 9h8v6H4z" /><path d="M11 7h1" /></g>,
  preferences: () => <><circle cx="5" cy="4" r="2" {...S} /><path d="M1 14 V11 A4 4 0 0 1 9 11 V14 M11 3 H15 M11 8 H15 M11 13 H15" {...S} /></>,
  diagnostics: () => <><path d="M1 8 H4 L6 2 L10 14 L12 8 H15" {...S} /></>,
  configuration: () => <><rect x="2" y="1" width="12" height="14" {...S} /><path d="M5 5 H11 M5 8 H11 M5 11 H8" {...S} /></>,
  privacy: () => <><path d="M8 1 L14 3 V8 Q14 12 8 15 Q2 12 2 8 V3 Z" {...S} /><circle cx="8" cy="7" r="1" {...F} /><path d="M8 8 V11" {...S} /></>,
  sessions: () => <><rect x="1" y="2" width="10" height="8" {...S} /><path d="M4 14 H8 M6 10 V14" {...S} /><rect x="12" y="6" width="3" height="8" {...S} /></>,
  admins: () => <><circle cx="5" cy="4" r="2" {...S} /><path d="M1 14 V11 A4 4 0 0 1 9 11 V14 M11 7 L15 7 V11 L13 14 L11 11 Z" {...S} /></>,
  roles: () => <><path d="M1 3 H7 M1 8 H7 M1 13 H7" {...S} /><path d="M10 3 L12 5 L15 1 M10 8 L12 10 L15 6 M10 13 L12 15 L15 11" {...S} /></>,
  // Two filled blocks, a ring and a triangle: the four tiles of an overview.
  nextcloud: () => <><circle cx="8" cy="8" r="3" {...S} /><circle cx="2" cy="8" r="1.5" {...S} /><circle cx="14" cy="8" r="1.5" {...S} /><path d="M3.5 8H5 M11 8h1.5" {...S} /></>,
  analytics: () => <><path d="M1 1 V15 H15" {...S} /><path d="M4 11 V8 M8 11 V5 M12 11 V2" {...S} /></>,
  dashboard: () => (
    <>
      <rect x="1" y="1" width="6" height="6" {...F} />
      <circle cx="12" cy="4" r="2.5" {...S} />
      <rect x="10" y="10" width="5" height="5" {...S} />
      <path d="M1 15 L4 9 L7 15 Z" {...F} />
    </>
  ),
  users: () => (
    <>
      <circle cx="6" cy="4.5" r="3" {...F} />
      <path d="M1 15 A5 5 0 0 1 11 15 Z" {...F} />
      <circle cx="12.5" cy="6" r="2" {...S} />
      <path d="M12 15 L15 15 L15 12" {...S} />
    </>
  ),
  domains: () => (
    <>
      <circle cx="8" cy="8" r="6" {...S} />
      <path d="M2 8 H14 M8 2 V14" {...S} />
    </>
  ),
  twofactor: () => (
    <>
      <rect x="1" y="5" width="6" height="6" {...F} />
      <rect x="9" y="5" width="6" height="6" {...S} />
    </>
  ),
  access: () => (
    <>
      <path d="M3 1 V15 M13 1 V15" {...S} />
      <path d="M3 8 H13" {...S} />
      <rect x="6" y="3" width="4" height="3" {...F} />
    </>
  ),
  sso: () => (
    <>
      <circle cx="5" cy="8" r="3" {...S} />
      <path d="M8 8 H15 M12 8 V11 M15 8 V11" {...S} />
    </>
  ),
  settings: () => (
    <>
      <path d="M1 4 H15 M1 12 H15" {...S} />
      <rect x="9" y="1.5" width="4" height="5" {...F} />
      <rect x="3" y="9.5" width="4" height="5" {...F} />
    </>
  ),
  assistant: () => (
    <>
      <circle cx="7" cy="9" r="5" {...S} />
      <circle cx="7" cy="9" r="1.5" {...F} />
      <rect x="12" y="1" width="3" height="3" {...F} />
    </>
  ),
  integrations: () => (
    <>
      <circle cx="4" cy="8" r="3" {...S} />
      <circle cx="12" cy="8" r="3" {...F} />
      <path d="M7 8 H9" {...S} />
    </>
  ),
  plugins: () => (
    <>
      <rect x="2" y="5" width="10" height="10" {...S} />
      <rect x="9" y="1" width="6" height="6" {...F} />
    </>
  ),
  uiPlugins: () => (
    <>
      <rect x="1" y="2" width="14" height="12" {...S} />
      <rect x="1" y="2" width="14" height="4" {...F} />
      <rect x="4" y="9" width="3" height="3" {...F} />
    </>
  ),
  egress: () => (
    <>
      <path d="M9 2 H2 V14 H9" {...S} />
      <path d="M6 8 H15 M12 5 L15 8 L12 11" {...S} />
    </>
  ),
  observability: () => (
    <>
      <rect x="1" y="9" width="3" height="6" {...F} />
      <rect x="6.5" y="5" width="3" height="10" {...F} />
      <rect x="12" y="1" width="3" height="14" {...S} />
    </>
  ),
  audit: () => (
    <>
      <rect x="1" y="2" width="3" height="3" {...F} />
      <rect x="1" y="11" width="3" height="3" {...F} />
      <path d="M7 3.5 H15 M7 8 H15 M7 12.5 H15" {...S} />
      <circle cx="2.5" cy="8" r="1.5" {...F} />
    </>
  ),
  maintenance: () => (
    <>
      <circle cx="8" cy="8" r="3" {...S} />
      <path d="M8 1 V4 M8 12 V15 M1 8 H4 M12 8 H15" {...S} />
    </>
  ),
  serverMetadata: () => (
    <>
      <rect x="1" y="2" width="14" height="5" {...S} />
      <rect x="1" y="9" width="14" height="5" {...S} />
      <rect x="3" y="4" width="2" height="1" {...F} />
      <rect x="3" y="11" width="2" height="1" {...F} />
    </>
  ),
  logs: () => (
    <>
      <path d="M1 3 H15 M1 8 H11 M1 13 H13" {...S} />
    </>
  ),
  branding: () => (
    <>
      <path d="M1 15 L6 6 L11 15 Z" {...F} />
      <circle cx="11.5" cy="4.5" r="3" {...S} />
    </>
  ),
  signOut: () => (
    <>
      <path d="M7 2 H2 V14 H7" {...S} />
      <path d="M5 8 H14 M11 5 L14 8 L11 11" {...S} />
    </>
  ),
  menu: () => <path d="M1 3 H15 M1 8 H15 M1 13 H15" {...S} />,
  close: () => <path d="M3 3 L13 13 M13 3 L3 13" {...S} />,
  search: () => (
    <>
      <circle cx="6.5" cy="6.5" r="4.5" {...S} />
      <path d="M10 10 L15 15" {...S} />
    </>
  ),
  chevron: () => <path d="M6 3 L11 8 L6 13" {...S} />,
  check: () => <path d="M2 8 L6 12 L14 4" {...S} />,
  alert: () => (
    <>
      <path d="M8 1 L15 14 H1 Z" {...S} />
      <path d="M8 6 V9" {...S} />
      <rect x="7" y="11" width="2" height="2" {...F} />
    </>
  ),
  info: () => (
    <>
      <circle cx="8" cy="8" r="6.5" {...S} />
      <path d="M8 7 V12" {...S} />
      <rect x="7" y="3.5" width="2" height="2" {...F} />
    </>
  ),
  // Show and hide a password (components/PasswordInput.tsx): an open eye, and the
  // same eye struck through.
  eye: () => (
    <>
      <path d="M1 8 Q8 1 15 8 Q8 15 1 8 Z" {...S} />
      <circle cx="8" cy="8" r="2" {...F} />
    </>
  ),
  eyeOff: () => (
    <>
      <path d="M1 8 Q8 1 15 8 Q8 15 1 8 Z" {...S} />
      <circle cx="8" cy="8" r="2" {...F} />
      <path d="M2 14 L14 2" {...S} />
    </>
  ),
  // A shield with a filled lower half: the rules the deployment enforces.
  policy: () => (
    <>
      <path d="M8 1 L14 3 V8 Q14 13 8 15 Q2 13 2 8 V3 Z" {...S} />
      <path d="M2.5 9 H13.5 Q12.5 13 8 14.5 Q3.5 13 2.5 9 Z" {...F} />
    </>
  ),
  // A key: a filled bow and a stroked shaft with two teeth, for registered clients.
  oauthClients: () => (
    <>
      <circle cx="4.5" cy="8" r="3.5" {...F} />
      <path d="M8 8 H15 M12 8 V11 M15 8 V10" {...S} />
    </>
  ),
};

/** Every icon name, for tests and the section table. */
export const ICON_NAMES = Object.keys(PATHS) as IconName[];

export interface IconProps {
  name: IconName;
  class?: string;
}

export function Icon(props: IconProps): JSX.Element {
  return (
    <svg
      class={[DIRECTIONAL.has(props.name) ? 'icon--directional' : '', props.class ?? ''].join(' ').trim() || undefined}
      width="16"
      height="16"
      viewBox="0 0 16 16"
      aria-hidden="true"
      data-icon={props.name}
    >
      {PATHS[props.name]()}
    </svg>
  );
}

/** Persisted tag metadata may predate SVG names; unsupported custom glyphs use a tag symbol. */
export function tagIconName(value: string | undefined): IconName {
  const legacy: Record<string, IconName> = { '\u{1f4bc}': 'archive', '\u{1f3e0}': 'users', '\u2b50': 'star', '\u{1f552}': 'clock', '\u{1f9fe}': 'file', '\u{1f3f7}\ufe0f': 'tag' };
  return legacy[value ?? ''] ?? (ICON_NAMES.includes(value as IconName) ? value as IconName : 'tag');
}
