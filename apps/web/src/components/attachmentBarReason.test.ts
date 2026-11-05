import { describe, expect, it } from 'vitest';
import { attachmentBarReason, type AttachmentBarInput } from './attachmentBarReason.ts';

const base: AttachmentBarInput = { hasAttachment: true, attachments: [{ blobId: 'a.1' }], policy: 'ready' };
const run = (over: Partial<AttachmentBarInput>) => attachmentBarReason({ ...base, ...over });

describe('attachmentBarReason', () => {
  it('shows the bar with no reason for a normal message', () => {
    expect(run({})).toEqual({ show: true, reason: null, count: 0 });
  });
  it('shows nothing for a message without attachments', () => {
    expect(run({ hasAttachment: false, attachments: [] })).toEqual({ show: false, reason: null, count: 0 });
    expect(run({ hasAttachment: undefined, attachments: undefined })).toEqual({ show: false, reason: null, count: 0 });
  });
  it('flags a hasAttachment message whose list is missing or empty', () => {
    expect(run({ attachments: undefined }).reason).toBe('list-missing');
    expect(run({ attachments: [] })).toMatchObject({ show: false, reason: 'list-missing' });
  });
  it('flags parts without a blob', () => {
    expect(run({ attachments: [{ blobId: null }, { blobId: '' }] })).toEqual({ show: false, reason: 'unavailable-parts', count: 2 });
  });
  it('still shows the bar when only some parts lack a blob', () => {
    expect(run({ attachments: [{ blobId: null }, { blobId: 'a.2' }] }).show).toBe(true);
  });
  it('explains an encrypted message that is not unlocked', () => {
    expect(run({ attachments: [], encrypted: true })).toMatchObject({ show: false, reason: 'encrypted' });
  });
  it('explains attachments that are only inline images in the body', () => {
    expect(run({ hasAttachment: false, attachments: [], inlineCount: 3 })).toEqual({ show: false, reason: 'inline-only', count: 3 });
    expect(run({ attachments: [], inlineCount: 1 }).reason).toBe('inline-only');
  });
  it('keeps the bar and reports a policy that blocks everything', () => {
    expect(run({ blockedCount: 1 })).toEqual({ show: true, reason: 'policy-blocked', count: 1 });
    expect(run({ attachments: [{ blobId: 'a' }, { blobId: 'b' }], blockedCount: 1 }).reason).toBeNull();
  });
  it('keeps the bar and reports a policy that failed to load', () => {
    expect(run({ policy: 'error' })).toEqual({ show: true, reason: 'policy-unavailable', count: 1 });
  });
  it('does not claim a policy block while the policy is loading', () => {
    expect(run({ policy: 'loading', blockedCount: 1 }).reason).toBeNull();
  });
});
