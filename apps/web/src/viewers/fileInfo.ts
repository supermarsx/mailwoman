import type { IconName } from '../components/Icon.tsx';

const EXECUTABLE = new Set('exe dll com scr msi msp bat cmd ps1 psm1 vbs vbe js jse wsf wsh hta lnk url desktop app dmg pkg deb rpm sh bash jar apk iso img reg chm'.split(' '));
const MACRO = new Set('docm dotm xlsm xltm xlam xlsb pptm potm ppam ppsm sldm'.split(' '));
const EXTENSIONS: Record<string, string> = {
  pdf: 'PDF document', txt: 'Plain text', csv: 'CSV table', tsv: 'Tab-separated table', json: 'JSON data', xml: 'XML document', html: 'HTML document', htm: 'HTML document', md: 'Markdown text', log: 'Log text',
  eml: 'Email message', msg: 'Outlook message', dat: 'Data file', ics: 'Calendar event', vcf: 'Contact card',
  doc: 'Word document', docx: 'Word document', odt: 'OpenDocument text', rtf: 'Rich text', xls: 'Excel workbook', xlsx: 'Excel workbook', ods: 'OpenDocument spreadsheet', ppt: 'PowerPoint presentation', pptx: 'PowerPoint presentation', odp: 'OpenDocument presentation',
  zip: 'ZIP archive', '7z': '7-Zip archive', rar: 'RAR archive', tar: 'TAR archive', gz: 'Gzip archive', bz2: 'Bzip2 archive', xz: 'XZ archive',
  png: 'PNG image', jpg: 'JPEG image', jpeg: 'JPEG image', gif: 'GIF image', webp: 'WebP image', avif: 'AVIF image', heic: 'HEIC image', svg: 'SVG image', tif: 'TIFF image', tiff: 'TIFF image', bmp: 'Bitmap image', ico: 'Icon',
  mp3: 'MP3 audio', wav: 'WAVE audio', ogg: 'Ogg media', opus: 'Opus audio', flac: 'FLAC audio', m4a: 'MPEG-4 audio', aac: 'AAC audio', mp4: 'MPEG-4 video', webm: 'WebM video', mov: 'QuickTime video', avi: 'AVI video', mkv: 'Matroska video',
};
export type FileWarning = 'executable' | 'macro' | 'disguisedName' | 'typeMismatch' | 'activeDocument';
export function inspectFile(name: string, mime: string, header = new Uint8Array()): { extension: string; description: string; detected: string | null; icon: IconName; warnings: FileWarning[]; blockPreview: boolean } {
  const extension = name.trim().split('.').length > 1 ? name.trim().split('.').pop()!.toLowerCase() : '';
  const normalized = mime.toLowerCase().split(';')[0]!.trim();
  const prefix = (...bytes: number[]): boolean => bytes.every((byte, i) => header[i] === byte);
  let detected: string | null = null;
  if (prefix(0x4d, 0x5a) || prefix(0x7f, 0x45, 0x4c, 0x46)) detected = 'application/x-executable';
  else if (prefix(0x25, 0x50, 0x44, 0x46, 0x2d)) detected = 'application/pdf';
  else if (prefix(0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a)) detected = 'image/png';
  else if (prefix(0xff, 0xd8, 0xff)) detected = 'image/jpeg';
  else if (prefix(0x47, 0x49, 0x46, 0x38)) detected = 'image/gif';
  else if (prefix(0x50, 0x4b, 0x03, 0x04)) detected = 'application/zip';
  const warnings: FileWarning[] = [];
  if (EXECUTABLE.has(extension) || detected === 'application/x-executable') warnings.push('executable');
  if (MACRO.has(extension)) warnings.push('macro');
  if (/[\u202a-\u202e\u2066-\u2069]/u.test(name) || /\.(?:pdf|txt|docx?|xlsx?|jpe?g|png)\s*\.[^.]+$/i.test(name)) warnings.push('disguisedName');
  const zipDocument = /^(?:docx|xlsx|pptx|odt|ods|odp)$/.test(extension) || MACRO.has(extension);
  if (detected && normalized && normalized !== 'application/octet-stream' && detected !== normalized && !(detected === 'application/zip' && zipDocument)) warnings.push('typeMismatch');
  if (['html', 'htm', 'svg'].includes(extension)) warnings.push('activeDocument');
  const icon: IconName = normalized.startsWith('image/') ? 'image' : normalized.startsWith('audio/') ? 'fileAudio' : normalized.startsWith('video/') ? 'fileVideo' : extension === 'pdf' ? 'filePdf' : /^(?:zip|7z|rar|tar|gz|bz2|xz)$/.test(extension) ? 'archive' : /^(?:json|xml|html|htm|md|js|css|yaml|yml)$/.test(extension) ? 'fileCode' : ['eml', 'msg'].includes(extension) ? 'mail' : 'file';
  return { extension, description: EXTENSIONS[extension] ?? (EXECUTABLE.has(extension) ? 'Executable or system file' : MACRO.has(extension) ? 'Macro-enabled document' : 'File'), detected, icon, warnings, blockPreview: warnings.includes('executable') || warnings.includes('typeMismatch') };
}
