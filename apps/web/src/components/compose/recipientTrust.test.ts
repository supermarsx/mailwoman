import { expect, it } from 'vitest';
import { externalRecipients, normalizeTrustedDomain } from './recipientTrust.ts';
it('checks all recipient fields and matches exact domains without trusting suffix lookalikes',()=>{
  expect(externalRecipients(['a@example.org, b@example.org.evil.test','c@partner.test','secret@external.test'],'me@example.org',['partner.test'])).toEqual(['b@example.org.evil.test','secret@external.test']);
});
it('normalizes IDNA, casing and trailing dots but rejects URL and wildcard entries',()=>{
  expect(normalizeTrustedDomain(' BÜCHER.example. ')).toBe('xn--bcher-kva.example');
  for(const invalid of ['*.example.org','https://example.org','alice@example.org','example.org:443','example.org/path'])expect(normalizeTrustedDomain(invalid)).toBeNull();
});
