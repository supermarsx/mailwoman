// What the "File details" dialog shows, and the helpers that format it. Kept
// apart from the component so the fingerprint grouping and the PDF date parsing
// are unit-testable without a DOM.
import type { ViewerDetails } from '../contracts/viewer.ts';

/** Everything the dialog lists. `extra` is what the open viewer reported about the document itself (a PDF's page count, producer...). */
export interface AttachmentDetails {
  name: string;
  /** The MIME type the message declared. */
  mime: string;
  size: number;
  /** Lowercase hex SHA-256 of the bytes, or null when it could not be computed. */
  digest: string | null;
  /** A description of the file type, from its extension. */
  type: string;
  extension: string;
  /** The type the file's own leading bytes name, when recognised. */
  detected: string | null;
  extra?: ViewerDetails | undefined;
}

/** The algorithm the fingerprint uses (shown next to it). */
export const FINGERPRINT_ALGORITHM = 'SHA-256';

/** `abcdef0123…` as `abcdef01 23…`: groups of `size` characters, for reading aloud and comparing by eye. */
export function groupHex(hex: string, size = 8): string[] {
  const out: string[] = [];
  for (let i = 0; i < hex.length; i += size) out.push(hex.slice(i, i + size));
  return out;
}

/** A PDF date string (`D:20240131120000+01'00'`) as an ISO 8601 timestamp, or null when it is not one. */
export function parsePdfDate(raw: unknown): string | null {
  if (typeof raw !== 'string') return null;
  const m = /^D:(\d{4})(\d{2})?(\d{2})?(\d{2})?(\d{2})?(\d{2})?(?:(Z)|([+-])(\d{2})(?:'?(\d{2})'?)?)?/.exec(raw.trim());
  if (!m) return null;
  const [, y, mo = '01', d = '01', h = '00', mi = '00', s = '00', z, sign, oh = '00', om = '00'] = m;
  const zone = z || sign === undefined ? 'Z' : `${sign}${oh}:${om}`;
  const iso = `${y}-${mo}-${d}T${h}:${mi}:${s}${zone}`;
  return Number.isNaN(Date.parse(iso)) ? null : iso;
}

const bounded = (v: unknown, max = 200): string | undefined => {
  if (typeof v !== 'string') return undefined;
  // eslint-disable-next-line no-control-regex
  const text = v.replace(/[\u0000-\u001f\u007f‪-‮⁦-⁩]/g, ' ').trim().slice(0, max);
  return text === '' ? undefined : text;
};

/** The document facts a viewer frame reported with its `loaded` message, bounded and free of control characters. */
export function viewerDetailsFrom(data: unknown): ViewerDetails {
  const d = (data !== null && typeof data === 'object' ? data : {}) as Record<string, unknown>;
  const out: ViewerDetails = {};
  if (typeof d['pages'] === 'number' && Number.isInteger(d['pages']) && d['pages'] > 0 && d['pages'] < 1_000_000) out.pages = d['pages'];
  const producer = bounded(d['producer']), creator = bounded(d['creator']), version = bounded(d['pdfVersion'], 20);
  if (producer) out.producer = producer;
  if (creator) out.creator = creator;
  if (version) out.pdfVersion = version;
  const created = parsePdfDate(d['created']);
  if (created) out.created = created;
  return out;
}
