import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import Encryption, { type EncryptionApi } from './index.tsx';
import { counts, evidenceArgs, keyAgeDays, needsAttention } from './model.ts';
import type { EncryptionSnapshot } from '../../../api/admin/encryption.ts';
import { renderInShell } from '../shell/testkit.tsx';

const snapshot = (over: Partial<EncryptionSnapshot> = {}): EncryptionSnapshot => ({
  mode: 'required',
  backend: 'sqlite',
  databaseFile: 'mailwoman.db',
  key: { configured: true, source: { kind: 'file', detail: '/run/secrets/db-key', age_secs: 86400 * 40 }, fingerprint: 'abcd1234abcd1234' },
  rotation: { days: 365, lastRotatedAt: '2026-01-01T00:00:00Z', encryptedAt: '2026-01-01T00:00:00Z', dueAt: '2027-01-01T00:00:00Z', overdue: false },
  stores: [
    { id: 'database', state: 'encrypted', evidence: 'evidence-db-header', args: { cipher: 'SQLCipher 4', includesWal: true } },
    { id: 'searchIndex', state: 'plaintext', evidence: 'evidence-search-disk', args: { files: 12 } },
    { id: 'logs', state: 'not-stored', evidence: 'evidence-logs-stdout', args: {} },
    { id: 'secrets', state: 'plaintext', evidence: 'evidence-secrets', args: { serverKey: 'file', dbKeyBesideDatabase: false } },
  ],
  postgres: null,
  columnCoverage: [
    { table: 'sessions', column: 'sealed_creds', what: 'mailbox credentials', sealed: true },
    { table: 'settings', column: 'value', what: 'configuration values', sealed: false },
  ],
  warnings: ['searchIndexPlaintext'],
  keys: ['encryption.rotationDays'],
  commands: {
    dryRun: 'mailwoman db encrypt --dry-run', encrypt: 'mailwoman db encrypt', rekey: 'mailwoman db rekey',
    decrypt: 'mailwoman db decrypt', verify: 'mailwoman db verify', backup: 'mailwoman db backup --to FILE', keygen: 'mailwoman db keygen',
  },
  ...over,
});

const api = (over: Partial<EncryptionApi> = {}): EncryptionApi => ({
  load: vi.fn(async () => snapshot()),
  verify: vi.fn(async () => ({
    passed: true,
    checks: [
      { name: 'header-not-sqlite', passed: true, detail: 'ok' },
      { name: 'wrong-key-rejected', passed: true, detail: 'refused' },
    ],
  })),
  rehearse: vi.fn(async () => ({ ok: true, operation: 'rekey', tables: 100, rows: 2500, bytes: 4096, millis: 420 })),
  settings: vi.fn(async () => []),
  ...over,
});

describe('Admin encryption', () => {
  it('lists each store with its state and how that was determined', async () => {
    renderInShell(() => <Encryption api={api()} />);
    const table = await screen.findByTestId('encryption-stores');
    expect(within(table).getByTestId('encryption-row-database')).toHaveAttribute('data-state', 'encrypted');
    expect(within(table).getByTestId('encryption-row-searchIndex')).toHaveAttribute('data-state', 'plaintext');
    expect(within(table).getByTestId('encryption-row-searchIndex')).toHaveTextContent('12');
    expect(within(table).getByTestId('encryption-row-logs')).toHaveAttribute('data-state', 'not-stored');
  });

  it('shows findings and the sealed-column coverage', async () => {
    renderInShell(() => <Encryption api={api()} />);
    expect(await screen.findByTestId('encryption-warnings')).toBeInTheDocument();
    const cols = screen.getByTestId('encryption-columns');
    expect(within(cols).getByText('sealed_creds')).toBeInTheDocument();
    expect(within(cols).getByText('value')).toBeInTheDocument();
  });

  it('runs the verification and reports each check', async () => {
    const a = api();
    renderInShell(() => <Encryption api={a} />);
    fireEvent.click(await screen.findByTestId('encryption-verify'));
    const result = await screen.findByTestId('encryption-verify-result');
    expect(a.verify).toHaveBeenCalledTimes(1);
    expect(within(result).getAllByRole('listitem')).toHaveLength(2);
  });

  it('reports a failed verification', async () => {
    const a = api({ verify: vi.fn(async () => ({ passed: false, checks: [{ name: 'header-not-sqlite', passed: false, detail: 'readable' }] })) });
    renderInShell(() => <Encryption api={a} />);
    fireEvent.click(await screen.findByTestId('encryption-verify'));
    const result = await screen.findByTestId('encryption-verify-result');
    expect(result.querySelector('[data-passed="false"]')).not.toBeNull();
  });

  it('runs the migration dry run and shows its totals', async () => {
    const a = api();
    renderInShell(() => <Encryption api={a} />);
    fireEvent.click(await screen.findByTestId('encryption-rehearse'));
    await waitFor(() => expect(screen.getByTestId('encryption-rehearse-result')).toBeInTheDocument());
    expect(screen.getByTestId('encryption-rehearse-result')).toHaveTextContent('2,500');
  });

  it('disables the file checks on Postgres and shows what the connection reveals', async () => {
    const pg = snapshot({
      backend: 'postgres',
      stores: [{ id: 'database', state: 'sealed-columns-only', evidence: 'evidence-db-postgres', args: {} }],
      postgres: { connectionSsl: false, serverSsl: 'on', dataChecksums: 'off', pgcryptoInstalled: false, pgcryptoAvailable: true, tdeIndicators: { settings: [], extensions: [] } },
      warnings: ['postgresNoTls'],
    });
    renderInShell(() => <Encryption api={api({ load: async () => pg })} />);
    expect(await screen.findByTestId('encryption-postgres')).toBeInTheDocument();
    expect(screen.getByTestId('encryption-verify')).toBeDisabled();
    expect(screen.getByTestId('encryption-row-database')).toHaveAttribute('data-state', 'sealed-columns-only');
  });

  it('flags an overdue rotation', async () => {
    const s = snapshot({ rotation: { days: 30, lastRotatedAt: '2025-01-01T00:00:00Z', encryptedAt: null, dueAt: '2025-01-31T00:00:00Z', overdue: true } });
    renderInShell(() => <Encryption api={api({ load: async () => s })} />);
    expect(await screen.findByTestId('encryption-overdue')).toBeInTheDocument();
  });
});

describe('encryption model', () => {
  it('counts states and flags the ones that need a look', () => {
    const c = counts(snapshot().stores);
    expect(c.encrypted).toBe(1);
    expect(c.plaintext).toBe(2);
    expect(needsAttention('plaintext')).toBe(true);
    expect(needsAttention('encrypted')).toBe(false);
  });

  it('turns evidence arguments into message values', () => {
    expect(evidenceArgs({ id: 'x', state: 'encrypted', evidence: 'e', args: { a: true, b: 3, c: 'x', d: [1, 2], e: null } }))
      .toEqual({ a: 'yes', b: 3, c: 'x', d: 2, e: '-' });
  });

  it('reads key age from the key file, else from the last rotation', () => {
    expect(keyAgeDays(snapshot())).toBe(40);
    const s = snapshot({ key: { configured: true, source: { kind: 'env', detail: 'MW_DB_KEY', age_secs: null }, fingerprint: null } });
    expect(keyAgeDays(s, Date.parse('2026-01-11T00:00:00Z'))).toBe(10);
    expect(keyAgeDays(snapshot({ key: { configured: false, source: null, fingerprint: null }, rotation: { days: 0, lastRotatedAt: null, encryptedAt: null, dueAt: null, overdue: false } }))).toBeNull();
  });
});
