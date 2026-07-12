// FROZEN attachment-viewer contract (plan §2.4, SPEC §10.2). Implemented by e8
// (viewers/** + screens/Attachments.tsx). `<AttachmentViewer>` dispatches by
// `mime` to a per-type viewer, each rendered inside a sandboxed container
// (image/text/audio/video in `<iframe sandbox>` + per-message CSP; PDF via
// pdfjs canvas in a sandboxed frame, worker self-hosted). All lazy-loaded so
// pdfjs stays off the login→inbox critical path (§23 bundle gate).

import type { EmailBodyPart } from '../api/jmap-types.ts';

/** Props every concrete viewer receives (frozen §2.4). */
export interface ViewerProps {
  part: EmailBodyPart;
  /** An object URL for the (already-fetched) attachment blob. */
  blobUrl: string;
  mime: string;
  name: string;
}

/** The viewer variants `<AttachmentViewer>` dispatches to (plan §2.4). */
export type ViewerKind = 'image' | 'pdf' | 'text' | 'audio' | 'video' | 'unsupported';

/** Frozen mime→viewer routing helper so dispatch is identical everywhere. */
export function viewerKindFor(mime: string): ViewerKind {
  if (mime === 'application/pdf') return 'pdf';
  if (mime.startsWith('image/')) return 'image';
  if (mime.startsWith('audio/')) return 'audio';
  if (mime.startsWith('video/')) return 'video';
  if (mime.startsWith('text/')) return 'text';
  return 'unsupported';
}
