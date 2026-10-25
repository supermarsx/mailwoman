// SAML helpers for the admin SSO screen: read an IdP's metadata file and the
// signing certificates in it, in the browser (t32-cfg §4.12: metadata upload
// with a parsed preview, certificate list with expiry).
//
// The server signs nothing with these and fetches no metadata itself: a SAML
// backend stores the IdP's sign-in address, optional sign-out address and the
// certificates assertions are checked against (`SamlConfig`, crates/mw-sso). So
// the screen reads the metadata here, shows what it found, and on "Use these
// values" fills those fields; the metadata text is kept with the backend for
// reference.
//
// Certificates are read with a minimal DER walk (X.509 `tbsCertificate`: serial,
// algorithm, issuer, validity, subject) for the subject's common name and the
// validity dates. Nothing here verifies a signature.

export interface CertInfo {
  /** The PEM text as stored. */
  pem: string;
  /** Subject common name (or the whole subject when it has none); null when unreadable. */
  subject: string | null;
  notBefore: Date | null;
  notAfter: Date | null;
}

export type CertState = 'valid' | 'expiring' | 'expired' | 'not-yet-valid' | 'unreadable';

/** Days before expiry from which a certificate is shown as expiring. */
export const EXPIRING_DAYS = 30;

export interface IdpMetadata {
  entityId: string | null;
  /** HTTP-Redirect SingleSignOnService (the binding the server sends the request with). */
  ssoUrl: string | null;
  /** True when the IdP lists a sign-in address only for another binding. */
  ssoOtherBindingOnly: boolean;
  sloUrl: string | null;
  /** Signing certificates (KeyDescriptor use="signing" or without `use`), as PEM. */
  certsPem: string[];
}

export type MetadataResult = { ok: true; metadata: IdpMetadata } | { ok: false; reason: 'not-xml' | 'no-idp' };

const REDIRECT = 'urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect';


/** Wrap base64 certificate bytes as PEM (64 columns). */
export function toPem(base64: string): string {
  const body = base64.replace(/\s+/g, '');
  const lines = body.match(/.{1,64}/g) ?? [];
  return `-----BEGIN CERTIFICATE-----\n${lines.join('\n')}\n-----END CERTIFICATE-----`;
}

/** Every PEM certificate block in a text (pasted or from a file). */
export function splitPem(text: string): string[] {
  const out: string[] = [];
  const re = /-----BEGIN CERTIFICATE-----([\s\S]*?)-----END CERTIFICATE-----/g;
  for (const m of text.matchAll(re)) out.push(toPem(m[1] ?? ''));
  return out;
}

/** Parse IdP metadata XML. */
export function parseIdpMetadata(xml: string): MetadataResult {
  if (typeof DOMParser === 'undefined') return { ok: false, reason: 'not-xml' };
  const doc = new DOMParser().parseFromString(xml, 'application/xml');
  if (doc.getElementsByTagName('parsererror').length > 0 || doc.documentElement === null) return { ok: false, reason: 'not-xml' };
  const idp = doc.getElementsByTagNameNS('*', 'IDPSSODescriptor')[0];
  if (!idp) return { ok: false, reason: 'no-idp' };
  let entity: Element | null = idp.parentElement;
  while (entity !== null && entity.localName !== 'EntityDescriptor') entity = entity.parentElement;
  const services = [...idp.getElementsByTagNameNS('*', 'SingleSignOnService')];
  const redirect = services.find((s) => s.getAttribute('Binding') === REDIRECT);
  const slo = [...idp.getElementsByTagNameNS('*', 'SingleLogoutService')];
  const sloRedirect = slo.find((s) => s.getAttribute('Binding') === REDIRECT) ?? slo[0];
  const certsPem: string[] = [];
  for (const kd of idp.getElementsByTagNameNS('*', 'KeyDescriptor')) {
    const use = kd.getAttribute('use');
    if (use !== null && use !== 'signing') continue;
    for (const c of kd.getElementsByTagNameNS('*', 'X509Certificate')) {
      const text = (c.textContent ?? '').trim();
      if (text !== '') certsPem.push(toPem(text));
    }
  }
  return {
    ok: true,
    metadata: {
      entityId: entity?.getAttribute('entityID') ?? null,
      ssoUrl: redirect?.getAttribute('Location') ?? null,
      ssoOtherBindingOnly: redirect === undefined && services.length > 0,
      sloUrl: sloRedirect?.getAttribute('Location') ?? null,
      certsPem: [...new Set(certsPem)],
    },
  };
}

// ── DER ───────────────────────────────────────────────────────────────────────

interface Tlv {
  tag: number;
  /** Start of the value. */
  start: number;
  /** End of the value (exclusive). */
  end: number;
}

function readTlv(b: Uint8Array, at: number): Tlv {
  const tag = b[at];
  let len = b[at + 1];
  if (tag === undefined || len === undefined) throw new Error('truncated');
  let p = at + 2;
  if (len & 0x80) {
    const n = len & 0x7f;
    if (n === 0 || n > 4) throw new Error('length');
    len = 0;
    for (let i = 0; i < n; i += 1) {
      const byte = b[p + i];
      if (byte === undefined) throw new Error('truncated');
      len = len * 256 + byte;
    }
    p += n;
  }
  if (p + len > b.length) throw new Error('truncated');
  return { tag, start: p, end: p + len };
}

function children(b: Uint8Array, t: Tlv): Tlv[] {
  const out: Tlv[] = [];
  let p = t.start;
  while (p < t.end) {
    const c = readTlv(b, p);
    out.push(c);
    p = c.end;
  }
  return out;
}

const ascii = (b: Uint8Array, t: Tlv): string => new TextDecoder().decode(b.subarray(t.start, t.end));

function readTime(b: Uint8Array, t: Tlv): Date | null {
  const s = ascii(b, t);
  // UTCTime YYMMDDHHMMSSZ (years 50-99 are 19xx) or GeneralizedTime YYYYMMDDHHMMSSZ.
  const m = t.tag === 0x17 ? /^(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})?Z$/.exec(s) : /^(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})?Z$/.exec(s);
  if (!m) return null;
  let year = Number(m[1]);
  if (t.tag === 0x17) year += year >= 50 ? 1900 : 2000;
  return new Date(Date.UTC(year, Number(m[2]) - 1, Number(m[3]), Number(m[4]), Number(m[5]), Number(m[6] ?? 0)));
}

/** OID 2.5.4.3 (commonName) as DER bytes. */
const CN_OID = [0x55, 0x04, 0x03];

function readName(b: Uint8Array, name: Tlv): string | null {
  const parts: string[] = [];
  let cn: string | null = null;
  for (const rdn of children(b, name)) {
    for (const atv of children(b, rdn)) {
      const [oid, value] = children(b, atv);
      if (!oid || !value) continue;
      const text = ascii(b, value);
      parts.push(text);
      const o = b.subarray(oid.start, oid.end);
      if (o.length === 3 && CN_OID.every((x, i) => o[i] === x)) cn = text;
    }
  }
  return cn ?? (parts.length > 0 ? parts.join(', ') : null);
}

function base64Bytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i += 1) out[i] = bin.charCodeAt(i);
  return out;
}

/** Subject and validity of a PEM certificate; nulls when it cannot be read. */
export function certInfo(pem: string): CertInfo {
  const body = /-----BEGIN CERTIFICATE-----([\s\S]*?)-----END CERTIFICATE-----/.exec(pem)?.[1]?.replace(/\s+/g, '') ?? pem.replace(/\s+/g, '');
  try {
    const b = base64Bytes(body);
    const cert = readTlv(b, 0);
    const [tbsTlv] = children(b, cert);
    if (!tbsTlv) throw new Error('no tbs');
    const tbs = children(b, tbsTlv);
    // An explicit version is the context tag [0]; skip it.
    const fields = tbs[0]?.tag === 0xa0 ? tbs.slice(1) : tbs;
    const validity = fields[3];
    const subject = fields[4];
    if (!validity || !subject) throw new Error('short tbs');
    const [nb, na] = children(b, validity);
    return {
      pem,
      subject: readName(b, subject),
      notBefore: nb ? readTime(b, nb) : null,
      notAfter: na ? readTime(b, na) : null,
    };
  } catch {
    return { pem, subject: null, notBefore: null, notAfter: null };
  }
}

/** Where a certificate stands at `now`. */
export function certState(c: CertInfo, now: Date = new Date()): CertState {
  if (c.notAfter === null) return 'unreadable';
  if (c.notAfter.getTime() <= now.getTime()) return 'expired';
  if (c.notBefore !== null && c.notBefore.getTime() > now.getTime()) return 'not-yet-valid';
  if (c.notAfter.getTime() - now.getTime() < EXPIRING_DAYS * 86_400_000) return 'expiring';
  return 'valid';
}

/** Whole days from `now` to the certificate's end (negative once expired). */
export function daysLeft(c: CertInfo, now: Date = new Date()): number | null {
  return c.notAfter === null ? null : Math.floor((c.notAfter.getTime() - now.getTime()) / 86_400_000);
}
