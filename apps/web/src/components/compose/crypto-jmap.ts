// Compose-crypto JMAP glue (plan §2.2, e4). The `compose-crypto.tsx`
// subcomponents are transport-agnostic: they take a `KeyLookupFn` +
// `DlpScanFn` as props so they can be component-tested with plain mocks. This
// module supplies the REAL request builders + client-backed factories so e8's
// mount/wire step can drop them in without re-deriving the envelope:
//
//   <ComposeCrypto lookupKeys={createJmapKeyLookup(client, acct)}
//                  scanDlp={createJmapDlpScan(client, acct)} … />
//
// The method names / arg shapes mirror the frozen §2.2 families
// (`CryptoKey/lookup`, `Dlp/scan`) and the mock (`mw-mock-jmap`); the engine
// (e6) emits byte-identical shapes (the §1.5 parity gate).

import { CAP_CORE, type Id, type JmapRequest, type JmapResponse } from '../../api/jmap-types.ts';
import { responseFor } from '../../api/jmap.ts';
import { CAP_CRYPTO, CAP_SECURITY, type CryptoKey, type DlpVerdict } from '../../api/crypto-types.ts';
import type { Client } from '../../api/client.ts';

const CRYPTO_USING = [CAP_CORE, CAP_CRYPTO, CAP_SECURITY];

/** The `CryptoKey/lookup` key-discovery sources (frozen §2.2). */
export type KeyLookupSource = 'wkd' | 'vks' | 'autocrypt' | 'harvested';

/** All non-interactive sources — the default for compose capability probing. */
export const DEFAULT_LOOKUP_SOURCES: KeyLookupSource[] = ['harvested', 'autocrypt', 'wkd', 'vks'];

/** One attachment's metadata for a `Dlp/scan` dry-run (name/type/size only). */
export interface DlpAttachmentMeta {
  name: string;
  type: string;
  size: number;
}

/** The compose-time `Dlp/scan` draft payload (plan §2.2 — no `draftId` form here). */
export interface DlpScanDraft {
  recipients: string[];
  subject: string;
  bodyText: string;
  attachments: DlpAttachmentMeta[];
}

/** Resolve the public keys/certs known for one recipient address. */
export type KeyLookupFn = (address: string) => Promise<CryptoKey[]>;

/** Dry-run the outbound DLP rules against a draft (the compose-time heads-up). */
export type DlpScanFn = (draft: DlpScanDraft) => Promise<DlpVerdict[]>;

/** `CryptoKey/lookup {address, sources}` → `{list, notFound}` (§2.2). */
export function keyLookupRequest(
  accountId: Id,
  address: string,
  sources: KeyLookupSource[] = DEFAULT_LOOKUP_SOURCES,
): JmapRequest {
  return {
    using: CRYPTO_USING,
    methodCalls: [['CryptoKey/lookup', { accountId, address, sources }, 'l']],
  };
}

/** `Dlp/scan {…draft}` → `{list:[DlpVerdict]}` (§2.2 compose-time dry-run). */
export function dlpScanRequest(accountId: Id, draft: DlpScanDraft): JmapRequest {
  return {
    using: CRYPTO_USING,
    methodCalls: [['Dlp/scan', { accountId, ...draft }, 's']],
  };
}

interface KeyLookupResponse {
  list: CryptoKey[];
  notFound: string[];
}
interface DlpScanResponse {
  list: DlpVerdict[];
}

/** A client-backed `KeyLookupFn` for e8 to hand `ComposeCrypto` (real engine). */
export function createJmapKeyLookup(
  client: Pick<Client, 'jmap'>,
  accountId: Id,
  sources?: KeyLookupSource[],
): KeyLookupFn {
  return async (address: string): Promise<CryptoKey[]> => {
    const res: JmapResponse = await client.jmap(keyLookupRequest(accountId, address, sources));
    return responseFor<KeyLookupResponse>(res, 'l').list;
  };
}

/** A client-backed `DlpScanFn` for e8 to hand `ComposeCrypto` (real engine). */
export function createJmapDlpScan(client: Pick<Client, 'jmap'>, accountId: Id): DlpScanFn {
  return async (draft: DlpScanDraft): Promise<DlpVerdict[]> => {
    const res: JmapResponse = await client.jmap(dlpScanRequest(accountId, draft));
    return responseFor<DlpScanResponse>(res, 's').list;
  };
}
