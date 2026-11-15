export * from './pushClientCore.ts';
import {createPushClient as create,type PushClientOptions,type PushClientImpl,type SessionCheck} from './pushClientCore.ts';
import {apiFetch} from '../api/http.ts';
import {withBase} from '../api/basePath.ts';
export function createPushClient(opts:PushClientOptions={}):PushClientImpl{return create({...opts,checkSession:opts.checkSession??(()=>defaultCheckSession(opts.bearer))});}
async function defaultCheckSession(bearer?: string): Promise<SessionCheck> {
  if (typeof fetch !== 'function') return 'unknown';
  try {
    const headers: Record<string, string> = {};
    if (bearer !== undefined && bearer.length > 0) headers['Authorization'] = `Bearer ${bearer}`;
    const res = await apiFetch(withBase('/jmap/session'), { credentials: 'include', headers });
    return res.status === 401 || res.status === 403 ? 'unauthorized' : 'ok';
  } catch {
    return 'unknown';
  }
}
