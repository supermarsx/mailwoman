// CSV contact import (with field mapping) + export (plan §3 e7). CSV has no
// standard contact schema, so import is a two-step flow: parse the sheet into
// rows, let the user map each column to a `ContactCard` field, then materialize
// drafts. Export flattens the structured card to a spreadsheet-friendly table.

import type { ContactCard } from '../../api/pim-types.ts';
import type { ParsedContact } from './vcard.ts';

/** The `ContactCard` fields a CSV column can be mapped onto. */
export type CsvField =
  | 'ignore'
  | 'fullName'
  | 'given'
  | 'surname'
  | 'prefix'
  | 'suffix'
  | 'nickname'
  | 'organization'
  | 'title'
  | 'email'
  | 'phone'
  | 'birthday'
  | 'notes';

/** Column-index → target field. Length matches the parsed header row. */
export type CsvMapping = CsvField[];

export interface ParsedCsv {
  headers: string[];
  rows: string[][];
}

// ── Reading ──────────────────────────────────────────────────────────────────

/** RFC 4180-ish CSV parse: quoted fields, escaped `""`, CRLF/LF rows. */
export function parseCsv(text: string): ParsedCsv {
  const rows: string[][] = [];
  let field = '';
  let row: string[] = [];
  let quoted = false;
  const src = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');

  for (let i = 0; i < src.length; i += 1) {
    const ch = src[i]!;
    if (quoted) {
      if (ch === '"') {
        if (src[i + 1] === '"') {
          field += '"';
          i += 1;
        } else {
          quoted = false;
        }
      } else {
        field += ch;
      }
    } else if (ch === '"') {
      quoted = true;
    } else if (ch === ',') {
      row.push(field);
      field = '';
    } else if (ch === '\n') {
      row.push(field);
      rows.push(row);
      field = '';
      row = [];
    } else {
      field += ch;
    }
  }
  // Flush the trailing field/row unless the input ended on a clean newline.
  if (field.length > 0 || row.length > 0) {
    row.push(field);
    rows.push(row);
  }

  const headers = rows.shift() ?? [];
  // Drop fully-empty trailing rows.
  const clean = rows.filter((r) => r.some((c) => c.trim().length > 0));
  return { headers: headers.map((h) => h.trim()), rows: clean };
}

/** Common header spellings → a best-guess field, for a sensible default mapping. */
const HEADER_GUESSES: Array<[RegExp, CsvField]> = [
  [/^(full ?name|display ?name|name)$/i, 'fullName'],
  [/^(first ?name|given ?name|given)$/i, 'given'],
  [/^(last ?name|surname|family ?name)$/i, 'surname'],
  [/^(prefix|title ?prefix|honorific)$/i, 'prefix'],
  [/^(suffix)$/i, 'suffix'],
  [/^(nick ?name)$/i, 'nickname'],
  [/^(org(anization)?|company|employer)$/i, 'organization'],
  [/^(job ?title|title|role|position)$/i, 'title'],
  [/^(e-?mail.*|email ?address)$/i, 'email'],
  [/^(phone.*|tel(ephone)?.*|mobile|cell)$/i, 'phone'],
  [/^(birthday|bday|date of birth|dob)$/i, 'birthday'],
  [/^(notes?|comments?)$/i, 'notes'],
];

/** Propose a mapping from the header labels (columns default to `ignore`). */
export function guessMapping(headers: string[]): CsvMapping {
  return headers.map((h) => {
    for (const [re, field] of HEADER_GUESSES) {
      if (re.test(h.trim())) return field;
    }
    return 'ignore';
  });
}

function blank(): ParsedContact {
  return {
    uid: '',
    kind: 'individual',
    name: { full: '', given: '', surname: '', prefix: '', suffix: '' },
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

/** Materialize contact drafts from parsed rows under the chosen mapping. */
export function csvToContacts(parsed: ParsedCsv, mapping: CsvMapping): ParsedContact[] {
  const out: ParsedContact[] = [];
  for (const row of parsed.rows) {
    const c = blank();
    mapping.forEach((field, col) => {
      const raw = (row[col] ?? '').trim();
      if (raw.length === 0 || field === 'ignore') return;
      switch (field) {
        case 'fullName':
          c.name.full = raw;
          break;
        case 'given':
          c.name.given = raw;
          break;
        case 'surname':
          c.name.surname = raw;
          break;
        case 'prefix':
          c.name.prefix = raw;
          break;
        case 'suffix':
          c.name.suffix = raw;
          break;
        case 'nickname':
          c.nicknames.push(raw);
          break;
        case 'organization':
          c.organizations.push(raw);
          break;
        case 'title':
          c.titles.push(raw);
          break;
        case 'email':
          c.emails.push({ context: '', value: raw, pref: 0 });
          break;
        case 'phone':
          c.phones.push({ context: '', value: raw });
          break;
        case 'birthday':
          c.anniversaries.push({ kind: 'birthday', date: raw });
          break;
        case 'notes':
          c.notes = c.notes.length > 0 ? `${c.notes}\n${raw}` : raw;
          break;
        default:
          break;
      }
    });
    if (c.name.full.length === 0) {
      const parts = [c.name.given, c.name.surname].filter((p) => p.length > 0);
      c.name.full = parts.length > 0 ? parts.join(' ') : (c.emails[0]?.value ?? c.organizations[0] ?? '');
    }
    // Skip a row that carried no usable content.
    if (c.name.full.length > 0 || c.emails.length > 0 || c.phones.length > 0) out.push(c);
  }
  return out;
}

// ── Writing ──────────────────────────────────────────────────────────────────

const EXPORT_HEADERS = [
  'Full Name',
  'Given Name',
  'Surname',
  'Nickname',
  'Organization',
  'Title',
  'Email',
  'Phone',
  'Birthday',
  'Notes',
] as const;

function csvCell(value: string): string {
  if (/[",\n]/.test(value)) return `"${value.replace(/"/g, '""')}"`;
  return value;
}

/** Export cards to a CSV sheet (one row per card; primary email/phone). */
export function contactsToCsv(cards: ContactCard[]): string {
  const lines = [EXPORT_HEADERS.join(',')];
  for (const card of cards) {
    const primaryEmail = [...card.emails].sort((a, b) => (b.pref || 0) - (a.pref || 0))[0]?.value ?? '';
    const primaryPhone = card.phones[0]?.value ?? '';
    const bday = card.anniversaries.find((a) => a.kind === 'birthday')?.date ?? '';
    const cells = [
      card.name.full,
      card.name.given,
      card.name.surname,
      card.nicknames.join('; '),
      card.organizations.join('; '),
      card.titles.join('; '),
      primaryEmail,
      primaryPhone,
      bday,
      card.notes,
    ];
    lines.push(cells.map(csvCell).join(','));
  }
  return lines.join('\r\n') + '\r\n';
}
