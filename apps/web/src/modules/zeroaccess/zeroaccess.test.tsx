import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { ZeroAccessSettings } from './ZeroAccessSettings.tsx';
import { DevicePairing } from './DevicePairing.tsx';
import { ZeroAccessService } from './service.ts';
import { sasMatches } from './pairing.ts';
import { encodeQr } from './qr.ts';
import { utf8ToB64, b64ToUtf8, type ZeroAccessCrypto, type ZaSealRowIn, type ZaOpenRowIn } from './crypto.ts';
import {
  ZA_ACTIVE_SERVER_CAVEAT,
  ZA_SERVER_STILL_SEES,
  ZA_NO_SEARCH_CLAIM,
} from './disclosure.ts';

// ── A mock ZeroAccessCrypto that HONOURS the frozen row AAD ───────────────────────
// Frozen (state.md / e6): AAD = table ‖ 0x1F ‖ row_id ‖ 0x1F ‖ ascii-decimal(schema).
// seal binds that exact AAD into the blob; open recomputes it from its own inputs and
// refuses to decrypt on any mismatch — the same location-binding the wasm enforces.
function frozenAad(table: string, rowId: string, schemaVersion: number): string {
  return `${table}${rowId}${schemaVersion}`;
}

function mockZa(): ZeroAccessCrypto {
  const keys = new Map<string, string>();
  let seq = 0;
  const put = (material: string): string => {
    const ref = `k${(seq += 1)}`;
    keys.set(ref, material);
    return ref;
  };
  return {
    deriveRootKey: async (i) => ({ keyRef: put(`root:${i.secretB64}:${i.saltB64}`) }),
    deriveKek: async (i) => ({ keyRef: put(`kek:${keys.get(i.keyRef) ?? ''}`) }),
    deriveSubkey: async (i) => ({ keyRef: put(`sub:${i.label}:${keys.get(i.keyRef) ?? ''}`) }),
    generateDataKey: async () => ({ keyRef: put(`data:${Math.random()}`) }),
    wrapKey: async (i) => ({ blobB64: btoa(`wrap:${keys.get(i.dataKeyRef) ?? ''}`) }),
    unwrapKey: async (i) => ({ keyRef: put(atob(i.blobB64).replace(/^wrap:/, '')) }),
    sealRow: async (i: ZaSealRowIn) => {
      const aad = frozenAad(i.table, i.rowId, i.schemaVersion);
      return { ciphertextB64: btoa(JSON.stringify({ aad, key: keys.get(i.keyRef) ?? '', pt: i.plaintextB64 })) };
    },
    openRow: async (i: ZaOpenRowIn) => {
      const parsed = JSON.parse(atob(i.ciphertextB64)) as { aad: string; key: string; pt: string };
      const wantAad = frozenAad(i.table, i.rowId, i.schemaVersion);
      if (parsed.aad !== wantAad) throw new Error('AAD mismatch (row moved/replayed)');
      if (parsed.key !== (keys.get(i.keyRef) ?? '')) throw new Error('wrong key');
      return { plaintextB64: parsed.pt };
    },
    recoveryPhrase: async () => ({ phrase: 'alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima' }),
    restoreFromPhrase: async () => ({ keyRef: put('restored-root') }),
    pairGenerate: async () => ({ publicB64: 'PEER_PUBLIC_B64', secretRef: put('pair-secret') }),
    pairSeal: async () => ({ sasWords: ['able', 'acid', 'aged', 'also', 'area', 'army'], envelopeB64: 'ENVELOPE_B64' }),
    pairComplete: async () => ({ sasWords: ['able', 'acid', 'aged', 'also', 'area', 'army'], keyRef: put('paired-root') }),
    lock: async () => undefined,
    lockAll: async () => undefined,
  };
}

function okJson(body: unknown): Response {
  return new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
}

describe('zero-access disclosure (honesty gate)', () => {
  it('shows the malicious-active-server caveat, server-visible metadata, and no-search claim', () => {
    render(() => (
      <ZeroAccessSettings za={mockZa()} initialStatus={{ enabled: false, pairedDevices: [] }} />
    ));
    expect(screen.getByTestId('active-server-caveat').textContent).toBe(ZA_ACTIVE_SERVER_CAVEAT);
    expect(screen.getByTestId('no-search-claim').textContent).toBe(ZA_NO_SEARCH_CLAIM);
    // Every server-visible metadata item is listed verbatim.
    for (const item of ZA_SERVER_STILL_SEES) {
      expect(screen.getByText(item)).toBeInTheDocument();
    }
  });

  it('never uses hype wording', () => {
    expect(ZA_ACTIVE_SERVER_CAVEAT.toLowerCase()).not.toContain('ultra');
    expect(ZA_ACTIVE_SERVER_CAVEAT.toLowerCase()).not.toContain('unbreakable');
    expect(ZA_ACTIVE_SERVER_CAVEAT).toContain('does NOT defend');
  });
});

describe('zero-access seal→store→open round-trip (frozen AAD)', () => {
  it('unwraps a data key on unlock and round-trips a row bound to its location', async () => {
    const za = mockZa();
    const account = {
      enabled: true,
      saltB64: 'c2FsdHNhbHRzYWx0c2FsdA==',
      kdfParams: { mCost: 19456, tCost: 2, pCost: 1 },
      wrappedDataKeyB64: btoa('wrap:data:fixed'),
      pairedDevices: [],
    };
    const service = new ZeroAccessService(za);
    const session = await service.unlock(utf8ToB64('correct horse battery staple'), account);

    const ct = await service.sealRow(session, 'message-cache', 'hello body', 'messages', 'row-1', 7);
    // Server stores ciphertext only; the client reads it back and decrypts.
    const pt = await service.openRow(session, 'message-cache', ct, 'messages', 'row-1', 7);
    expect(pt).toBe('hello body');

    // A moved/replayed row (different row_id) fails the AAD check.
    await expect(service.openRow(session, 'message-cache', ct, 'messages', 'row-2', 7)).rejects.toThrow(/AAD mismatch/);
    // A different schema version also fails.
    await expect(service.openRow(session, 'message-cache', ct, 'messages', 'row-1', 8)).rejects.toThrow(/AAD mismatch/);
  });

  it('utf8/base64 helpers round-trip', () => {
    expect(b64ToUtf8(utf8ToB64('héllo 🌱'))).toBe('héllo 🌱');
  });
});

describe('zero-access enable flow', () => {
  it('derives keys, POSTs only wrapped material, and shows the recovery phrase once', async () => {
    const za = mockZa();
    const fetcher = vi.fn(async (input: string, init?: RequestInit) => {
      if (input === '/api/zeroaccess/enable') {
        const body = JSON.parse((init?.body as string) ?? '{}') as Record<string, unknown>;
        // No raw key is ever sent — only wrapped material + kdf params.
        expect(body).toHaveProperty('wrappedDataKeyB64');
        expect(body).toHaveProperty('kdfParams');
        expect(JSON.stringify(body)).not.toContain('root:');
        return okJson({ ok: true });
      }
      return okJson({ enabled: false, pairedDevices: [] });
    });

    render(() => (
      <ZeroAccessSettings za={za} fetcher={fetcher} initialStatus={{ enabled: false, pairedDevices: [] }} />
    ));
    fireEvent.input(screen.getByLabelText('Zero-access passphrase'), { target: { value: 'a-good-passphrase' } });
    fireEvent.click(screen.getByRole('button', { name: 'Enable zero-access' }));

    await waitFor(() => expect(screen.getByTestId('recovery-phrase')).toBeInTheDocument());
    expect(screen.getByTestId('recovery-phrase').textContent).toContain('alpha bravo charlie');
    expect(fetcher).toHaveBeenCalledWith('/api/zeroaccess/enable', expect.anything());
  });
});

describe('device pairing SAS', () => {
  it('sasMatches is exact', () => {
    expect(sasMatches(['a', 'b'], ['a', 'b'])).toBe(true);
    expect(sasMatches(['a', 'b'], ['a', 'c'])).toBe(false);
    expect(sasMatches(['a'], ['a', 'b'])).toBe(false);
  });

  it('new device recovers a matching SAS to compare', async () => {
    const fetcher = vi.fn(async () => okJson({ pairingId: 'pair-1' }));
    render(() => <DevicePairing za={mockZa()} fetcher={fetcher} />);
    fireEvent.click(screen.getByRole('button', { name: 'Show pairing QR' }));
    await waitFor(() => expect(screen.getByTestId('pairing-qr')).toBeInTheDocument());
    fireEvent.input(screen.getByLabelText('Sealed envelope'), { target: { value: 'ENVELOPE_B64' } });
    fireEvent.click(screen.getByRole('button', { name: 'Complete pairing' }));
    await waitFor(() => expect(screen.getByTestId('sas-block')).toBeInTheDocument());
    // The six SAS words are shown for out-of-band comparison.
    expect(screen.getByText('able')).toBeInTheDocument();
    expect(screen.getByText('army')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'The words match' }));
    expect(screen.getByTestId('pairing-confirmed')).toBeInTheDocument();
  });

  it('existing device seals the root to a scanned public and can relay the envelope', async () => {
    render(() => <DevicePairing za={mockZa()} rootRef="root-ref" />);
    fireEvent.input(screen.getByLabelText('Pairing code'), { target: { value: 'PEER_PUBLIC_B64' } });
    fireEvent.click(screen.getByRole('button', { name: 'Seal my keys for that device' }));
    await waitFor(() => expect(screen.getByTestId('envelope-out')).toBeInTheDocument());
    expect((screen.getByTestId('envelope-out') as HTMLInputElement).value).toBe('ENVELOPE_B64');
  });
});

describe('QR encoder', () => {
  it('produces a square module matrix that encodes the pairing public point', () => {
    const m = encodeQr('PEER_PUBLIC_B64_AAAAAAAAAAAAAAAAAAAAAAAA', 'M');
    expect(m.length).toBeGreaterThanOrEqual(21);
    expect(m.every((row) => row.length === m.length)).toBe(true);
    // Finder pattern top-left corner is dark.
    expect(m[0]![0]).toBe(true);
    // At least some modules are set.
    expect(m.some((row) => row.some((c) => c))).toBe(true);
  });
});
