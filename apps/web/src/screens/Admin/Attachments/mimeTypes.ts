// Common MIME types for the "Test a file" form: one shared list, grouped, plus
// helpers to validate a custom type and to guess a type from a file name.

export type MimeGroupId = 'documents' | 'images' | 'archives' | 'executables' | 'media' | 'email' | 'other';
export interface MimeEntry { value: string; label: string; ext: readonly string[] }
export interface MimeGroup { id: MimeGroupId; entries: readonly MimeEntry[] }

const e = (value: string, label: string, ...ext: string[]): MimeEntry => ({ value, label, ext });

export const MIME_GROUPS: readonly MimeGroup[] = [
  { id: 'documents', entries: [
    e('application/pdf', 'PDF', 'pdf'),
    e('application/msword', 'Word (doc)', 'doc'),
    e('application/vnd.openxmlformats-officedocument.wordprocessingml.document', 'Word (docx)', 'docx'),
    e('application/vnd.ms-excel', 'Excel (xls)', 'xls'),
    e('application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', 'Excel (xlsx)', 'xlsx'),
    e('application/vnd.ms-powerpoint', 'PowerPoint (ppt)', 'ppt'),
    e('application/vnd.openxmlformats-officedocument.presentationml.presentation', 'PowerPoint (pptx)', 'pptx'),
    e('application/vnd.oasis.opendocument.text', 'OpenDocument text (odt)', 'odt'),
    e('application/vnd.oasis.opendocument.spreadsheet', 'OpenDocument spreadsheet (ods)', 'ods'),
    e('application/vnd.oasis.opendocument.presentation', 'OpenDocument presentation (odp)', 'odp'),
    e('application/rtf', 'Rich text (rtf)', 'rtf'),
    e('text/plain', 'Plain text', 'txt', 'log'),
    e('text/csv', 'CSV', 'csv'),
    e('text/html', 'HTML', 'html', 'htm'),
    e('application/json', 'JSON', 'json'),
    e('application/xml', 'XML', 'xml'),
    e('text/markdown', 'Markdown', 'md', 'markdown'),
  ] },
  { id: 'images', entries: [
    e('image/png', 'PNG', 'png'),
    e('image/jpeg', 'JPEG', 'jpg', 'jpeg'),
    e('image/gif', 'GIF', 'gif'),
    e('image/webp', 'WebP', 'webp'),
    e('image/svg+xml', 'SVG', 'svg'),
    e('image/heic', 'HEIC', 'heic'),
    e('image/tiff', 'TIFF', 'tif', 'tiff'),
    e('image/bmp', 'BMP', 'bmp'),
  ] },
  { id: 'archives', entries: [
    e('application/zip', 'ZIP', 'zip'),
    e('application/x-7z-compressed', '7-Zip', '7z'),
    e('application/vnd.rar', 'RAR', 'rar'),
    e('application/x-tar', 'TAR', 'tar'),
    e('application/gzip', 'Gzip', 'gz', 'tgz'),
  ] },
  { id: 'executables', entries: [
    e('application/x-msdownload', 'Windows executable (exe)', 'exe', 'dll'),
    e('application/x-msi', 'Windows installer (msi)', 'msi'),
    e('text/javascript', 'JavaScript', 'js', 'mjs'),
    e('application/x-sh', 'Shell script (sh)', 'sh'),
    e('application/x-bat', 'Batch file (bat)', 'bat', 'cmd'),
    e('application/java-archive', 'Java archive (jar)', 'jar'),
    e('application/vnd.android.package-archive', 'Android package (apk)', 'apk'),
    e('application/x-apple-diskimage', 'macOS disk image (dmg)', 'dmg'),
  ] },
  { id: 'media', entries: [
    e('audio/mpeg', 'MP3', 'mp3'),
    e('video/mp4', 'MP4', 'mp4'),
    e('audio/wav', 'WAV', 'wav'),
    e('video/webm', 'WebM', 'webm'),
    e('video/quicktime', 'QuickTime (mov)', 'mov'),
  ] },
  { id: 'email', entries: [
    e('message/rfc822', 'Email message (eml)', 'eml'),
    e('text/calendar', 'Calendar (ics)', 'ics'),
    e('text/vcard', 'Contact card (vcf)', 'vcf'),
  ] },
  { id: 'other', entries: [e('application/octet-stream', 'Unknown binary (octet-stream)', 'bin')] },
];

export const MIME_ENTRIES: readonly MimeEntry[] = MIME_GROUPS.flatMap((g) => g.entries);
export const DEFAULT_MIME = 'application/octet-stream';

const TOKEN = /^[a-z0-9!#$&^_.+-]+$/i;
/** `type/subtype` shape (RFC 6838 tokens). Parameters (`; charset=...`) are not accepted. */
export function isMimeShape(value: string): boolean {
  const parts = value.trim().split('/');
  return parts.length === 2 && TOKEN.test(parts[0]!) && TOKEN.test(parts[1]!);
}

/** Filter by case-insensitive substring of the type, label or extension. Empty query lists everything. */
export function filterMimeGroups(query: string, groups: readonly MimeGroup[] = MIME_GROUPS): MimeGroup[] {
  const q = query.trim().toLowerCase();
  if (q === '') return [...groups];
  const ext = q.replace(/^\./, '');
  return groups
    .map((g) => ({ ...g, entries: g.entries.filter((x) => x.value.includes(q) || x.label.toLowerCase().includes(q) || x.ext.some((s) => s.includes(ext))) }))
    .filter((g) => g.entries.length > 0);
}

/** Type for a file name's extension, or null when it is not in the list. */
export function mimeForFilename(name: string): string | null {
  const dot = name.lastIndexOf('.');
  if (dot < 0) return null;
  const ext = name.slice(dot + 1).toLowerCase();
  return MIME_ENTRIES.find((x) => x.ext.includes(ext))?.value ?? null;
}

/** Type to pre-fill for a chosen file: the browser-detected type if usable, else the extension, else octet-stream. */
export function detectMime(file: { name: string; type: string }): string {
  const detected = file.type.trim().toLowerCase();
  return detected !== '' && isMimeShape(detected) ? detected : (mimeForFilename(file.name) ?? DEFAULT_MIME);
}
