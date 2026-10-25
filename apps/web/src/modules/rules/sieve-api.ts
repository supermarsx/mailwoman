import type { Client } from '../../api/client.ts';
import { responseFor, uploadBlob } from '../../api/jmap.ts';
import type { JmapSession } from '../../api/jmap-types.ts';
import { apiFetch } from '../../api/http.ts';

export const CAP_SIEVE = 'urn:ietf:params:jmap:sieve';
const USING = ['urn:ietf:params:jmap:core', CAP_SIEVE];
export interface ScriptRecord { id: string; name: string; blobId: string; isActive: boolean }
export interface ScriptError { type: string; description?: string; properties?: string[] }
export interface SieveCapability {
  maxSizeScriptName: number;
  maxSizeScript: number | null;
  maxNumberScripts: number | null;
  maxNumberRedirects: number | null;
  sieveExtensions: string[];
  'urn:mailwoman:policy'?: { rawEdit: boolean; forwardExternal: boolean; ownDomains: string[]; runsAt: 'engine' | 'server-sieve' };
}
export interface ScriptList { accountId: string; state: string; list: ScriptRecord[]; notFound: string[] }
interface SetResult {
  newState: string;
  created?: Record<string, { id: string }>;
  notCreated?: Record<string, ScriptError>;
  notUpdated?: Record<string, ScriptError>;
  notDestroyed?: Record<string, ScriptError>;
}
export class SieveRefused extends Error {
  constructor(readonly detail: ScriptError) { super(detail.description ?? detail.type); this.name = 'SieveRefused'; }
}
function checked(result: SetResult): SetResult {
  for (const errors of [result.notCreated, result.notUpdated, result.notDestroyed]) {
    const first = Object.values(errors ?? {})[0];
    if (first) throw new SieveRefused(first);
  }
  return result;
}
/** Every call carries the selected account. Save/activate use the last observed state. */
export function sieveApi(client: Client, accountId: string) {
  let session: JmapSession | undefined;
  let capability: SieveCapability | undefined;
  async function connect(): Promise<SieveCapability | null> {
    session = await client.session();
    const cap = session.accounts[accountId]?.accountCapabilities[CAP_SIEVE];
    if (!session.capabilities[CAP_SIEVE] || cap === undefined) return null;
    capability = cap as SieveCapability;
    return capability;
  }
  async function get(): Promise<ScriptList> {
    return responseFor<ScriptList>(await client.jmap({ using: USING, methodCalls: [['SieveScript/get', { accountId }, 'g']] }), 'g');
  }
  async function source(script: ScriptRecord): Promise<string> {
    if (!session) await connect();
    if (!session) throw new Error('Sieve session unavailable');
    const url = session.downloadUrl.replace('{accountId}', encodeURIComponent(accountId))
      .replace('{blobId}', encodeURIComponent(script.blobId)).replace('{name}', encodeURIComponent(`${script.name}.sieve`))
      .replace('{type}', encodeURIComponent('application/sieve'));
    const response = await apiFetch(url, { credentials: 'same-origin' });
    if (!response.ok) throw new SieveRefused({ type: 'downloadFailed', description: String(response.status) });
    return response.text();
  }
  async function upload(text: string): Promise<string> {
    if (!session) await connect();
    if (!session || !capability) throw new Error('Sieve session unavailable');
    const body = new Blob([text], { type: 'application/sieve' });
    if (capability.maxSizeScript !== null && body.size > capability.maxSizeScript) throw new SieveRefused({ type: 'tooLarge' });
    return (await uploadBlob(session.uploadUrl, accountId, body)).blobId;
  }
  async function set(args: Record<string, unknown>): Promise<SetResult> {
    return checked(responseFor<SetResult>(await client.jmap({ using: USING, methodCalls: [['SieveScript/set', { accountId, ...args }, 's']] }), 's'));
  }
  return {
    connect, get, source,
    async validate(text: string): Promise<ScriptError | null> {
      const blobId = await upload(text);
      return responseFor<{ error: ScriptError | null }>(await client.jmap({ using: USING, methodCalls: [['SieveScript/validate', { accountId, blobId }, 'v']] }), 'v').error;
    },
    async save(id: string | null, name: string, text: string, state: string, activate: boolean): Promise<void> {
      const blobId = await upload(text);
      await set({ ifInState: state, ...(id === null ? { create: { new: { name, blobId } } } : { update: { [id]: { name, blobId } } }),
        ...(activate ? { onSuccessActivateScript: id ?? '#new' } : {}) });
    },
    async activate(id: string | null, state: string): Promise<void> {
      await set({ ifInState: state, ...(id === null ? { onSuccessDeactivateScript: true } : { onSuccessActivateScript: id }) });
    },
    async remove(id: string, state: string): Promise<void> { await set({ ifInState: state, destroy: [id] }); },
  };
}
export type SieveApi = ReturnType<typeof sieveApi>;
