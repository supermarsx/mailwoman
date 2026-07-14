// V7 Nextcloud attach/save/share (SPEC §18.4, plan §3 e7). SCAFFOLD stub (e0):
// inert placeholder. e7 builds the WebDAV picker + save + share-link composer; e14
// wires it to /api/nextcloud/*.

import type { JSX } from 'solid-js';

export interface NextcloudAttachProps {
  accountId?: string;
}

export function NextcloudAttach(_props: NextcloudAttachProps): JSX.Element {
  return <div data-module="nextcloud">Nextcloud attach/save/share not yet implemented (t7 e7).</div>;
}
