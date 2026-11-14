// FROZEN attachment-viewer contract (plan §2.4, SPEC §10.2). Implemented by e8
// (viewers/** + screens/Attachments.tsx). `<AttachmentViewer>` dispatches by
// `mime` to a per-type viewer, each rendered inside a sandboxed container
// (image/text/audio/video in `<iframe sandbox>` + per-message CSP; PDF via
// PDF.js's full self-hosted viewer in a separately secured frame). All lazy-loaded so
// pdfjs stays off the login→inbox critical path (§23 bundle gate).

import type { EmailBodyPart } from '../api/jmap-types.ts';

/** Props every concrete viewer receives (frozen §2.4). */
export interface ViewerProps {
  part: EmailBodyPart;
  /** An object URL for the (already-fetched) attachment blob. */
  blobUrl: string;
  mime: string;
  name: string;
  /** Optional: told what the viewer learned about the document itself (the File details dialog lists it). */
  onDetails?: (details: ViewerDetails) => void;
}

/** Facts about a document that only its viewer can read. Every field is optional: a viewer reports what it has. */
export interface ViewerDetails {
  pages?: number;
  producer?: string;
  creator?: string;
  /** ISO 8601. */
  created?: string;
  pdfVersion?: string;
  /** A password was asked for before the document opened. */
  passwordRequired?: boolean;
}

/** The viewer variants `<AttachmentViewer>` dispatches to (plan §2.4). */
export type ViewerKind = 'image' | 'pdf' | 'text' | 'audio' | 'video' | 'message' | 'calendar' | 'html' | 'code' | 'unsupported';

/** Frozen mime→viewer routing helper so dispatch is identical everywhere. */
export function viewerKindFor(mime: string, name = ''): ViewerKind {
  mime = mime.toLowerCase().split(';')[0]!.trim();
  if (['message/rfc822', 'message/global', 'application/ms-tnef', 'application/vnd.ms-tnef'].includes(mime) || ((!mime || mime === 'application/octet-stream') && /(?:\.eml|winmail\.dat)$/i.test(name))) return 'message';
  // t53: calendar data renders as the event card (with the raw text as its fallback).
  if (['text/calendar', 'application/ics', 'text/x-vcalendar'].includes(mime) || ((!mime || mime === 'application/octet-stream' || mime === 'text/plain') && /\.(?:ics|ical|ifb)$/i.test(name))) return 'calendar';
  const generic=!mime||mime==='application/octet-stream'||mime==='binary/octet-stream';
  if (mime==='text/html'||mime==='application/xhtml+xml'||generic&&/\.html?$/i.test(name)) return 'html';
  if (['application/json','application/ld+json','application/xml','application/javascript','application/ecmascript','application/x-javascript','application/yaml','application/x-yaml','text/yaml'].includes(mime)||(generic||mime==='text/plain')&&/\.(?:c|h|cpp|hpp|cc|cs|rs|go|py|pyw|js|jsx|ts|tsx|mjs|cjs|java|kt|kts|swift|rb|php|pl|pm|r|sql|sh|bash|zsh|fish|ps1|psm1|bat|cmd|vbs|lua|scala|vue|svelte|css|scss|sass|less|json|jsonc|json5|xml|xsd|xsl|yaml|yml|toml|ini|cfg|conf|env|properties|dockerfile|makefile)$/i.test(name)||generic&&/^(?:Dockerfile|Makefile|\.gitignore|\.env)$/i.test(name)) return 'code';
  if (generic&&/\.(?:txt|text|log|md|markdown|csv|tsv|rst|nfo)$/i.test(name)) return 'text';
  if (mime === 'application/pdf'||generic&&/\.pdf$/i.test(name)) return 'pdf';
  if(generic&&/\.(?:png|apng|jpe?g|jfif|gif|webp|avif|svg|bmp|ico|cur|tiff?|heic|heif|jxl|jp2|j2k|psd|dds|tga|pnm|ppm|pgm|pbm|qoi|exr|hdr|ff)$/i.test(name))return 'image';
  if (mime.startsWith('image/')) return 'image';
  if (mime.startsWith('audio/')) return 'audio';
  if (mime.startsWith('video/')) return 'video';
  if (mime.startsWith('text/')) return 'text';
  return 'unsupported';
}
