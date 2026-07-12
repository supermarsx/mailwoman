// vCard 3.0 / 4.0 parse + emit (plan §3 e7, §2.1). A small hand-rolled reader/
// writer that maps the frozen `ContactCard` shape ⇄ vCard bytes for the
// import/export UI. The engine's `mw-ics` (`vcard4` crate) is the round-trip
// source of truth at integration (e10); this client-side pass powers the import
// *preview* + a local export without a server round-trip, so it stays lenient:
// unknown properties are ignored, both 3.0 and 4.0 spellings are accepted.

import type {
  Anniversary,
  ContactCard,
  ContactEmail,
  ContactName,
  ContactValue,
} from '../../api/pim-types.ts';

/** A parsed contact plus a stable client id (import preview needs a key). */
export type ParsedContact = Omit<ContactCard, 'id' | 'addressBookId' | 'etag'>;

interface VCardProperty {
  name: string;
  params: Record<string, string[]>;
  value: string;
}

// ── Reading ──────────────────────────────────────────────────────────────────

/** Unfold RFC 6350 line folding: a leading space/tab continues the prior line. */
function unfold(text: string): string[] {
  const raw = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n').split('\n');
  const out: string[] = [];
  for (const line of raw) {
    if ((line.startsWith(' ') || line.startsWith('\t')) && out.length > 0) {
      out[out.length - 1] += line.slice(1);
    } else {
      out.push(line);
    }
  }
  return out;
}

/** Split a string on `sep`, honouring double-quoted spans (for params). */
function splitQuoted(input: string, sep: string): string[] {
  const parts: string[] = [];
  let cur = '';
  let quoted = false;
  for (const ch of input) {
    if (ch === '"') {
      quoted = !quoted;
      cur += ch;
    } else if (ch === sep && !quoted) {
      parts.push(cur);
      cur = '';
    } else {
      cur += ch;
    }
  }
  parts.push(cur);
  return parts;
}

/** Unescape a vCard TEXT value (`\n` `\,` `\;` `\\`). */
function unescapeText(value: string): string {
  let out = '';
  for (let i = 0; i < value.length; i += 1) {
    const ch = value[i];
    if (ch === '\\' && i + 1 < value.length) {
      const next = value[i + 1]!;
      if (next === 'n' || next === 'N') out += '\n';
      else out += next;
      i += 1;
    } else {
      out += ch!;
    }
  }
  return out;
}

/** Split a structured/list value on unescaped `sep`. */
function splitValue(value: string, sep: string): string[] {
  const parts: string[] = [];
  let cur = '';
  for (let i = 0; i < value.length; i += 1) {
    const ch = value[i]!;
    if (ch === '\\' && i + 1 < value.length) {
      cur += ch + value[i + 1]!;
      i += 1;
    } else if (ch === sep) {
      parts.push(cur);
      cur = '';
    } else {
      cur += ch;
    }
  }
  parts.push(cur);
  return parts;
}

function parseLine(line: string): VCardProperty | null {
  // Split name+params from value at the first unquoted colon.
  const quoted = splitQuoted(line, ':');
  if (quoted.length < 2) return null;
  const head = quoted[0]!;
  const value = quoted.slice(1).join(':');
  const segs = splitQuoted(head, ';');
  let name = segs[0]!;
  // Drop any grouping prefix (`item1.EMAIL` → `EMAIL`).
  const dot = name.indexOf('.');
  if (dot >= 0) name = name.slice(dot + 1);
  name = name.trim().toUpperCase();
  const params: Record<string, string[]> = {};
  for (const seg of segs.slice(1)) {
    const eq = seg.indexOf('=');
    if (eq < 0) {
      // A bare `TYPE`-less param value (vCard 2.1 style): treat as a TYPE.
      const bare = seg.replace(/"/g, '').trim().toUpperCase();
      if (bare.length > 0) (params['TYPE'] ??= []).push(bare);
      continue;
    }
    const key = seg.slice(0, eq).trim().toUpperCase();
    const vals = splitQuoted(seg.slice(eq + 1), ',').map((v) => v.replace(/"/g, '').trim());
    (params[key] ??= []).push(...vals);
  }
  return { name, params, value };
}

/** First TYPE param, lowercased, as the Mailwoman `context` (e.g. `work`). */
function contextOf(prop: VCardProperty): string {
  const types = (prop.params['TYPE'] ?? []).map((t) => t.toLowerCase()).filter((t) => t !== 'pref');
  const first = types[0];
  if (first === undefined) return '';
  if (first === 'cell') return 'mobile';
  return first;
}

/** PREF weighting: vCard 4 `PREF=n`, or vCard 3 `TYPE=PREF` ⇒ 1. */
function prefOf(prop: VCardProperty): number {
  const p = prop.params['PREF']?.[0];
  if (p !== undefined) {
    const n = Number.parseInt(p, 10);
    if (Number.isFinite(n)) return n;
  }
  if ((prop.params['TYPE'] ?? []).some((t) => t.toLowerCase() === 'pref')) return 1;
  return 0;
}

function emptyName(): ContactName {
  return { full: '', given: '', surname: '', prefix: '', suffix: '' };
}

/** Parse a vCard document (possibly many cards) into contact drafts. */
export function parseVCards(text: string): ParsedContact[] {
  const lines = unfold(text);
  const cards: ParsedContact[] = [];
  let cur: ParsedContact | null = null;

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.length === 0) continue;
    const prop = parseLine(trimmed);
    if (prop === null) continue;

    if (prop.name === 'BEGIN' && prop.value.toUpperCase() === 'VCARD') {
      cur = blankContact();
      continue;
    }
    if (prop.name === 'END' && prop.value.toUpperCase() === 'VCARD') {
      if (cur !== null) cards.push(finalizeContact(cur));
      cur = null;
      continue;
    }
    if (cur === null) continue;
    applyProperty(cur, prop);
  }
  return cards;
}

function blankContact(): ParsedContact {
  return {
    uid: '',
    kind: 'individual',
    name: emptyName(),
    nicknames: [],
    organizations: [],
    titles: [],
    emails: [],
    phones: [],
    onlineServices: [],
    addresses: [],
    anniversaries: [],
    notes: '',
    photoBlobId: null,
    isFavorite: false,
    groupIds: [],
    pgpKey: null,
    smimeCert: null,
  };
}

function applyProperty(c: ParsedContact, prop: VCardProperty): void {
  const value = unescapeText(prop.value);
  switch (prop.name) {
    case 'UID':
      c.uid = value.replace(/^urn:uuid:/i, '');
      break;
    case 'KIND':
      c.kind = value.toLowerCase() === 'org' || value.toLowerCase() === 'organization' ? 'org' : 'individual';
      break;
    case 'FN':
      c.name.full = value;
      break;
    case 'N': {
      const [surname, given, , prefix, suffix] = splitValue(prop.value, ';').map(unescapeText);
      c.name.surname = surname ?? '';
      c.name.given = given ?? '';
      c.name.prefix = prefix ?? '';
      c.name.suffix = suffix ?? '';
      break;
    }
    case 'NICKNAME':
      c.nicknames.push(...splitValue(prop.value, ',').map(unescapeText).filter((n) => n.length > 0));
      break;
    case 'ORG':
      c.organizations.push(splitValue(prop.value, ';').map(unescapeText).filter((s) => s.length > 0).join(' · '));
      break;
    case 'TITLE':
    case 'ROLE':
      if (value.length > 0) c.titles.push(value);
      break;
    case 'EMAIL': {
      const email: ContactEmail = { context: contextOf(prop), value, pref: prefOf(prop) };
      c.emails.push(email);
      break;
    }
    case 'TEL': {
      const phone: ContactValue = { context: contextOf(prop), value };
      c.phones.push(phone);
      break;
    }
    case 'IMPP':
    case 'X-SOCIALPROFILE': {
      const svc: ContactValue = { context: contextOf(prop) || prop.name.replace(/^X-/, '').toLowerCase(), value };
      c.onlineServices.push(svc);
      break;
    }
    case 'ADR': {
      const [pobox, ext, street, locality, region, postcode, country] = splitValue(prop.value, ';').map(unescapeText);
      c.addresses.push({
        context: contextOf(prop),
        pobox: pobox ?? '',
        ext: ext ?? '',
        street: street ?? '',
        locality: locality ?? '',
        region: region ?? '',
        postcode: postcode ?? '',
        country: country ?? '',
      });
      break;
    }
    case 'BDAY':
      c.anniversaries.push({ kind: 'birthday', date: normalizeDate(value) });
      break;
    case 'ANNIVERSARY':
      c.anniversaries.push({ kind: 'anniversary', date: normalizeDate(value) });
      break;
    case 'NOTE':
      c.notes = c.notes.length > 0 ? `${c.notes}\n${value}` : value;
      break;
    case 'KEY':
      c.pgpKey = value;
      break;
    default:
      break;
  }
}

/** Coerce a vCard date (`19900101`, `1990-01-01`) to ISO `YYYY-MM-DD`. */
function normalizeDate(value: string): string {
  const compact = value.match(/^(\d{4})(\d{2})(\d{2})$/);
  if (compact) return `${compact[1]}-${compact[2]}-${compact[3]}`;
  return value;
}

function finalizeContact(c: ParsedContact): ParsedContact {
  // Derive a full name from the structured N when FN is absent.
  if (c.name.full.length === 0) {
    const parts = [c.name.prefix, c.name.given, c.name.surname, c.name.suffix].filter((p) => p.length > 0);
    c.name.full = parts.join(' ').trim();
  }
  if (c.name.full.length === 0 && c.organizations.length > 0) {
    c.name.full = c.organizations[0]!;
  }
  return c;
}

// ── Writing ──────────────────────────────────────────────────────────────────

function escapeText(value: string): string {
  return value.replace(/\\/g, '\\\\').replace(/\n/g, '\\n').replace(/,/g, '\\,').replace(/;/g, '\\;');
}

/** Fold a content line to <=75 chars per RFC 6350 (CRLF + leading space). */
function foldLine(line: string): string {
  if (line.length <= 75) return line;
  let out = line.slice(0, 75);
  let rest = line.slice(75);
  while (rest.length > 0) {
    out += `\r\n ${rest.slice(0, 74)}`;
    rest = rest.slice(74);
  }
  return out;
}

function emitProp(name: string, value: string, params: string[] = []): string {
  const head = params.length > 0 ? `${name};${params.join(';')}` : name;
  return foldLine(`${head}:${escapeText(value)}`);
}

/** Serialize one contact card as a vCard 4.0 entry. */
export function toVCard(card: ContactCard): string {
  const lines: string[] = ['BEGIN:VCARD', 'VERSION:4.0'];
  if (card.uid.length > 0) lines.push(emitProp('UID', card.uid));
  if (card.kind === 'org') lines.push(emitProp('KIND', 'org'));
  lines.push(emitProp('FN', card.name.full));
  lines.push(
    foldLine(
      `N:${[card.name.surname, card.name.given, '', card.name.prefix, card.name.suffix].map(escapeText).join(';')}`,
    ),
  );
  if (card.nicknames.length > 0) lines.push(emitProp('NICKNAME', card.nicknames.map(escapeText).join(',')));
  for (const org of card.organizations) lines.push(emitProp('ORG', org));
  for (const title of card.titles) lines.push(emitProp('TITLE', title));
  for (const email of card.emails) {
    const params: string[] = [];
    if (email.context.length > 0) params.push(`TYPE=${email.context}`);
    if (email.pref > 0) params.push(`PREF=${email.pref}`);
    lines.push(emitProp('EMAIL', email.value, params));
  }
  for (const phone of card.phones) {
    const params = phone.context.length > 0 ? [`TYPE=${phone.context}`] : [];
    lines.push(emitProp('TEL', phone.value, params));
  }
  for (const svc of card.onlineServices) {
    const params = svc.context.length > 0 ? [`TYPE=${svc.context}`] : [];
    lines.push(emitProp('IMPP', svc.value, params));
  }
  for (const anniv of card.anniversaries) {
    lines.push(emitProp(anniv.kind === 'birthday' ? 'BDAY' : 'ANNIVERSARY', anniv.date));
  }
  if (card.notes.length > 0) lines.push(emitProp('NOTE', card.notes));
  if (card.pgpKey !== null && card.pgpKey.length > 0) lines.push(emitProp('KEY', card.pgpKey));
  lines.push('END:VCARD');
  return lines.join('\r\n');
}

/** Serialize many cards as one vCard document. */
export function toVCardDocument(cards: ContactCard[]): string {
  return cards.map(toVCard).join('\r\n') + '\r\n';
}

/** Re-export helper so callers building anniversaries stay consistent. */
export function birthday(date: string): Anniversary {
  return { kind: 'birthday', date };
}
