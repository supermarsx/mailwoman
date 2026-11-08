import { createHash, randomBytes } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { deflateSync } from 'node:zlib';

/**
 * Real PDF files for the viewer specs, built from bytes (no PDF library): a
 * multi-page text PDF, one with an embedded TrueType font, a password-protected
 * one (standard security handler, RC4 40-bit), a scanned page (one image
 * XObject), and a large one (~20 MB of incompressible image data).
 */

const latin1 = (text: string): Buffer => Buffer.from(text, 'latin1');

/** Assemble indirect objects (1-based) into a file with a correct xref. `extraTrailer` goes inside the trailer dictionary. */
function assemble(objects: Buffer[], root = 1, extraTrailer = ''): Buffer {
  const chunks: Buffer[] = [latin1('%PDF-1.4\n%\xe2\xe3\xcf\xd3\n')];
  const offsets: number[] = [];
  let length = chunks[0]!.length;
  objects.forEach((body, i) => {
    offsets.push(length);
    const chunk = Buffer.concat([latin1(`${i + 1} 0 obj\n`), body, latin1('\nendobj\n')]);
    chunks.push(chunk);
    length += chunk.length;
  });
  let xref = `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n`;
  for (const offset of offsets) xref += `${String(offset).padStart(10, '0')} 00000 n \n`;
  chunks.push(latin1(`${xref}trailer\n<< /Root ${root} 0 R /Size ${objects.length + 1} ${extraTrailer}>>\nstartxref\n${length}\n%%EOF`));
  return Buffer.concat(chunks);
}

const stream = (dictionary: string, data: Buffer): Buffer => Buffer.concat([latin1(`<< ${dictionary} /Length ${data.length} >>\nstream\n`), data, latin1('\nendstream')]);

/** A valid multi-page text PDF (standard Helvetica, not embedded). */
export function buildPdf(pages: number): string {
  const objs: string[] = [];
  const kids = Array.from({ length: pages }, (_, i) => `${4 + i * 2} 0 R`).join(' ');
  objs.push('<< /Type /Catalog /Pages 2 0 R >>');
  objs.push(`<< /Type /Pages /Kids [${kids}] /Count ${pages} >>`);
  objs.push('<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>');
  for (let i = 0; i < pages; i += 1) {
    const text = `BT /F1 24 Tf 72 700 Td (Mailwoman PDF page ${i + 1} of ${pages}) Tj ET`;
    objs.push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents ${5 + i * 2} 0 R >>`);
    objs.push(`<< /Length ${text.length} >>\nstream\n${text}\nendstream`);
  }
  let pdf = '%PDF-1.4\n';
  const offsets: number[] = [];
  objs.forEach((body, i) => { offsets.push(pdf.length); pdf += `${i + 1} 0 obj\n${body}\nendobj\n`; });
  const xref = pdf.length;
  pdf += `xref\n0 ${objs.length + 1}\n0000000000 65535 f \n`;
  for (const off of offsets) pdf += `${String(off).padStart(10, '0')} 00000 n \n`;
  return `${pdf}trailer\n<< /Root 1 0 R /Size ${objs.length + 1} >>\nstartxref\n${xref}\n%%EOF`;
}

/** Two pages whose text uses an embedded TrueType font (Liberation Sans, from the vendored PDF.js font set), Flate-compressed. */
export function buildEmbeddedFontPdf(): Buffer {
  const font = readFileSync(resolve(process.cwd(), 'public/pdfjs/web/standard_fonts/LiberationSans-Regular.ttf'));
  const widths = Array.from({ length: 95 }, () => 569).join(' ');
  const page = (n: number): Buffer[] => [
    latin1(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents ${n} 0 R >>`),
    stream('', latin1(`BT /F1 20 Tf 72 700 Td (Embedded font page text) Tj 0 -30 Td (The quick brown fox jumps over the lazy dog) Tj ET`)),
  ];
  return assemble([
    latin1('<< /Type /Catalog /Pages 2 0 R >>'),
    latin1('<< /Type /Pages /Kids [6 0 R 8 0 R] /Count 2 >>'),
    latin1('<< /Type /Font /Subtype /TrueType /BaseFont /LiberationSans /FirstChar 32 /LastChar 126 /Encoding /WinAnsiEncoding /FontDescriptor 4 0 R /Widths [' + widths + '] >>'),
    latin1('<< /Type /FontDescriptor /FontName /LiberationSans /Flags 32 /FontBBox [-200 -300 1200 1000] /ItalicAngle 0 /Ascent 905 /Descent -212 /CapHeight 700 /StemV 80 /FontFile2 5 0 R >>'),
    stream(`/Filter /FlateDecode /Length1 ${font.length}`, deflateSync(font)),
    ...page(7),
    ...page(9),
  ]);
}

/** One page that is a single RGB image (a scan): a gradient with a diagonal, Flate-compressed. */
export function buildScannedPdf(): Buffer {
  const width = 640, height = 480;
  const pixels = Buffer.alloc(width * height * 3);
  for (let y = 0; y < height; y += 1) for (let x = 0; x < width; x += 1) {
    const at = (y * width + x) * 3;
    const line = Math.abs(x - (y * width) / height) < 3;
    pixels[at] = line ? 0 : (x * 255) / width; pixels[at + 1] = line ? 0 : (y * 255) / height; pixels[at + 2] = line ? 0 : 180;
  }
  return assemble([
    latin1('<< /Type /Catalog /Pages 2 0 R >>'),
    latin1('<< /Type /Pages /Kids [3 0 R] /Count 1 >>'),
    latin1('<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 459] /Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>'),
    stream(`/Type /XObject /Subtype /Image /Width ${width} /Height ${height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode`, deflateSync(pixels)),
    stream('', latin1('q 612 0 0 459 0 0 cm /Im0 Do Q')),
  ]);
}

/** About `megabytes` MB: one page, one image of incompressible bytes in stored Flate blocks (so the file really is that large). */
export function buildLargePdf(megabytes = 20): Buffer {
  const width = 2400, height = Math.ceil((megabytes * 1024 * 1024) / (width * 3));
  const raw = randomBytes(width * height * 3);
  return assemble([
    latin1('<< /Type /Catalog /Pages 2 0 R >>'),
    latin1('<< /Type /Pages /Kids [3 0 R] /Count 1 >>'),
    latin1('<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 740] /Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>'),
    stream(`/Type /XObject /Subtype /Image /Width ${width} /Height ${height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode`, deflateSync(raw, { level: 0 })),
    stream('', latin1('q 612 0 0 740 0 0 cm /Im0 Do Q')),
  ]);
}

const PAD = Buffer.from('28BF4E5E4E758A4164004E56FFFA01082E2E00B6D0683E802F0CA9FE6453697A', 'hex');
const md5 = (...parts: Buffer[]): Buffer => createHash('md5').update(Buffer.concat(parts)).digest();
/** RC4 (not offered by every OpenSSL build, and a few lines long). */
function rc4(key: Buffer, data: Buffer): Buffer {
  const s = Uint8Array.from({ length: 256 }, (_, i) => i);
  for (let i = 0, j = 0; i < 256; i += 1) { j = (j + s[i]! + key[i % key.length]!) & 255; [s[i], s[j]] = [s[j]!, s[i]!]; }
  const out = Buffer.alloc(data.length);
  for (let n = 0, i = 0, j = 0; n < data.length; n += 1) { i = (i + 1) & 255; j = (j + s[i]!) & 255; [s[i], s[j]] = [s[j]!, s[i]!]; out[n] = data[n]! ^ s[(s[i]! + s[j]!) & 255]!; }
  return out;
}
const padPassword = (password: string): Buffer => Buffer.concat([latin1(password), PAD]).subarray(0, 32);

/** A one-page PDF encrypted with the standard security handler (V1, R2, 40-bit RC4); `password` opens it. */
export function buildEncryptedPdf(password = 'mailwoman'): Buffer {
  const id = randomBytes(16);
  const owner = rc4(md5(padPassword(`${password}-owner`)).subarray(0, 5), padPassword(password));
  const permissions = Buffer.alloc(4); permissions.writeInt32LE(-4);
  const key = md5(padPassword(password), owner, permissions, id).subarray(0, 5);
  const user = rc4(key, PAD);
  const objectKey = (n: number): Buffer => md5(key, Buffer.from([n & 255, (n >> 8) & 255, (n >> 16) & 255, 0, 0])).subarray(0, 10);
  const content = rc4(objectKey(4), latin1('BT /F1 24 Tf 72 700 Td (Encrypted PDF opened with its password) Tj ET'));
  const hex = (b: Buffer): string => `<${b.toString('hex')}>`;
  return assemble([
    latin1('<< /Type /Catalog /Pages 2 0 R >>'),
    latin1('<< /Type /Pages /Kids [3 0 R] /Count 1 >>'),
    latin1('<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> /Contents 4 0 R >>'),
    stream('', content),
    latin1(`<< /Filter /Standard /V 1 /R 2 /O ${hex(owner)} /U ${hex(user)} /P -4 >>`),
  ], 1, `/Encrypt 5 0 R /ID [${hex(id)} ${hex(id)}] `);
}
