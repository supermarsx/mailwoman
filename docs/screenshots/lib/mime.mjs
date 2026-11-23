// Demo messages (demo/mail/*.json) as RFC 5322 text (stack E, IMAP APPEND) and as
// JMAP Email objects (stack M, the mock's Email/set).
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { DEMO } from './paths.mjs';

const TYPES = { pdf: 'application/pdf', png: 'image/png', jpg: 'image/jpeg', txt: 'text/plain', ics: 'text/calendar' };

export const messageId = (id) => `${id}@demo.example.com`;

/** Resolve person keys and '@id' references of one dataset message. */
export function resolveMessage(message, people, byId) {
  const person = (key) => {
    const p = people.people[key];
    if (!p) throw new Error(`mail ${message.id}: unknown person '${key}' (people.json)`);
    return { name: p.name, email: p.email };
  };
  const chain = [];
  for (let parent = message.reply; parent; parent = byId.get(parent)?.reply) {
    if (!byId.has(parent)) throw new Error(`mail ${message.id}: reply to unknown message '${parent}'`);
    chain.unshift(messageId(parent));
  }
  const headers = Object.fromEntries(
    Object.entries(message.headers ?? {}).map(([k, v]) => [k, String(v).replace(/@([a-z0-9-]+)/g, (m, id) => (byId.has(id) ? `<${messageId(id)}>` : m))]),
  );
  return {
    ...message,
    fromAddr: person(message.from),
    toAddrs: (message.to ?? []).map(person),
    ccAddrs: (message.cc ?? []).map(person),
    messageId: messageId(message.id),
    inReplyTo: message.reply ? messageId(message.reply) : null,
    references: chain.length ? chain : null,
    headers,
    attachmentFiles: (message.attachments ?? []).map((name) => {
      const bytes = readFileSync(join(DEMO, 'attachments', name));
      return { name, bytes, type: TYPES[name.split('.').pop().toLowerCase()] ?? 'application/octet-stream' };
    }),
  };
}

/** Load a dataset file (relative to demo/) with people and the id index. */
export function loadMail(file) {
  const data = JSON.parse(readFileSync(join(DEMO, file), 'utf8'));
  const people = JSON.parse(readFileSync(join(DEMO, 'people.json'), 'utf8'));
  const byId = new Map(data.messages.map((m) => [m.id, m]));
  if (byId.size !== data.messages.length) throw new Error(`${file}: duplicate message id`);
  const messages = data.messages.map((m) => resolveMessage(m, people, byId)).sort((a, b) => a.date.localeCompare(b.date));
  return { folders: data.folders ?? [], messages };
}

/** The HTML body of a message: its own `html`, or its text as paragraphs. */
export function htmlOf(m) {
  if (m.html) return m.html;
  const esc = (t) => t.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  return m.text
    .trim()
    .split(/\n{2,}/)
    .map((p) => `<p>${esc(p).replace(/\n/g, '<br>')}</p>`)
    .join('\n');
}

// ── RFC 5322 ───────────────────────────────────────────────────────────────────

const ascii = (s) => /^[\x20-\x7e]*$/.test(s);
const word = (s) => (ascii(s) ? s : `=?UTF-8?B?${Buffer.from(s, 'utf8').toString('base64')}?=`);
const mailbox = ({ name, email }) => (name ? `${ascii(name) ? `"${name.replace(/"/g, '')}"` : word(name)} <${email}>` : `<${email}>`);
const wrap = (b64) => b64.replace(/(.{76})/g, '$1\r\n');

export function toMime(m) {
  const lines = [
    `From: ${mailbox(m.fromAddr)}`,
    ...(m.toAddrs.length ? [`To: ${m.toAddrs.map(mailbox).join(', ')}`] : []),
    ...(m.ccAddrs.length ? [`Cc: ${m.ccAddrs.map(mailbox).join(', ')}`] : []),
    `Subject: ${word(m.subject)}`,
    `Date: ${new Date(m.date).toUTCString().replace('GMT', '+0000')}`,
    `Message-ID: <${m.messageId}>`,
    ...(m.inReplyTo ? [`In-Reply-To: <${m.inReplyTo}>`] : []),
    ...(m.references ? [`References: ${m.references.map((r) => `<${r}>`).join(' ')}`] : []),
    ...Object.entries(m.headers).map(([k, v]) => `${k}: ${v}`),
    'MIME-Version: 1.0',
  ];
  const text = m.text.replace(/\r?\n/g, '\r\n');
  const b64 = (t) => wrap(Buffer.from(t, 'utf8').toString('base64'));
  const alt = `demo-${m.id}-alt`;
  // Text and HTML alternatives, as most mail programs send.
  const bodyPart = [
    `Content-Type: multipart/alternative; boundary="${alt}"`,
    '',
    `--${alt}`,
    'Content-Type: text/plain; charset=utf-8',
    'Content-Transfer-Encoding: base64',
    '',
    b64(text),
    `--${alt}`,
    'Content-Type: text/html; charset=utf-8',
    'Content-Transfer-Encoding: base64',
    '',
    b64(`<!doctype html><html><body>${htmlOf(m)}</body></html>`),
    `--${alt}--`,
    '',
  ];
  if (!m.attachmentFiles.length) return [...lines, ...bodyPart].join('\r\n');
  const boundary = `demo-${m.id}-boundary`;
  const parts = [`--${boundary}`, ...bodyPart];
  for (const a of m.attachmentFiles) {
    parts.push(
      `--${boundary}`,
      `Content-Type: ${a.type}; name="${a.name}"`,
      `Content-Disposition: attachment; filename="${a.name}"`,
      'Content-Transfer-Encoding: base64',
      '',
      wrap(a.bytes.toString('base64')),
    );
  }
  parts.push(`--${boundary}--`, '');
  return [...lines, `Content-Type: multipart/mixed; boundary="${boundary}"`, '', ...parts].join('\r\n');
}

// ── JMAP (mock) ────────────────────────────────────────────────────────────────

export function toJmap(m, mailboxId) {
  const keywords = {};
  if (!m.unread) keywords.$seen = true;
  if (m.flagged) keywords.$flagged = true;
  if (m.draft) keywords.$draft = true;
  for (const k of m.keywords ?? []) keywords[k] = true;
  const preview = m.text.replace(/\s+/g, ' ').trim().slice(0, 120);
  const html = htmlOf(m);
  const part = { partId: '1', type: 'text/plain', size: Buffer.byteLength(m.text) };
  const htmlPart = { partId: '2', type: 'text/html', size: Buffer.byteLength(html) };
  const email = {
    mailboxIds: { [mailboxId]: true },
    keywords,
    from: [m.fromAddr],
    to: m.toAddrs,
    cc: m.ccAddrs,
    subject: m.subject,
    messageId: [m.messageId],
    inReplyTo: m.inReplyTo ? [m.inReplyTo] : null,
    references: m.references,
    receivedAt: new Date(m.date).toISOString().replace('.000Z', 'Z'),
    sentAt: new Date(m.date).toISOString().replace('.000Z', 'Z'),
    preview,
    hasAttachment: m.attachmentFiles.length > 0,
    size: Buffer.byteLength(m.text) + m.attachmentFiles.reduce((n, a) => n + a.bytes.length, 0),
    bodyValues: { 1: { value: m.text, isTruncated: false }, 2: { value: html, isTruncated: false } },
    textBody: [part],
    htmlBody: [htmlPart],
    attachments: m.attachmentFiles.map((a, i) => ({
      partId: `a${i + 1}`,
      blobId: `demo-${m.id}-${i + 1}`,
      name: a.name,
      type: a.type,
      size: a.bytes.length,
      disposition: 'attachment',
    })),
  };
  for (const [k, v] of Object.entries(m.headers)) email[`header:${k}:asText`] = v;
  return email;
}
