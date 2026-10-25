import type { Client } from '../../api/client.ts';
import { responseFor } from '../../api/jmap.ts';
import { SieveRefused, type ScriptError } from './sieve-api.ts';

export interface Vacation {
  id: 'singleton'; isEnabled: boolean; fromDate: string | null; toDate: string | null;
  subject: string | null; textBody: string | null; htmlBody: string | null;
}
const USING = ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:vacationresponse'];
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
