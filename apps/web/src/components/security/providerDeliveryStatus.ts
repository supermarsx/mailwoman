/** Vendor interpretations are gated by host/diagnostic evidence, never the recipient domain.
 * Official references are attached to each result. Reviewed 2026-11-09.
 */
export interface DeliveryStatusContext { diagnostic?: string | null; remoteMta?: string | null }
export interface ProviderMeaning { reasonKey: string; adviceKey: string; providerKey: string; sourceUrl: string }
type Provider = 'exchange' | 'gmail' | 'mimecast' | 'proofpoint' | 'barracuda' | 'fortimail' | 'relay' | 'yahoo';
const sources: Record<Provider,string> = {
  yahoo: 'https://senders.yahooinc.com/smtp-error-codes/',
  relay: 'https://support.microsoft.com/en-gb/support/known-issues/i-receive-a-550-553-or-relay-prohibited-error-when-sending-email-messages',
  exchange: 'https://learn.microsoft.com/en-us/troubleshoot/exchange/email-delivery/ndr/non-delivery-reports-in-exchange-online',
  gmail: 'https://support.google.com/mail/answer/3726730?hl=en',
  mimecast: 'https://mimecastsupport.zendesk.com/hc/en-us/articles/34000709564691-Policies-Mimecast-SMTP-Error-Codes',
  proofpoint: 'https://www.proofpoint.com/us/postmaster',
  barracuda: 'https://campus.barracuda.com/product/emailsecuritygateway/doc/171804672/smtp-error-codes/',
  fortimail: 'https://community.fortinet.com/fortimail-26/troubleshooting-tip-unable-to-send-outbound-email-from-microsoft-with-error-554-5-7-1-access-denied-message-rejected-by-fortimail-207084',
};
const markers: [Provider, RegExp][] = [
  ['mimecast', /\bmimecast\b/i],
  ['proofpoint', /\bproofpoint\b|\bpphosted\.com\b/i],
  ['barracuda', /\bbarracuda(?:networks)?\b/i],
  ['fortimail', /\bfortimail(?:cloud)?\b/i],
  ['yahoo', /\b(?:senders|postmaster)\.yahooinc\.com\b|\[(?:TSS?\d+|PH01)\]/i],
  ['gmail', /\bgsmtp\b|\bsupport\.google\.com\/(?:mail|a)\b/i],
  ['exchange', /\b(?:RESOLVER\.|TenantAttribution\b|Exchange Online\b|Microsoft 365\b)|\b(?:[a-z0-9-]+\.)*(?:protection\.outlook\.com|outlook\.office365\.com)\b/i],
];
function hostProvider(value: string): Provider | undefined {
  // Remote-MTA is normally "dns; hostname". Require a complete DNS suffix.
  const host=value.trim().toLowerCase().replace(/^dns;\s*/, '').replace(/\.$/, '');
  const suffix=(domains:string[])=>domains.some(domain=>host===domain||host.endsWith(`.${domain}`));
  if(suffix(['outlook.com','office365.com','exchange.microsoft.com'])) return 'exchange';
  if(suffix(['google.com','googlemail.com','gmail.com'])) return 'gmail';
  if(suffix(['yahoo.com','yahoodns.net','aol.com'])) return 'yahoo';
  if(suffix(['mimecast.com'])) return 'mimecast';
  if(suffix(['proofpoint.com','pphosted.com'])) return 'proofpoint';
  if(suffix(['barracuda.com','barracudanetworks.com'])) return 'barracuda';
  if(suffix(['fortimailcloud.com'])) return 'fortimail';
  return undefined;
}
// Code -> explanation, next step. Unlisted variants keep the standard meaning.
const exchange: Record<string,[string,string]> = {
  '5.0.350':['exchange-generic','0'],
  '5.1.0':['exchange-recipient','1'], '5.1.10':['exchange-recipient','1'],
  '5.1.8':['exchange-sender-blocked','account'], '5.1.20':['headers','format'],
  '5.1.90':['sending-limit','rate'],
  '5.2.121':['exchange-recipient-rate','rate'], '5.2.122':['exchange-receive-rate','rate'],
  '5.3.190':['exchange-journaling','admin'],
  '5.4.1':['exchange-recipient-access','1'], '5.4.8':['mta-sts-host','tls'], '5.6.11':['headers','format'], '5.4.6':['routing-loop','route'],
  '5.4.14':['routing-loop','route'], '5.4.300':['expired','route'],
  '5.5.0':['exchange-recipient','1'],
  '5.7.12':['exchange-internal-only','recipient-admin'],
  '5.7.13':['exchange-public-folder','recipient-admin'],
  '5.7.57':['exchange-auth','auth'], '5.7.64':['exchange-connector','connector'],
  '5.7.124':['exchange-group-allow','recipient-admin'],
  '5.7.133':['exchange-group-internal','recipient-admin'],
  '5.7.134':['exchange-internal-only','recipient-admin'],
  '5.7.135':['exchange-public-folder','recipient-admin'],
  '5.7.136':['exchange-internal-only','recipient-admin'],
  '5.7.232':['exchange-tenant-limit','rate'], '5.7.233':['exchange-tenant-limit','rate'],
  '5.7.236':['exchange-onmicrosoft-limit','domain'],
  '5.7.367':['exchange-forward-relay','connector'],
  '5.7.501':['exchange-sender-blocked','account'],
  '5.7.502':['exchange-sender-blocked','account'], '5.7.503':['exchange-sender-blocked','account'],
  '5.7.504':['exchange-recipient','1'], '5.7.505':['exchange-recipient-access','recipient-admin'],
  '5.7.506':['helo','dns'], '5.7.507':['recipient-ip-block','recipient-admin'],
  '5.7.508':['sending-limit','rate'], '5.7.509':['dmarc','authentication'],
  '5.7.510':['exchange-ipv6','route'], '5.7.511':['ip-block','reputation'],
  '5.7.512':['headers','format'], '5.7.513':['recipient-ip-block','recipient-admin'],
  '5.7.520':['exchange-forwarding','recipient-admin'],
  '5.7.703':['exchange-tenant-blocklist','admin'],
  '5.7.705':['exchange-tenant-blocked','account'], '5.7.708':['exchange-tenant-blocked','account'],
  '5.7.750':['exchange-unregistered','domain'], '5.7.800':['exchange-domain-blocked','account'],
  '4.4.8':['mta-sts-host','tls'], '4.4.316':['connection-refused','route'],
  '4.4.317':['certificate-chain','tls'],
};
for(const kind of ['4','5']) {
  for(const [detail,reason] of [[321,'starttls'],[322,'certificate-expired'],[323,'dane'],[324,'dnssec'],[325,'certificate-host']] as const) exchange[`${kind}.7.${detail}`]=[reason,'tls'];
}
const gmail: Record<string,[string,string]> = {
  '5.7.14':['gmail-browser-login','auth'],
  '5.7.24':['gmail-spf-record','authentication'], '4.7.24':['gmail-spf-record','authentication'],
  '5.7.25':['ptr','dns'], '4.7.23':['ptr','dns'],
  '5.7.26':['gmail-authentication','authentication'], '4.7.26':['gmail-authentication','authentication'],
  '5.7.27':['spf','authentication'], '4.7.27':['spf','authentication'],
  '5.7.28':['gmail-rate','rate'], '4.7.28':['gmail-rate','rate'],
  '5.7.29':['starttls','tls'], '4.7.29':['starttls','tls'],
  '5.7.30':['dkim','authentication'], '4.7.30':['dkim','authentication'],
  '5.7.80':['credentials','auth'],
  '5.7.90':['gmail-app-password','auth'],
};
function result(provider:Provider, reason:string, advice:string):ProviderMeaning {
  return {reasonKey:`dsn-vendor-${reason}`,adviceKey:/^[0-7]$/.test(advice)?`dsn-advice-${advice}`:`dsn-vendor-advice-${advice}`,providerKey:`dsn-provider-${provider}`,sourceUrl:sources[provider]};
}
export function providerDeliveryStatus(status:string, context:DeliveryStatusContext):ProviderMeaning | null {
  if(!/^[45]\.\d{1,3}\.\d{1,3}$/.test(status)) return null;
  const text=(context.diagnostic??'').slice(0,4096);
  const provider=markers.find(([,pattern])=>pattern.test(text))?.[0]??hostProvider(context.remoteMta??'')
    ??(/relay (?:access )?denied|not permitted to relay|unable to relay|relaying denied|must issue (?:a )?STARTTLS|TLS (?:is )?required|greylist(?:ed|ing)?/i.test(text)?'relay':undefined);
  if(!provider) return null;
  const found=(reason:string,advice:string)=>result(provider,reason,advice);
  // Specific diagnostic variants win over overloaded numeric codes.
  if(provider==='yahoo') {
    if(/\[(?:TSS?\d+)\]|temporarily deferred/i.test(text)) return found('yahoo-deferred','rate');
    if(status==='5.7.9'||/\[PH01\]|not accepted for policy reasons/i.test(text)) return found('yahoo-policy','authentication');
  }
  if(provider==='gmail') {
    if(/\bgcdp\b/i.test(text)) return found('gmail-custom-policy','recipient-admin');
    if(/DMARC record/i.test(text)) return found('gmail-dmarc-record','authentication');
    if(/DMARC/i.test(text)&&/not accepted|fail|policy|record/i.test(text)) return found('dmarc','authentication');
    if(/SPF/i.test(text)&&/hard fail|hardfail/i.test(text)) return found('spf','authentication');
    if(/\b550[- ]5\.7\.40\b/i.test(text)) return found('gmail-dmarc-record','authentication');
    if(/Unrecognized authentication type|XOAUTH is no longer supported/i.test(text)) return found('gmail-auth-method','auth');
    if(/log in (?:through|with) your web browser/i.test(text)) return found('gmail-browser-login','auth');
    if(/(?:SMTP relay|sending) limit exceeded|rate limited|unusual rate/i.test(text)) return found('gmail-rate','rate');
    if(/(?:missing|multiple|duplicate|malformed|valid).{0,50}(?:header|From:|Message-ID)|not RFC 5322 compliant/i.test(text)) return found('headers','format');
    if(/very low reputation|likely unsolicited/i.test(text)) return found('gmail-reputation','reputation');
    if(/potential security issue/i.test(text)) return found('content-security','content');
    if(/Invalid credentials for relay|Email relay denied/i.test(text)) return found('gmail-relay','connector');
    if(/PTR record|forward DNS/i.test(text)) return found('ptr','dns');
    if(/STARTTLS|TLS required/i.test(text)) return found('starttls','tls');
  }
  if(provider==='exchange') {
    if((status==='5.4.1'||status==='5.7.1')&&/relay (?:access )?denied|unable to relay/i.test(text)) return found('relay','connector');
    if(/Submission quota exceeded/i.test(text)) return found('sending-limit','rate');
    if(/MTA-STS/i.test(text)) return found(/certificate/i.test(text)?'certificate-chain':'mta-sts-host','tls');
    if(/RESOLVER\.ADR\.RecipNotFound/i.test(text)) return found('exchange-recipient','1');
    if(/RESOLVER\.RST\.NotAuthorized/i.test(text)) return found('exchange-group-allow','recipient-admin');
    if(/RESOLVER\.RST\.AuthRequired/i.test(text)) return found('exchange-internal-only','recipient-admin');
    if(/Client was not authenticated/i.test(text)) return found('exchange-auth','auth');
  }
  if(provider==='mimecast') {
    if(/(?:SPF|DKIM|DMARC) Sender Invalid/i.test(text)) return found(/DMARC/i.test(text)?'dmarc':/DKIM/i.test(text)?'dkim':'spf','authentication');
    if(/Anti-Spoofing/i.test(text)) return found('mimecast-spoof','authentication');
    if(/Envelope blocked|Administrative prohibition|Block policy|Blocked Senders/i.test(text)) return found('mimecast-block','recipient-admin');
    if(/Local CT IP Reputation|\bRBL\b/i.test(text)) return found('ip-block','reputation');
    if(/Content Examination/i.test(text)) return found('mimecast-content','content');
    if(/Invalid Recipient/i.test(text)) return found('exchange-recipient','1');
    if(/requires (?:encryption|TLS)/i.test(text)) return found('starttls','tls');
    if(/failed to authenticate|Incorrect authentication data/i.test(text)) return found('credentials','auth');
  }
  if(provider==='proofpoint') {
    if(/\bDeferred\b|\bthrottl/i.test(text)) return found('proofpoint-throttle','rate');
    if(/\bBlocked\b|blocked using prs\.proofpoint/i.test(text)) return found('proofpoint-block','reputation');
  }
  if(provider==='barracuda') {
    if(/blocked using Barracuda Reputation|\bBBL\b/i.test(text)) return found('barracuda-reputation','reputation');
    if(/Rejected due to (?:virus|spam)/i.test(text)) return found('content-security','content');
    if(/Recipient (?:email )?address rejected/i.test(text)) return found('exchange-recipient-access','recipient-admin');
  }
  if(provider==='fortimail'&&/Access denied|Relay access denied/i.test(text)) return found('fortimail-relay','connector');
  const codes=[...(text.match(/\b[45]\.\d{1,3}\.\d{1,3}\b/g)??[]),status];
  for(const code of codes) {
    const entry=provider==='exchange'?exchange[code]:provider==='gmail'?gmail[code]:undefined;
    if(entry) return found(...entry);
    if(provider==='exchange') {
      const match=/^([45])\.7\.(\d{3})$/.exec(code);
      const detail=Number(match?.[2]);
      if(match?.[1]==='5'&&detail>=606&&detail<=649) return found('ip-block','reputation');
      if(match?.[1]==='4'&&((detail>=500&&detail<=699)||(detail>=850&&detail<=899))) return found('exchange-temporary-restriction','rate');
    }
  }
  // Strong diagnostic phrases common to relays and email firewalls.
  if(/relay (?:access )?denied|not permitted to relay|unable to relay|relaying denied/i.test(text)) return found('relay','connector');
  if(/must issue (?:a )?STARTTLS|TLS (?:is )?required/i.test(text)) return {...found('starttls','tls'),sourceUrl:'https://www.rfc-editor.org/rfc/rfc3207.html'};
  if(/greylist(?:ed|ing)?/i.test(text)) return {...found('greylist','rate'),sourceUrl:'https://www.rfc-editor.org/rfc/rfc6647.html'};
  return null;
}
