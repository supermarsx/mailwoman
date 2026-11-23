// Start and stop the application stacks the screenshots are taken from.
//
//   M  proxy mode: the server fronting the in-repo mock JMAP server (mail,
//      settings, admin console). No Docker. Mail is seeded over JMAP.
//   E  engine mode: the server driving a Greenmail container over IMAP/SMTP. The
//      only stack with calendar, contacts, tasks and notes, real threads, folder
//      roles, unread counts and search. Mail is seeded over IMAP APPEND.
//
// Both use a private copy of the server binary, a scratch database, free ports and
// the web client built from the current source (MW_WEB_DIR). Run directly to keep
// a stack up for exploring:  node docs/screenshots/stack.mjs M|E [--seed a,b]
import { spawn, execFileSync } from 'node:child_process';
import { createWriteStream, mkdirSync, readFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { DEMO, SCRATCH, freePort, log, waitFor } from './lib/paths.mjs';
import { ensureWeb, mockBinary, serverBinary } from './lib/prepare.mjs';

/** The demo user and the console administrator every stack signs in as. */
export function accounts() {
  const people = JSON.parse(readFileSync(join(DEMO, 'people.json'), 'utf8'));
  const me = people.people[people.me];
  return {
    user: { username: me.email, password: people.password, name: me.name },
    admin: { ...people.admin },
  };
}

const ADMIN_PATH = '/admin';
const CONTAINER_PREFIX = process.env.MW_SHOTS_CONTAINER_PREFIX ?? 't40-shots';

function docker(...args) {
  return execFileSync('docker', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
}

/**
 * Start stack `kind` ('M' or 'E'). `beforeServer(stack)` runs after the mail
 * backend is up and before the server starts (mail seeding). Returns the stack;
 * always call `stop()`.
 */
export async function startStack(kind, { webDir, webHash, runDir, beforeServer } = {}) {
  if (kind !== 'M' && kind !== 'E') throw new Error(`unknown stack ${kind} (M or E)`);
  const { user, admin } = accounts();
  const web = webDir && webHash ? { dir: webDir, hash: webHash } : await ensureWeb(webDir);
  const server = serverBinary();
  const dir = runDir ?? join(SCRATCH, 'runs', `${kind}-${Date.now()}-${process.pid}`);
  mkdirSync(dir, { recursive: true });
  const children = [];
  const containers = [];
  let stopped = false;

  const stop = () => {
    if (stopped) return;
    stopped = true;
    for (const child of children) {
      try {
        child.kill();
      } catch {
        /* gone */
      }
    }
    for (const name of containers) {
      try {
        docker('rm', '-f', name);
      } catch {
        /* gone */
      }
    }
  };
  const onExit = () => stop();
  process.once('exit', onExit);

  const run = (name, command, args, env) => {
    const out = createWriteStream(join(dir, `${name}.log`));
    const child = spawn(command, args, { cwd: dir, env: { ...process.env, ...env }, stdio: ['ignore', 'pipe', 'pipe'] });
    child.stdout.pipe(out);
    child.stderr.pipe(out);
    child.on('exit', (code) => {
      if (!stopped) log(`${name} exited early (${code}); see ${join(dir, `${name}.log`)}`);
    });
    children.push(child);
    return child;
  };

  const stack = {
    kind,
    dir,
    user,
    admin,
    adminPath: ADMIN_PATH,
    versions: { serverBinary: server.source, serverBuilt: server.built, web: web.hash },
    stop,
  };

  try {
    const env = {};
    if (kind === 'M') {
      const mock = mockBinary();
      stack.versions.mockBinary = mock.source;
      stack.versions.mockBuilt = mock.built;
      const mockPort = await freePort();
      run('mock', mock.path, [], {
        MW_MOCK_PORT: String(mockPort),
        MW_MOCK_USER: user.username,
        MW_MOCK_PASS: user.password,
        MW_MOCK_NAME: user.name,
        RUST_LOG: 'warn',
      });
      stack.jmapUrl = `http://127.0.0.1:${mockPort}`;
      await waitFor(async () => (await fetch(`${stack.jmapUrl}/healthz`)).ok, 'the mock JMAP server', 30_000);
      Object.assign(env, {
        MW_MODE: 'proxy',
        MW_JMAP_UPSTREAMS: stack.jmapUrl,
        MW_UPSTREAM_DEFAULT: `${stack.jmapUrl}/.well-known/jmap`,
      });
    } else {
      const [imap, smtp] = [await freePort(), await freePort()];
      const name = `${CONTAINER_PREFIX}-greenmail-${imap}`;
      containers.push(name);
      try {
        docker('rm', '-f', name);
      } catch {
        /* not there */
      }
      // The demo account, signing in with its full address.
      const [local, domain] = user.username.split('@');
      docker(
        'run', '-d', '--name', name, '-p', `127.0.0.1:${smtp}:3025`, '-p', `127.0.0.1:${imap}:3143`,
        '-e', `GREENMAIL_OPTS=-Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.users=${local}:${user.password}@${domain} -Dgreenmail.users.login=email`,
        'greenmail/standalone:2.1.4',
      );
      stack.imap = { host: '127.0.0.1', port: imap };
      stack.smtp = { host: '127.0.0.1', port: smtp };
      stack.container = name;
      const { imapGreeting, imapLogin } = await import('./lib/imap.mjs');
      await waitFor(() => imapGreeting(stack.imap), 'Greenmail IMAP', 90_000);
      // Greenmail greets before it has created the preset user.
      await waitFor(() => imapLogin(stack.imap, user), 'the Greenmail demo account', 60_000);
      Object.assign(env, {
        MW_MODE: 'engine',
        MW_UPSTREAM_DEFAULT: `imap://127.0.0.1:${imap}`,
        MW_ENGINE_TLS: 'plaintext',
        MW_SMTP_HOST: '127.0.0.1',
        MW_SMTP_PORT: String(smtp),
        MW_SMTP_SECURITY: 'plaintext',
        MW_ALLOWED_FROMS: user.username,
      });
    }
    if (beforeServer) await beforeServer(stack);

    const port = await freePort();
    stack.base = `http://localhost:${port}`;
    run('server', server.path, ['serve'], {
      ...env,
      MW_BIND: `127.0.0.1:${port}`,
      MW_DB_PATH: join(dir, 'mailwoman.db'),
      MW_UPLOAD_DIR: join(dir, 'uploads'),
      MW_SERVER_KEY: '5ce7e5ce7e5ce7e5ce7e5ce7e5ce7e5ce7e5ce7e5ce7e5ce7e5ce7e5ce7e5ce7',
      MW_WEB_DIR: web.dir,
      MW_PUBLIC_URL: stack.base,
      MW_WEBAUTHN_RP_ID: 'localhost',
      MW_WEBAUTHN_ORIGIN: stack.base,
      MW_COOKIE_SECURE: 'false',
      MW_CSRF_STRICT: 'true',
      MW_ADMIN_ENABLED: 'true',
      MW_ADMIN_USER: admin.username,
      MW_ADMIN_PASSWORD: admin.password,
      MW_ADMIN_PATH: ADMIN_PATH,
      // The Windows debug build overflows the default 2 MiB worker stack.
      RUST_MIN_STACK: '67108864',
      RUST_LOG: process.env.MW_SHOTS_RUST_LOG ?? 'warn',
    });
    await waitFor(async () => (await fetch(`http://127.0.0.1:${port}/healthz`)).ok, 'the server', 180_000);
    log(`stack ${kind} up at ${stack.base} (run dir ${dir})`);
    return stack;
  } catch (error) {
    stop();
    process.removeListener('exit', onExit);
    throw error;
  }
}

// ── direct use: keep a stack up for exploring ──────────────────────────────────
if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const kind = (process.argv[2] ?? 'M').toUpperCase();
  const seedArg = process.argv.indexOf('--seed');
  const seeds = seedArg > 0 ? process.argv[seedArg + 1].split(',') : ['mail-basic'];
  const { applySeeds, mailSeeder } = await import('./lib/seed.mjs');
  const stack = await startStack(kind, { beforeServer: mailSeeder(seeds) });
  const { chromium } = await import('./lib/browser.mjs').then((m) => m.playwright());
  const browser = await chromium.launch({ args: (await import('./lib/browser.mjs')).launchArgs() });
  await applySeeds(browser, stack, seeds);
  await browser.close();
  log(`ready: ${stack.base}  user ${stack.user.username} / ${stack.user.password}  console ${stack.base}${stack.adminPath} ${stack.admin.username} / ${stack.admin.password}`);
  log('Ctrl-C stops it');
  const quit = () => {
    stack.stop();
    rmSync(stack.dir, { recursive: true, force: true });
    process.exit(0);
  };
  process.on('SIGINT', quit);
  process.on('SIGTERM', quit);
  setInterval(() => {}, 1 << 30);
}
