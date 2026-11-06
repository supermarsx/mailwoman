// Drag and drop onto the composer: what a drop carries, and what to do with it.
// Pure planning (types, limits, policy) so it is testable without a DOM drag.

import { attachmentDecision, type AttachmentPolicy } from '../../../api/attachmentPolicy.ts';
import { INLINE_IMAGE_TYPES } from '../inlineImages.ts';

import { MESSAGE_DRAG_TYPE } from '../../messageDrag.ts';
export { MESSAGE_DRAG_TYPE };
export const ATTACHMENT_DRAG_TYPE = 'application/x-mailwoman-attachment';

export type RefusalReason = 'size' | 'blocked' | 'empty' | 'total';

export interface DropPlan {
  /** Files to attach. */
  attach: File[];
  /** Image files to place in the body (Shift held, or dropped on the body). */
  inline: File[];
  refused: { name: string; reason: RefusalReason }[];
  /** Attached, but the admin policy flags them (executables and the like). */
  warned: string[];
}

export interface DropOptions {
  maxBytes: number;
  policy?: AttachmentPolicy | null;
  /** Place images in the body (Shift, or a drop on the body) and the feature is on. */
  inlineImages: boolean;
  attachAllowed: boolean;
  alreadyBytes?: number;
}

const EXECUTABLE = /\.(exe|msi|bat|cmd|com|scr|ps1|vbs|js|jar|apk|dmg|app|sh)$/i;

/** Sort dropped files into attach / inline / refused. */
export function planFiles(files: readonly File[], o: DropOptions): DropPlan {
  const plan: DropPlan = { attach: [], inline: [], refused: [], warned: [] };
  let total = o.alreadyBytes ?? 0;
  for (const file of files) {
    if (file.size === 0) { plan.refused.push({ name: file.name, reason: 'empty' }); continue; }
    if (file.size > o.maxBytes) { plan.refused.push({ name: file.name, reason: 'size' }); continue; }
    const image = INLINE_IMAGE_TYPES.has(file.type);
    if (image && o.inlineImages) {
      plan.inline.push(file);
      continue;
    }
    if (!o.attachAllowed) { plan.refused.push({ name: file.name, reason: 'blocked' }); continue; }
    if (o.policy) {
      const decision = attachmentDecision(o.policy, { name: file.name, mime: file.type, size: file.size });
      if (!decision.allowed) { plan.refused.push({ name: file.name, reason: 'blocked' }); continue; }
      if (decision.warning) plan.warned.push(file.name);
    } else if (EXECUTABLE.test(file.name)) {
      plan.warned.push(file.name);
    }
    if (total + file.size > o.maxBytes * 4) { plan.refused.push({ name: file.name, reason: 'total' }); continue; }
    total += file.size;
    plan.attach.push(file);
  }
  return plan;
}

export interface DropSummary {
  files: number;
  images: number;
  folders: number;
  messages: number;
  bytes: number;
}

/** What a drag carries, from the data transfer, before the drop (types and item kinds only). */
export function summarizeDrag(dt: Pick<DataTransfer, 'types' | 'items'> | null): DropSummary {
  const s: DropSummary = { files: 0, images: 0, folders: 0, messages: 0, bytes: 0 };
  if (!dt) return s;
  const types = Array.from(dt.types ?? []);
  if (types.includes(MESSAGE_DRAG_TYPE)) s.messages = 1;
  for (const item of Array.from(dt.items ?? [])) {
    if (item.kind !== 'file') continue;
    s.files++;
    if (INLINE_IMAGE_TYPES.has(item.type)) s.images++;
  }
  return s;
}

export function dragCarriesContent(dt: Pick<DataTransfer, 'types'> | null): boolean {
  if (!dt) return false;
  const types = Array.from(dt.types ?? []);
  return types.includes('Files') || types.includes(MESSAGE_DRAG_TYPE) || types.includes(ATTACHMENT_DRAG_TYPE);
}

/** Minimal shape of a FileSystemEntry for folder flattening. */
export interface FsEntry {
  isFile: boolean;
  isDirectory: boolean;
  name: string;
  fullPath?: string;
  file?(ok: (f: File) => void, fail: (e: unknown) => void): void;
  createReader?(): { readEntries(ok: (entries: FsEntry[]) => void, fail: (e: unknown) => void): void };
}

/** Flatten dropped folders into files (names carry the folder path, `/` replaced by ` - `). Depth and count are capped. */
export async function flattenEntries(entries: readonly FsEntry[], limit = 200, depth = 6): Promise<{ files: File[]; truncated: boolean }> {
  const files: File[] = [];
  let truncated = false;
  async function walk(entry: FsEntry, prefix: string, level: number): Promise<void> {
    if (files.length >= limit || level > depth) { truncated = true; return; }
    if (entry.isFile && entry.file) {
      const file = await new Promise<File>((ok, fail) => entry.file!(ok, fail));
      files.push(prefix === '' ? file : new File([file], `${prefix}${file.name}`, { type: file.type }));
    } else if (entry.isDirectory && entry.createReader) {
      const reader = entry.createReader();
      for (;;) {
        const batch = await new Promise<FsEntry[]>((ok, fail) => reader.readEntries(ok, fail));
        if (batch.length === 0) break;
        for (const child of batch) await walk(child, `${prefix}${entry.name} - `, level + 1);
      }
    }
  }
  for (const entry of entries) await walk(entry, '', 0);
  return { files, truncated };
}

/** A `.eml` name for a dropped message. */
export function emlFileName(subject: string): string {
  // eslint-disable-next-line no-control-regex
  const clean = subject.replace(/[\u0000-\u001f\\/:*?"<>|]+/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 80);
  return `${clean || 'message'}.eml`;
}
