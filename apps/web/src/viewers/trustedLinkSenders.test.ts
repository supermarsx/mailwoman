import {describe,expect,it} from 'vitest';
import vectors from './trustedLinkSenders.vectors.json';
import {isTrustedLinkSender,isTrustedLinkSenderDomain,normalizeTrustedLinkSender,trustedLinkSenders,trustedLinkSenderDomains} from './trustedLinkSenders.ts';
describe('exact trusted link senders',()=>{
 it('trusts only the exact sender domain and keeps sender and destination trust independent',()=>{
  const values={'links.trustedSenderDomains':{value:['example.test']},'links.trustedDomains':{value:['destination.test']}};
  expect(isTrustedLinkSenderDomain('Alice@EXAMPLE.TEST',values)).toBe(true);
  for(const sender of ['alice@sub.example.test','alice@evilexample.test','alice@example.test.evil','alice@destination.test','alice@[127.0.0.1]','Alice <alice@example.test>'])expect(isTrustedLinkSenderDomain(sender,values)).toBe(false);
  expect(isTrustedLinkSender('Alice@example.test',values)).toBe(false);
  for(const raw of [['EXAMPLE.TEST'],['*.example.test'],['com'],['example.test',3],Array(101).fill('example.test')])expect(trustedLinkSenderDomains({'links.trustedSenderDomains':{value:raw}})).toEqual([]);
 });
 it.each(vectors)('normalizes $input',({input,normalized})=>expect(normalizeTrustedLinkSender(input)).toBe(normalized));
 it('preserves local case, dots and plus aliases rather than broadening a saved trust decision',()=>{
  const values={'links.trustedSenders':{value:['Alice+news@example.test']}};
  expect(isTrustedLinkSender('Alice+news@EXAMPLE.TEST',values)).toBe(true);
  for(const sender of ['alice+news@example.test','Alice@example.test','Alice+other@example.test','A.lice+news@example.test','other@example.test','Alice+news@example.test.evil'])expect(isTrustedLinkSender(sender,values)).toBe(false);
  expect(isTrustedLinkSender('Alice+news@example.test',{'links.trustedSenders':{value:[]}})).toBe(false);
 });
 it('fails closed for unknown, malformed, noncanonical and oversized effective lists',()=>{
  expect(trustedLinkSenders({})).toEqual([]);
  for(const raw of ['alice@example.test',['alice@example.test',3],['Alice@EXAMPLE.TEST'],['*.example.test'],Array(101).fill('alice@example.test')])expect(trustedLinkSenders({'links.trustedSenders':{value:raw}})).toEqual([]);
 });
});
