// Dragging an attachment out of the reader's attachment bar into a composer.
// The drag carries a small JSON token (which message and blob, and the account
// it belongs to), never the bytes: the composer fetches through the
// policy-checked download URL on drop and refuses a token from another account.

export const ATTACHMENT_DRAG_TYPE = 'application/x-mailwoman-attachment';

export interface AttachmentDragToken {
  accountId: string | null;
  emailId: string;
  blobId: string;
  name: string;
  mime: string;
  size: number;
}

export function beginAttachmentDrag(event: DragEvent, token: AttachmentDragToken): void {
  if (!event.dataTransfer) return;
  event.dataTransfer.effectAllowed = 'copy';
  event.dataTransfer.setData(ATTACHMENT_DRAG_TYPE, JSON.stringify(token));
}

/** The token of a drop, or null when absent or malformed. */
export function readAttachmentDrag(dt: Pick<DataTransfer, 'getData'> | null): AttachmentDragToken | null {
  if (!dt) return null;
  try {
    const raw: unknown = JSON.parse(dt.getData(ATTACHMENT_DRAG_TYPE));
    if (typeof raw !== 'object' || raw === null) return null;
    const v = raw as Record<string, unknown>;
    if (typeof v['emailId'] !== 'string' || typeof v['blobId'] !== 'string' || typeof v['name'] !== 'string' || typeof v['mime'] !== 'string' || typeof v['size'] !== 'number') return null;
    return { accountId: typeof v['accountId'] === 'string' ? v['accountId'] : null, emailId: v['emailId'], blobId: v['blobId'], name: v['name'], mime: v['mime'], size: v['size'] };
  } catch { return null; }
}
