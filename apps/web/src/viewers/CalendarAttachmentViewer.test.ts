import { describe, expect, it } from 'vitest';
import { messagePart } from './CalendarAttachmentViewer.tsx';

describe('the calendar attachment viewer', () => {
  it('reads only a part of a stored message from the blob id, never an upload', () => {
    expect(messagePart('Mabc123.2')).toEqual({ emailId: 'Mabc123', blobId: 'Mabc123.2' });
    expect(messagePart('Uupload.2')).toBeNull();
    expect(messagePart('Mabc123')).toBeNull();
    expect(messagePart('Mabc.x')).toBeNull();
    expect(messagePart(null)).toBeNull();
  });
});
