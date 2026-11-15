// Pure composition builders shared by tabs and background replay.
import {CAP_CORE,CAP_MAIL,CAP_SUBMISSION,type EmailAddress,type Id,type Invocation,type JmapRequest} from './jmap-types.ts';
import {request} from './jmapWire.ts';
const SUBMISSION_USING=[CAP_CORE,CAP_MAIL,CAP_SUBMISSION];
export interface DraftInput {
  accountDraft?:{id:string;revision:string};
  from: EmailAddress;
  to: string;
  /** `Cc` recipients, in the same field syntax as `to`. */
  cc?: string;
  /** `Bcc` recipients, in the same field syntax as `to`. They are set on the
   *  draft's `bcc` and added to the envelope; keeping the `Bcc` header out of
   *  what the other recipients receive is the server's part (RFC 8621 §7.5). */
  bcc?: string;
  /** The message ids this one answers, without angle brackets. */
  inReplyTo?: string[];
  /** The thread's message ids, oldest first, without angle brackets. */
  references?: string[];
  subject: string;
  htmlBody: string;
  /** Explicit plain text mode; when present no HTML body is sent. */
  textBody?: string;
  importance?: 'high'|'normal'|'low';
  replyTo?: EmailAddress[];
  organization?: string;
  /** Send without keeping a copy: the Email is destroyed once the submission succeeded. */
  skipSentCopy?: boolean;
  /** Flag the sent copy for follow-up at this time (`Remind me if there is no reply`). */
  followUpAt?: string;
  /** Extra text headers (name, value); the engine writes only the ones it allows. */
  headers?: [string, string][];
  draftMailboxId: Id;
  sentMailboxId?: Id;
  /** V2: send via a specific sending identity (§2.1 `Identity`). */
  identityId?: Id;
  /** V2 send-later: scheduled send time (ISO 8601 UTC); omitted = send now. */
  sendAt?: string;
  /** V7 (§18.4): blob attachments materialised server-side (e.g. from Nextcloud),
   *  referenced by blob id on the draft. Omitted for an ordinary send. */
  attachments?: { blobId: Id; name: string; type: string; size?: number }[];
  /** Images the HTML body shows by `cid:` (t29-e12): each becomes an inline
   *  part of a `multipart/related` message. Omitted when there are none. */
  inline?: { blobId: Id; cid: string; disposition: 'inline'; type: string; name: string }[];
  /** Ask the recipient's mail program for a read receipt (t29-e12). The engine
   *  writes `Disposition-Notification-To` with the validated From mailbox; no
   *  other address can be named. */
  requestReadReceipt?: boolean;
  /** Request SMTP DSN success/failure/delay notifications; never a read receipt. */
  requestDeliveryReceipt?: boolean;
  /**
   * V2 undo-send: engine-held delay before SMTP dispatch, in seconds. The
   * client shows a Cancel toast for this window; the engine only dials SMTP
   * after it elapses (plan §1.3).
   */
  holdSeconds?: number;
}

/** Every recipient of a draft, once each (first spelling wins), in To, Cc, Bcc
 *  order: the submission's `rcptTo`. */
function envelopeRecipients(input: DraftInput): { email: string }[] {
  const seen = new Set<string>();
  const out: { email: string }[] = [];
  for (const field of [input.to, input.cc ?? '', input.bcc ?? '']) {
    for (const r of parseRecipients(field)) {
      const key = r.email.toLowerCase();
      if (seen.has(key)) continue;
      seen.add(key);
      out.push({ email: r.email });
    }
  }
  return out;
}

/**
 * The `Email/set` create object for a composition. `cc`, `bcc`, `inReplyTo`
 * and `references` are added only when they hold something, so a message
 * without them is the same object it was before they existed.
 */
export function draftCreateSpec(input: DraftInput): Record<string, unknown> {
  const spec: Record<string, unknown> = {
    mailboxIds: { [input.draftMailboxId]: true },
    keywords: {
      $draft: true,
      $seen: true,
      ...(input.requestReadReceipt ? { $mailwomanReadReceiptRequested: true } : {}),
      ...(input.requestDeliveryReceipt ? { $mailwomanDeliveryReceiptRequested: true } : {}),
    },
    from: [input.from],
    to: parseRecipients(input.to),
  };
  if(input.replyTo?.length)spec['replyTo']=input.replyTo;
  if(input.organization&&!Array.from(input.organization).some(char=>char.charCodeAt(0)<32||char.charCodeAt(0)===127))spec['header:Organization:asText']=input.organization;
  const cc = parseRecipients(input.cc ?? '');
  if (cc.length > 0) spec['cc'] = cc;
  const bcc = parseRecipients(input.bcc ?? '');
  if (bcc.length > 0) spec['bcc'] = bcc;
  if (input.inReplyTo !== undefined && input.inReplyTo.length > 0) spec['inReplyTo'] = [...input.inReplyTo];
  if (input.references !== undefined && input.references.length > 0) spec['references'] = [...input.references];
  spec['subject'] = input.subject;
  if(input.importance){spec['header:Importance:asText']=input.importance;spec['header:X-Priority:asText']=({high:'1',normal:'3',low:'5'} as const)[input.importance];}
  for(const [name,value] of input.headers??[]){
    // Header names are tokens and values one line: anything else never reaches the wire.
    // eslint-disable-next-line no-control-regex -- control characters are exactly what is rejected
    if(!/^[A-Za-z0-9!#$%&'*+.^_`|~-]{1,64}$/.test(name)||/[\u0000-\u001f\u007f\u0085\u2028\u2029]/.test(value)||value.length>998)continue;
    spec[`header:${name}:asText`]=value;
  }
  const plain = input.textBody !== undefined;
  spec[plain ? 'textBody' : 'htmlBody'] = [{ partId: 'body', type: plain ? 'text/plain' : 'text/html' }];
  spec['bodyValues'] = { body: { value: input.textBody ?? input.htmlBody } };
  if (input.requestReadReceipt === true) spec['mailwomanRequestReadReceipt'] = true;
  return spec;
}

/**
 * Compose + send in ONE request using creation-id back-references:
 * Email/set creates the draft under key `draft`; EmailSubmission/set references
 * it as `#draft`; on success the email is moved out of Drafts into Sent.
 */
export function sendEnvelope(accountId: Id, input: DraftInput): JmapRequest {
  const draftCreate = draftCreateSpec(input);
  // V7 (§18.4): server-materialised blob attachments (e.g. from Nextcloud) ride the
  // draft create as standard RFC 8621 `attachments` blob references. Added only when
  // present, so an ordinary send is byte-identical to before. Images shown in the
  // body (t29-e12) ride the same array with `disposition: 'inline'` and a `cid`.
  const attached: Record<string, unknown>[] = (input.attachments ?? []).map((a) => ({
    blobId: a.blobId,
    name: a.name,
    type: a.type,
    ...(a.size !== undefined ? { size: a.size } : {}),
    disposition: 'attachment',
  }));
  for (const i of input.inline ?? []) {
    attached.push({ blobId: i.blobId, cid: i.cid, disposition: 'inline', type: i.type, name: i.name });
  }
  if (attached.length > 0) draftCreate['attachments'] = attached;
  const emailSet: Invocation = ['Email/set', { accountId, create: { draft: draftCreate } }, 'set'];

  const onSuccessUpdate: Record<string, Record<string, unknown>> = {};
  const destroyCopy = input.skipSentCopy === true;
  if (input.sentMailboxId !== undefined || input.followUpAt !== undefined) {
    onSuccessUpdate['#send'] = {
      ...(input.sentMailboxId !== undefined ? { mailboxIds: { [input.sentMailboxId]: true } } : {}),
      'keywords/$draft': null,
      ...(input.followUpAt !== undefined ? { followUpAt: input.followUpAt } : {}),
    };
  }

  // V2 (§2.1): the submission is a persisted, engine-held row. `sendAt` /
  // `mailwomanHoldSeconds` ride the create so the engine can enqueue instead of
  // dialing SMTP synchronously; both are extra fields on the frozen submission
  // object, so no seam change is needed.
  const sendCreate: Record<string, unknown> = {
    emailId: '#draft',
    envelope: {
      mailFrom: { email: input.from.email, ...(input.requestDeliveryReceipt ? {parameters:{RET:'HDRS'}} : {}) },
      rcptTo: envelopeRecipients(input).map(recipient => ({...recipient,...(input.requestDeliveryReceipt ? {parameters:{NOTIFY:'SUCCESS,FAILURE,DELAY'}} : {})})),
    },
  };
  if(input.accountDraft)sendCreate['mailwomanAccountDraft']=input.accountDraft;
  if (input.identityId !== undefined) sendCreate['identityId'] = input.identityId;
  if (input.sendAt !== undefined) sendCreate['sendAt'] = input.sendAt;
  if (input.holdSeconds !== undefined) sendCreate['mailwomanHoldSeconds'] = input.holdSeconds;

  const submissionSet: Invocation = [
    'EmailSubmission/set',
    {
      accountId,
      create: { send: sendCreate },
      ...((input.sentMailboxId !== undefined || input.followUpAt !== undefined) && !destroyCopy ? { onSuccessUpdateEmail: onSuccessUpdate } : {}),
      ...(destroyCopy ? { onSuccessDestroyEmail: ['#send'] } : {}),
    },
    'submit',
  ];

  return request(SUBMISSION_USING, [emailSet, submissionSet]);
}

/**
 * Split an address-list field into its entries. `,` and `;` separate entries
 * except inside a double-quoted display name or between `<` and `>`.
 */
export function splitAddressList(raw: string): string[] {
  const out: string[] = [];
  let cur = '';
  let quoted = false;
  let angle = false;
  for (let i = 0; i < raw.length; i += 1) {
    const c = raw[i]!;
    if (quoted && c === '\\' && i + 1 < raw.length) {
      cur += c + raw[i + 1]!;
      i += 1;
      continue;
    }
    if (c === '"') quoted = !quoted;
    else if (c === '<' && !quoted) angle = true;
    else if (c === '>' && !quoted) angle = false;
    if ((c === ',' || c === ';') && !quoted && !angle) {
      out.push(cur);
      cur = '';
    } else {
      cur += c;
    }
  }
  out.push(cur);
  return out.map((s) => s.trim()).filter((s) => s.length > 0);
}

/** The display-name half of `Name <addr>`: unquoted, on one line, or `null`. */
function displayName(raw: string): string | null {
  let name = raw.trim();
  if (name.length >= 2 && name.startsWith('"') && name.endsWith('"')) {
    name = name.slice(1, -1).replace(/\\(.)/g, '$1');
  }
  name = name.replace(/[\p{Cc}\s]+/gu, ' ').trim();
  return name.length > 0 ? name : null;
}

/**
 * Parse a recipient field into `{ name, email }` entries. The one parser for
 * every consumer: the draft's `to`, the submission's `rcptTo`, and the
 * composer's per-recipient key lookup.
 *
 * `Name <addr>` — the form the contact autocomplete inserts — yields the
 * display name and the addr-spec separately; a bare token is an address with no
 * name. An entry that is neither (text after the `>`, say) is returned whole as
 * its `email`, so `isMailbox` refuses it rather than a repaired guess being sent.
 */
export function parseRecipients(raw: string): EmailAddress[] {
  return splitAddressList(raw).map((entry) => {
    let quoted=false,open=-1,close=-1,invalid=false;
    for(let i=0;i<entry.length;i++){
      const c=entry[i]!;
      if(c==='\\'&&quoted){i++;continue;}
      if(c==='"'){quoted=!quoted;continue;}
      if(!quoted&&c==='<'){if(open!==-1)invalid=true;open=i;}
      if(!quoted&&c==='>'){if(open===-1||close!==-1)invalid=true;close=i;}
    }
    if(invalid||quoted||open===-1||close!==entry.length-1)return {name:null,email:entry};
    return {name:displayName(entry.slice(0,open)),email:entry.slice(open+1,close).trim()};
  });
}

/** SMTP path excludes its enclosing angle brackets: at most 254 UTF-8 octets. */
const MAX_ADDR_BYTES = 254;

/**
 * Whether the server will accept `addr` as a recipient. This restates
 * `validate_mailbox` in `crates/mw-smtp/src/addr.rs`. Internationalized dot-atoms
 * and quoted local parts are preserved byte-for-byte. IDNA conversion is only
 * used to validate DNS labels, never to rewrite the submitted address.
 */
export function isMailbox(addr: string): boolean {
  if (addr.length === 0 || new TextEncoder().encode(addr).length > MAX_ADDR_BYTES) return false;
  if (/[\p{Cc}\p{Cs}]/u.test(addr)) return false;
  let separator=-1;
  if(addr.startsWith('"')){
    for(let i=1;i<addr.length;i++){
      const c=addr[i]!;
      if(c==='\\'){const escaped=addr[++i];if(!escaped||escaped.charCodeAt(0)<32||escaped.charCodeAt(0)>126)return false;continue;}
      if(c==='"'){if(addr[i+1]!=='@')return false;separator=i+1;break;}
      if(c.charCodeAt(0)>127&&/\p{White_Space}/u.test(c))return false;
    }
  }else{
    separator=addr.indexOf('@');
    const local=addr.slice(0,separator);
    if(!local||local.startsWith('.')||local.endsWith('.')||local.includes('..'))return false;
    for(const c of local)if(c.charCodeAt(0)<128?!/[A-Za-z0-9.!#$%&'*+/=?^_`{|}~-]/.test(c):/\p{White_Space}/u.test(c))return false;
  }
  if(separator<1||new TextEncoder().encode(addr.slice(0,separator)).length>64)return false;
  const domain=addr.slice(separator+1);
  if(!domain||/[\p{White_Space}@%/\\?#]/u.test(domain))return false;
  if(domain.startsWith('[')&&domain.endsWith(']')){
    const literal=domain.slice(1,-1);
    if(literal.startsWith('IPv6:')){try{return new URL(`http://[${literal.slice(5)}]`).hostname.startsWith('[');}catch{return false;}}
    const octets=literal.split('.');return octets.length===4&&octets.every(part=>/^(?:0|[1-9]\d{0,2})$/.test(part)&&Number(part)<=255);
  }
  if(/[:[\]]/.test(domain)||domain.endsWith('.'))return false;
  try{const ascii=new URL(`http://${domain}`).hostname;return ascii.length<=253&&ascii.split('.').every(label=>label.length>=1&&label.length<=63&&/^[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$/i.test(label));}catch{return false;}
}

/** The entries of a recipient field that are not a mailbox, as typed. */
export function invalidRecipients(raw: string): string[] {
  return parseRecipients(raw)
    .map((r) => r.email)
    .filter((email) => !isMailbox(email));
}

