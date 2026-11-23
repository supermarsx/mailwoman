import { describe, expect, it } from 'vitest';
import { discardDetail } from './discardDetail.ts';

describe('discardDetail', () => {
  it('names the draft by its subject', () => {
    expect(discardDetail('Quarterly plan')).toContain('“Quarterly plan” from account Drafts');
  });
  it('says "this draft" when there is no subject, never quoting a placeholder', () => {
    for (const none of [undefined, '', '   ']) {
      const text = discardDetail(none);
      expect(text).toBe('Permanently remove this draft from account Drafts? Closing the composer instead keeps the draft.');
      expect(text).not.toContain('(no subject)');
    }
  });
});
