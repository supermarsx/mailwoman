import {describe,expect,it} from 'vitest';
import {normalizeContactAddress,contactAddressFromMailto,emailAtTextPoint} from './contactAddress.ts';

describe('contact address identity',()=>{
 it('preserves local-part case, plus aliases, quoted local parts and international addresses',()=>{
  expect(normalizeContactAddress(' Alice+Team@EXAMPLE.ORG ')).toBe('Alice+Team@example.org');
  expect(normalizeContactAddress('"A B"@EXAMPLE.ORG')).toBe('"A B"@example.org');
  expect(normalizeContactAddress('δοκιμή@ΠΑΡΆΔΕΙΓΜΑ.ΔΟΚΙΜΉ')).toBe('δοκιμή@παράδειγμα.δοκιμή');
  expect(normalizeContactAddress('alice@example.org')).not.toBe(normalizeContactAddress('Alice@example.org'));
 });
 it('refuses display-name lists, malformed mailboxes and control-character injection',()=>{
  for(const value of ['Alice <alice@example.org>','one@example.org,two@example.org','a..b@example.org','person@','a\nb@example.org','person@example.org\r\nBcc:other@example.org','person@example.org/path'])expect(normalizeContactAddress(value),value).toBeNull();
 });
 it('extracts one mailbox from mailto without accepting header actions as recipients',()=>{
  expect(contactAddressFromMailto('mailto:Alice%2BTeam@EXAMPLE.org?subject=Hello&bcc=hidden@example.org')).toBe('Alice+Team@example.org');
  expect(contactAddressFromMailto('mailto:%22A%20B%22@EXAMPLE.org')).toBe('"A B"@example.org');
  for(const value of ['https://example.org','mailto:one@example.org,two@example.org','mailto:one@example.org%0d%0aBcc:two@example.org','mailto:%broken','mailto:'])expect(contactAddressFromMailto(value),value).toBeNull();
 });
});

describe('the address under the text caret',()=>{
 it('does not mistake web URL user-info for a plain email while preserving slash local parts',()=>{
  for(const text of ['https://person@example.org','Go to http://Person@example.org for details.'])expect(emailAtTextPoint(text,text.indexOf('@'))).toBeNull();
  const plain='Write //Person@example.org or team/name@example.org.';
  expect(emailAtTextPoint(plain,plain.indexOf('@'))).toBe('//Person@example.org');
  expect(emailAtTextPoint(plain,plain.lastIndexOf('@'))).toBe('team/name@example.org');
 });
 it('preserves valid IPv6 literal syntax through repeated normalization and mailto extraction',()=>{
  const address='Person+tag@[IPv6:2001:DB8::1]';
  expect(normalizeContactAddress(address)).toBe(address);
  expect(normalizeContactAddress(normalizeContactAddress(address)!)).toBe(address);
  expect(contactAddressFromMailto(`mailto:${address}`)).toBe(address);
  const text=`Write ${address} for help.`;expect(emailAtTextPoint(text,text.indexOf('@'))).toBe(address);
 });
 it('selects only the pointed address, not another email elsewhere in the paragraph',()=>{
  const text='Write Alice+team@EXAMPLE.org or Bob@example.org for help.';
  expect(emailAtTextPoint(text,text.indexOf('Alice')+4)).toBe('Alice+team@example.org');
  expect(emailAtTextPoint(text,text.indexOf('Bob')+2)).toBe('Bob@example.org');
  expect(emailAtTextPoint(text,text.indexOf(' or ')+2)).toBeNull();
  expect(emailAtTextPoint(text,text.indexOf('help'))).toBeNull();
 });
 it('handles quoted spaces, EAI, surrounding punctuation and wrapped address text safely',()=>{
  const quoted='Ask "A B"@EXAMPLE.org, please.';expect(emailAtTextPoint(quoted,quoted.indexOf('B"'))).toBe('"A B"@example.org');
  const eai='Contact δοκιμή@ΠΑΡΆΔΕΙΓΜΑ.ΔΟΚΙΜΉ.';expect(emailAtTextPoint(eai,eai.indexOf('@'))).toBe('δοκιμή@παράδειγμα.δοκιμή');
  const bracket='(<Person+tag@example.org>)';expect(emailAtTextPoint(bracket,bracket.indexOf('@'))).toBe('Person+tag@example.org');
 });
 it('never repairs a truncated oversized token into an apparently valid mailbox',()=>{
  const text='x'.repeat(600)+'person@example.org';
  expect(emailAtTextPoint(text,text.indexOf('@'))).toBeNull();
  for(const offset of [-1,1.5,Number.NaN,text.length+1])expect(emailAtTextPoint(text,offset)).toBeNull();
 });
});
