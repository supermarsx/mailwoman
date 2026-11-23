// A minimal IMAP client for seeding Greenmail: LOGIN, CREATE, APPEND (with flags
// and an internal date), LOGOUT. Plaintext only; for the scratch container.
import net from 'node:net';

/** True when the server at `{host, port}` sends an IMAP greeting. */
export function imapGreeting({ host, port }) {
  return new Promise((ok) => {
    const socket = net.connect(port, host);
    socket.setTimeout(3000, () => {
      socket.destroy();
      ok(false);
    });
    socket.once('data', (chunk) => {
      socket.destroy();
      ok(chunk.toString().startsWith('* OK'));
    });
    socket.once('error', () => ok(false));
    // Docker's port proxy accepts and then closes while the container starts.
    socket.once('close', () => ok(false));
  });
}

/** A connected session; `command(text, literal?)` resolves with the tagged reply. */
export async function imapSession({ host, port }) {
  const socket = net.connect(port, host);
  socket.setTimeout(30_000, () => socket.destroy(new Error('IMAP timeout')));
  let buffer = '';
  let waiter = null;
  socket.on('data', (chunk) => {
    buffer += chunk.toString('utf8');
    waiter?.();
  });
  const failures = [];
  socket.on('error', (error) => failures.push(error));
  const nextLine = (test) =>
    new Promise((ok, fail) => {
      const check = () => {
        if (failures.length) return fail(failures[0]);
        let index;
        while ((index = buffer.indexOf('\r\n')) >= 0) {
          const line = buffer.slice(0, index);
          buffer = buffer.slice(index + 2);
          if (test(line)) {
            waiter = null;
            return ok(line);
          }
        }
        waiter = check;
      };
      check();
    });
  await nextLine((l) => l.startsWith('* OK'));
  let tag = 0;
  const command = async (text, literal) => {
    tag += 1;
    const t = `s${tag}`;
    if (literal === undefined) {
      socket.write(`${t} ${text}\r\n`);
    } else {
      const bytes = Buffer.from(literal, 'utf8');
      socket.write(`${t} ${text} {${bytes.length}}\r\n`);
      await nextLine((l) => l.startsWith('+') || l.startsWith(`${t} `));
      socket.write(bytes);
      socket.write('\r\n');
    }
    const reply = await nextLine((l) => l.startsWith(`${t} `));
    if (!reply.startsWith(`${t} OK`)) throw new Error(`IMAP ${text.split(' ')[0]}: ${reply}`);
    return reply;
  };
  return { command, close: () => socket.end() };
}

/** True when `user` can sign in at `{host, port}`. */
export async function imapLogin(server, user) {
  const s = await imapSession(server);
  try {
    await s.command(`LOGIN ${quote(user.username)} ${quote(user.password)}`);
    return true;
  } catch {
    return false;
  } finally {
    s.close();
  }
}

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

/** IMAP date-time ("10-Mar-2026 09:30:00 +0000") of an ISO instant. */
export function imapDate(iso) {
  const d = new Date(iso);
  const p = (n) => String(n).padStart(2, '0');
  return `${p(d.getUTCDate())}-${MONTHS[d.getUTCMonth()]}-${d.getUTCFullYear()} ${p(d.getUTCHours())}:${p(d.getUTCMinutes())}:${p(d.getUTCSeconds())} +0000`;
}

/** IMAP quoted string. */
export const quote = (s) => `"${String(s).replace(/["\\]/g, (c) => `\\${c}`)}"`;
