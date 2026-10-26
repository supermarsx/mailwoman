import { expect, it } from 'vitest';
import { inspectFile } from './fileInfo.ts';

it('identifies disguised executable content independently of the filename and MIME', () => {
  const file = inspectFile('invoice.pdf', 'application/pdf', new Uint8Array([0x4d, 0x5a, 0, 0]));
  expect(file.warnings).toEqual(['executable', 'typeMismatch']);
  expect(file.blockPreview).toBe(true);
});
it('warns about double extensions, bidi controls and macro-enabled Office documents', () => {
  expect(inspectFile('photo.jpg.exe', 'application/octet-stream').warnings).toContain('disguisedName');
  expect(inspectFile('invoice\u202egnp.exe', 'application/octet-stream').warnings).toContain('disguisedName');
  expect(inspectFile('budget.XLSM', 'application/vnd.ms-excel.sheet.macroEnabled.12', new Uint8Array([0x50, 0x4b, 3, 4])).warnings).toEqual(['macro']);
});
it('recognizes legitimate document containers without treating every ZIP-backed Office file as a mismatch', () => {
  expect(inspectFile('report.docx', 'application/vnd.openxmlformats-officedocument.wordprocessingml.document', new Uint8Array([0x50, 0x4b, 3, 4])).warnings).toEqual([]);
  expect(inspectFile('recording.mp3', 'audio/mpeg').icon).toBe('fileAudio');
  expect(inspectFile('video.mp4', 'video/mp4').icon).toBe('fileVideo');
  expect(inspectFile('payload.html', 'text/html').warnings).toContain('activeDocument');
});
