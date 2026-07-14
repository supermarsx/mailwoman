// V7 Nextcloud module (SPEC §18.4, plan §2.6 / §3 e7). SCAFFOLD stub (e0): inert,
// lazily importable, typecheck-green, NOT routed. e7 fills attach-from-Nextcloud,
// save-attachment-to-Nextcloud, and large-attachment share links (optional
// password/expiry); e14 wires it to /api/nextcloud/*.

/** A created share link (mirrors the server's Nextcloud OCS response projection). */
export interface ShareLink {
  readonly url: string;
  readonly passwordProtected: boolean;
  readonly expiresAt: string | null;
}

export { NextcloudAttach } from './NextcloudAttach.tsx';
