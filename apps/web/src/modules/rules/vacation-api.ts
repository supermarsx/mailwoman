import type { Client } from '../../api/client.ts';
import { responseFor } from '../../api/jmap.ts';
import type { JmapSession } from '../../api/jmap-types.ts';
import { CAP_SIEVE, SieveRefused, type ScriptError, type SieveCapability } from './sieve-api.ts';

export interface Vacation {
  id: 'singleton'; isEnabled: boolean; fromDate: string | null; toDate: string | null;
  subject: string | null; textBody: string | null; htmlBody: string | null;
}
export const CAP_VACATION = 'urn:ietf:params:jmap:vacationresponse';
const USING = ['urn:ietf:params:jmap:core', CAP_VACATION];

/**
 * Whether a saved vacation reply will actually be sent, and by whom (t39-e7, L11).
 *
 * - `server-sieve`: Mailwoman installs it in the Sieve script on the account's
 *   ManageSieve server, which sends it at delivery.
 * - `engine`: no ManageSieve; Mailwoman's own Sieve run sends it when it syncs
 *   new inbox mail.
 * - `server`: the upstream JMAP server answers `VacationResponse` itself (proxy
 *   mode).
 * - unavailable: the account has no `vacationresponse` capability, or its Sieve
 *   capability does not offer the `vacation` extension (the server lacks it or
 *   the administrator's `sieve.vacationAllowed` is off). Nothing could send the
 *   reply, so the form is not offered.
 */
export type VacationSupport = { available: true; runsAt: 'server-sieve' | 'engine' | 'server' } | { available: false };

export function vacationSupport(session: JmapSession, accountId: string): VacationSupport {
  const caps = session.accounts[accountId]?.accountCapabilities ?? {};
  if (!session.capabilities[CAP_VACATION] || caps[CAP_VACATION] === undefined) return { available: false };
  const sieve = caps[CAP_SIEVE] as SieveCapability | undefined;
  if (sieve === undefined) return { available: true, runsAt: 'server' };
  if (!sieve.sieveExtensions?.includes('vacation')) return { available: false };
  return { available: true, runsAt: sieve['urn:mailwoman:policy']?.runsAt ?? 'server' };
}

export function vacationApi(client: Client, accountId: string) {
  return {
    async get(): Promise<{ state: string; value: Vacation }> {
      const response = responseFor<{ state: string; list: Vacation[] }>(await client.jmap({ using: USING, methodCalls: [['VacationResponse/get', { accountId }, 'g']] }), 'g');
      const value = response.list[0]; if (!value) throw new Error('Vacation response unavailable');
      return { state: response.state, value };
    },
    async save(value: Vacation, state: string): Promise<void> {
      const { id: _id, ...patch } = value;
      const result = responseFor<{ notUpdated?: Record<string, ScriptError> }>(await client.jmap({ using: USING,
        methodCalls: [['VacationResponse/set', { accountId, ifInState: state, update: { singleton: patch } }, 's']] }), 's');
      if (result.notUpdated?.singleton) throw new SieveRefused(result.notUpdated.singleton);
    },
  };
}
export type VacationApi = ReturnType<typeof vacationApi>;
