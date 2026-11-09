import { expect, it } from 'vitest';
import { loadCatalog, t } from '../../i18n/index.ts';
import { deliveryStatusMeaning } from './deliveryStatusMeaning.ts';
import { providerDeliveryStatus } from './providerDeliveryStatus.ts';

const exchange='dns; tenant.mail.protection.outlook.com';
const gmail='dns; gmail-smtp-in.l.google.com';
const cases: [string,string,string,string,string][] = [
  ['5.1.10',exchange,'550 5.1.10 Recipient not found','exchange','exchange-recipient'],
  ['5.1.8',exchange,'550 5.1.8 Access denied, bad outbound sender','exchange','exchange-sender-blocked'],
  ['5.2.2',exchange,'Submission quota exceeded','exchange','sending-limit'],
  ['5.2.2',exchange,'Mailbox full','', ''],
  ['5.7.12',exchange,'Sender was not authenticated by organization','exchange','exchange-internal-only'],
  ['5.7.133',exchange,'Sender not authenticated for group','exchange','exchange-group-internal'],
  ['5.7.124',exchange,'Sender not in allowed-senders list','exchange','exchange-group-allow'],
  ['5.7.64',exchange,'TenantAttribution; Relay Access Denied','exchange','exchange-connector'],
  ['5.7.57',exchange,'Client was not authenticated to send anonymous mail during MAIL FROM','exchange','exchange-auth'],
  ['5.7.606',exchange,'Access denied, banned sending IP','exchange','ip-block'],
  ['5.7.649',exchange,'Access denied','exchange','ip-block'],
  ['4.7.500',exchange,'Access denied, please try again later','exchange','exchange-temporary-restriction'],
  ['4.7.899',exchange,'Access denied','exchange','exchange-temporary-restriction'],
  ['5.7.520',exchange,'External forwarding is not allowed','exchange','exchange-forwarding'],
  ['5.7.509',exchange,'Access denied','exchange','dmarc'],
  ['5.7.233',exchange,'Daily external recipient limit','exchange','exchange-tenant-limit'],
  ['5.7.236',exchange,'onmicrosoft.com domain limit','exchange','exchange-onmicrosoft-limit'],
  ['5.7.367',exchange,'Remote server returned not permitted to relay','exchange','exchange-forward-relay'],
  ['5.7.322',exchange,'certificate-expired','exchange','certificate-expired'],
  ['4.4.317',exchange,'Cannot connect to remote server UntrustedRoot','exchange','certificate-chain'],
  ['5.4.1',exchange,'Relay Access Denied','exchange','relay'],
  ['5.7.26',gmail,'Unauthenticated email is not accepted due to DMARC policy - gsmtp','gmail','dmarc'],
  ['5.7.26',gmail,'SPF record with a hard fail policy - gsmtp','gmail','spf'],
  ['5.7.26',gmail,'Sender unauthenticated - gsmtp','gmail','gmail-authentication'],
  ['5.7.27',gmail,'SPF did not pass - gsmtp','gmail','spf'],
  ['5.7.29',gmail,'TLS required - gsmtp','gmail','starttls'],
  ['5.7.30',gmail,'DKIM did not pass - gsmtp','gmail','dkim'],
  ['5.7.40',gmail,'550 5.7.40 Domain has no DMARC record - gsmtp','gmail','gmail-dmarc-record'],
  ['5.7.40',gmail,'504 5.7.40 Unrecognized authentication type - gsmtp','gmail','gmail-auth-method'],
  ['5.7.90',gmail,'Application-specific password required - gsmtp','gmail','gmail-app-password'],
  ['5.7.90',gmail,'Please log in with your web browser - gsmtp','gmail','gmail-browser-login'],
  ['5.7.1',gmail,'550 5.7.1 Message violates example.org policy - gcdp 123 - gsmtp','gmail','gmail-custom-policy'],
  ['5.7.1',gmail,'Very low reputation of the sending IP address - gsmtp','gmail','gmail-reputation'],
  ['4.7.0',gmail,'Your email has been rate limited - gsmtp','gmail','gmail-rate'],
  ['5.7.1',gmail,'Messages missing a valid Message-ID: header - gsmtp','gmail','headers'],
  ['5.7.0',gmail,'Email relay denied 192.0.2.1 - gsmtp','gmail','gmail-relay'],
  ['5.7.0','dns; mx.example.org','530 5.7.0 Must issue a STARTTLS command first','relay','starttls'],
  ['4.7.1','dns; mx.example.org','451 4.7.1 Greylisted, please try again later','relay','greylist'],
  ['5.7.1','dns; mx.example.org','550 5.7.1 Relay access denied','relay','relay'],
  ['5.7.0','dns; eu-smtp-inbound-1.mimecast.com','550 Administrative prohibition - envelope blocked','mimecast','mimecast-block'],
  ['5.7.360',exchange,'550 Administrative prohibition - envelope blocked - https://community.mimecast.com/docs/DOC-1369','mimecast','mimecast-block'],
  ['5.7.0','dns; eu-smtp-inbound-1.mimecast.com','550 Anti-Spoofing policy - Inbound not allowed','mimecast','mimecast-spoof'],
  ['5.7.0','dns; eu-smtp-inbound-1.mimecast.com','550 Message bounced due to Content Examination Policy','mimecast','mimecast-content'],
  ['5.7.0','dns; mx.pphosted.com','554 Blocked - see https://proofpoint.com/postmaster/?ip=192.0.2.1','proofpoint','proofpoint-block'],
  ['4.7.0','dns; mx.pphosted.com','421 Deferred - see https://proofpoint.com/postmaster/','proofpoint','proofpoint-throttle'],
  ['5.7.1','dns; mx.barracudanetworks.com','554 client host blocked using Barracuda Reputation','barracuda','barracuda-reputation'],
  ['5.7.1','dns; mx.fortimailcloud.com','554 5.7.1 Access denied Message rejected by FortiMail','fortimail','fortimail-relay'],
  ['4.7.0','dns; mta5.am0.yahoodns.net','421 4.7.0 [TSS04] temporarily deferred due to unexpected volume or user complaints','yahoo','yahoo-deferred'],
  ['5.7.9','dns; mta5.am0.yahoodns.net','554 5.7.9 Message not accepted for policy reasons','yahoo','yahoo-policy'],
];
it.each(cases)('interprets %s at %s: %s',async(status,remoteMta,diagnostic,provider,reason)=>{
  await loadCatalog('mail-dsn');
  const meaning=providerDeliveryStatus(status,{remoteMta,diagnostic});
  if(!provider) {expect(meaning).toBeNull();return;}
  expect(meaning?.providerKey).toBe(`dsn-provider-${provider}`);
  expect(meaning?.reasonKey).toBe(`dsn-vendor-${reason}`);
  for(const key of [meaning!.providerKey,meaning!.reasonKey,meaning!.adviceKey]) expect(t(key)).not.toBe(key);
  expect(new URL(meaning!.sourceUrl).protocol).toBe('https:');
});
it('does not guess a vendor from the numeric code or a lookalike host',()=>{
  for(const remoteMta of ['dns; mx.example.org','dns; eviloutlook.com','dns; outlook.com.evil.example','dns; google.com.evil.example']) {
    expect(providerDeliveryStatus('5.1.10',{remoteMta})).toBeNull();
  }
  expect(deliveryStatusMeaning('5.1.10')?.reasonKey).toBe('dsn-status-1-10');
  expect(deliveryStatusMeaning('5.1.10',{remoteMta:exchange})?.reasonKey).toBe('dsn-vendor-exchange-recipient');
});
it('keeps code ranges bounded and future codes on the standard fallback',()=>{
  for(const status of ['5.7.605','5.7.650','4.7.499','4.7.700','4.7.849','4.7.900','5.7.999']) expect(providerDeliveryStatus(status,{remoteMta:exchange})).toBeNull();
});
it('does not reinterpret success as a rejection or accept malformed status values',()=>{
  for(const status of ['2.7.26','550','5.7.1 text','5.7.1234']) expect(providerDeliveryStatus(status,{remoteMta:gmail,diagnostic:'DMARC policy - gsmtp'})).toBeNull();
});
it('retains permanent severity when a wrapped diagnostic contains a temporary downstream code',()=>{
  const meaning=deliveryStatusMeaning('5.4.7',{remoteMta:gmail,diagnostic:'421 4.7.28 unusual rate - gsmtp'});
  expect(meaning?.classKey).toBe('dsn-class-5');
  expect(meaning?.reasonKey).toBe('dsn-vendor-gmail-rate');
});
