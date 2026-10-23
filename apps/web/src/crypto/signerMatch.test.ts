import { describe, expect, it } from 'vitest';
import type { CryptoKey } from '../api/crypto-types.ts';
import { pickSignerArmor } from './signerMatch.ts';

const NOW = new Date('2026-10-23T12:00:00Z');

function key(over: Partial<CryptoKey>): CryptoKey {
  return {
    id: 'k1',
    kind: 'pgp',
    isOwn: false,
    addresses: ['alice@bank.example'],
    fingerprint: 'FP',
    keyId: 'KID',
    algorithm: 'ed25519',
    createdAt: '2026-01-01T00:00:00Z',
    expiresAt: null,
    publicKeyArmored: 'ARMOR',
    certPem: null,
    trust: 'tofu',
    autocrypt: false,
    source: 'harvested',
    hasPrivate: false,
    encryptedPrivateBackup: null,
    verifiedAt: null,
    keyHistory: [],
    ...over,
  };
}

describe('pickSignerArmor', () => {
  it('accepts a valid key for the exact address', () => {
    expect(pickSignerArmor('alice@bank.example', [key({})], NOW)).toBe('ARMOR');
  });

  it('refuses a key that shares only the local part', () => {
    const evil = key({ addresses: ['alice@evil.example'], publicKeyArmored: 'EVIL' });
    expect(pickSignerArmor('alice@bank.example', [evil], NOW)).toBeUndefined();
  });

  it('refuses a key for a different domain', () => {
    const other = key({ addresses: ['alice@bank.example.net'] });
    expect(pickSignerArmor('alice@bank.example', [other], NOW)).toBeUndefined();
  });

  it('refuses any key for a From that has no domain', () => {
    expect(pickSignerArmor('alice', [key({})], NOW)).toBeUndefined();
    expect(pickSignerArmor('alice', [key({ addresses: ['alice'] })], NOW)).toBeUndefined();
  });

  it('accepts case differences in the local part and the domain', () => {
    expect(pickSignerArmor('Alice@Bank.EXAMPLE', [key({})], NOW)).toBe('ARMOR');
    expect(pickSignerArmor('alice@bank.example', [key({ addresses: ['ALICE@bank.Example'] })], NOW)).toBe(
      'ARMOR',
    );
  });

  it('accepts an IDN domain and its punycode form as the same address', () => {
    const k = key({ addresses: ['alice@xn--bcher-kva.example'] });
    expect(pickSignerArmor('alice@bücher.example', [k], NOW)).toBe('ARMOR');
    expect(pickSignerArmor('alice@xn--bcher-kva.example', [key({ addresses: ['alice@BÜCHER.example'] })], NOW)).toBe(
      'ARMOR',
    );
  });

  it('refuses a revoked key', () => {
    expect(pickSignerArmor('alice@bank.example', [key({ trust: 'revoked' })], NOW)).toBeUndefined();
  });

  it('refuses a key that has expired at verify time', () => {
    const expired = key({ expiresAt: '2026-10-01T00:00:00Z' });
    expect(pickSignerArmor('alice@bank.example', [expired], NOW)).toBeUndefined();
    const live = key({ expiresAt: '2027-01-01T00:00:00Z' });
    expect(pickSignerArmor('alice@bank.example', [live], NOW)).toBe('ARMOR');
  });

  it('skips S/MIME and armor-less keys and picks the first usable exact key', () => {
    const keys = [
      key({ kind: 'smime', publicKeyArmored: null, certPem: 'PEM' }),
      key({ publicKeyArmored: null }),
      key({ addresses: ['alice@evil.example'], publicKeyArmored: 'EVIL' }),
      key({ trust: 'revoked', publicKeyArmored: 'REVOKED' }),
      key({ publicKeyArmored: 'GOOD' }),
    ];
    expect(pickSignerArmor('alice@bank.example', keys, NOW)).toBe('GOOD');
  });

  it('returns undefined for an empty sender', () => {
    expect(pickSignerArmor('', [key({})], NOW)).toBeUndefined();
  });
});
