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
  | 'dashboard'
  | 'users'
  | 'domains'
  | 'twofactor'
  | 'access'
  | 'sso'
  | 'settings'
  | 'assistant'
  | 'integrations'
  | 'plugins'
  | 'uiPlugins'
  | 'egress'
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
  | 'chevron'
  | 'check'
  | 'alert'
  | 'info'
  | 'eye'
  | 'eyeOff'
  | 'policy'
  | 'oauthClients';

/** Icons whose meaning depends on reading direction. */
export const DIRECTIONAL: ReadonlySet<IconName> = new Set<IconName>(['signOut', 'egress', 'chevron']);

const S = { fill: 'none', stroke: 'currentColor', 'stroke-width': '2' } as const;
const F = { fill: 'currentColor' } as const;

const PATHS: Record<IconName, () => JSX.Element> = {
  // Two filled blocks, a ring and a triangle: the four tiles of an overview.
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
