/* global process, console, Buffer */
// Fixed mail set for the t32 restyle harness, delivered to the Greenmail account
// testuser@example.org over SMTP, then partly marked read over IMAP, so every
// stack starts from the same list: 30 messages, fixed senders, subjects and Date
// headers, one with an attachment, one with a long subject, ten already read (16-25).
//
//   MW_E2E_SMTP_HOST / MW_E2E_SMTP_PORT   Greenmail SMTP (stack.sh sets them)
//   MW_E2E_IMAP_PORT                      Greenmail IMAP (default SMTP port + 18)
//
// Run by stack.sh after the stack is healthy. Exits non-zero if any step fails.
import net from 'node:net';

const host = process.env.MW_E2E_SMTP_HOST ?? '127.0.0.1';
const smtpPort = Number(process.env.MW_E2E_SMTP_PORT ?? 3025);
const imapPort = Number(process.env.MW_E2E_IMAP_PORT ?? smtpPort + 18);
const to = 'testuser@example.org';

const SENDERS = [
  ['Ada Marsh', 'ada.marsh@example.net'],
  ['Bruno Keller', 'bruno.keller@example.net'],
  ['Chen Wei', 'chen.wei@example.net'],
  ['Dana Okafor', 'dana.okafor@example.net'],
  ['Elif Demir', 'elif.demir@example.net'],
  ['Farid Haddad', 'farid.haddad@example.net'],
];

const TOPICS = [
  'Quarterly report draft',
  'Invoice 2026-114',
  'Team lunch on Friday',
  'Server maintenance window',
  'Re: Contract renewal',
  'Travel booking confirmation',
  'Design review notes',
  'Weekly status',
  'Question about the API',
  'Holiday schedule',
];

/** The 30 messages, oldest first. Index 0 has the attachment, index 29 the long subject. */
export function messages() {
  const out = [];
  for (let i = 0; i < 30; i += 1) {
    const [name, addr] = SENDERS[i % SENDERS.length];
    let subject = `${TOPICS[i % TOPICS.length]} ${String(i + 1).padStart(2, '0')}`;
    if (i === 29) {
      subject =
        'Long subject 30: minutes of the planning meeting with every agenda item, decision, owner and follow-up date listed in full';
    }
    // 2026-10-01 08:00 UTC plus one hour per message.
    const date = new Date(Date.UTC(2026, 9, 1, 8 + i, 0, 0));
    out.push({
      from: `${name} <${addr}>`,
      fromAddr: addr,
      subject,
      date: date.toUTCString(),
      text: `Message ${i + 1} of the t32 seed.\r\n\r\nThis is the body of "${subject}".\r\nSecond paragraph with a link: https://example.org/t32/${i + 1}\r\n`,
      attachment: i === 0 ? { filename: 'report.pdf', contentType: 'application/pdf' } : null,
    });
  }
  return out;
}

function mime(m) {
  const head = `From: ${m.from}\r\nTo: ${to}\r\nSubject: ${m.subject}\r\nDate: ${m.date}\r\nMIME-Version: 1.0\r\n`;
  if (m.attachment === null) {
    return `${head}Content-Type: text/plain; charset=utf-8\r\n\r\n${m.text}`;
  }
  const bound = 't32seedboundary';
  // A minimal valid one-page PDF, so the attachment chip has a real type.
  const pdf = Buffer.from(
    '%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj 2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj ' +
      '3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 200 200]>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF\n',
  )
    .toString('base64')
    .replace(/(.{76})/g, '$1\r\n');
  return (
    `${head}Content-Type: multipart/mixed; boundary="${bound}"\r\n\r\n` +
    `--${bound}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n${m.text}\r\n` +
    `--${bound}\r\nContent-Type: ${m.attachment.contentType}; name="${m.attachment.filename}"\r\n` +
    `Content-Disposition: attachment; filename="${m.attachment.filename}"\r\n` +
    `Content-Transfer-Encoding: base64\r\n\r\n${pdf}\r\n--${bound}--\r\n`
  );
}

/** Drive a line protocol: send each step after the reply that `done` accepts. */
function converse(port, steps, done, fail) {
  return new Promise((resolve, reject) => {
    const sock = net.createConnection({ host, port });
    let step = -1;
    let buf = '';
    const stop = (e) => {
      sock.destroy();
      reject(e);
    };
    sock.setTimeout(15_000, () => stop(new Error(`timeout on port ${port} at step ${step}`)));
    sock.on('error', stop);
    sock.on('data', (chunk) => {
      buf += chunk.toString('utf8');
      const lines = buf.split('\r\n');
      const reply = lines.find((l) => done(l, step));
      if (reply === undefined) return;
      buf = '';
      const err = fail(reply, step);
      if (err) return stop(new Error(err));
      step += 1;
      if (step >= steps.length) {
        sock.end();
        return resolve();
      }
      sock.write(`${steps[step]}\r\n`);
    });
  });
}

async function send(m) {
  await converse(
    smtpPort,
    ['EHLO t32-seed', `MAIL FROM:<${m.fromAddr}>`, `RCPT TO:<${to}>`, 'DATA', `${mime(m)}\r\n.`, 'QUIT'],
    (l) => /^\d{3} /.test(l),
    (l) => (Number(l.slice(0, 3)) >= 400 ? `SMTP: ${l}` : null),
  );
}

async function markRead(range) {
  const steps = ['a1 LOGIN testuser testpass', 'a2 SELECT INBOX', `a3 STORE ${range} +FLAGS (\\Seen)`, 'a4 LOGOUT'];
  await converse(
    imapPort,
    steps,
    (l, step) => (step < 0 ? l.startsWith('* OK') : new RegExp(`^a${step + 1} `).test(l)),
    (l) => (/^a\d+ (NO|BAD)/.test(l) ? `IMAP: ${l}` : null),
  );
}

const all = messages();
for (const m of all) await send(m);
// Greenmail delivers synchronously, so the sequence numbers follow the send order.
// Messages 16-25 are read, so the first screen of the list mixes read and unread rows;
// the reader screen opens message 25, which is already read, so opening it changes nothing.
await markRead('16:25');
console.log(`seed.mjs: ${all.length} messages delivered to ${to}, 16:25 marked read`);
