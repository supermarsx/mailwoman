import {describe,it,expect,vi} from 'vitest';
import {revokeSessionBatch} from './sessionRevocation.ts';
const rows=[{id:'current',current:true},{id:'one',current:false},{id:'two',current:false}];
describe('reviewed session revocation',()=>{
 it('deduplicates and revokes the current session last',async()=>{const call=vi.fn(async(row:typeof rows[number])=>{void row;});const result=await revokeSessionBatch([...rows,rows[1]!],r=>r.id,r=>r.current,call);expect(call.mock.calls.map(call=>call[0])).toEqual([rows[1],rows[2],rows[0]]);expect(result.revoked).toEqual(['one','two','current']);expect(result.currentRevoked).toBe(true);});
 it('reports partial failure and retains the current session for recovery',async()=>{const call=vi.fn(async(row:typeof rows[number])=>{if(row.id==='two')throw{status:404};});const result=await revokeSessionBatch(rows,r=>r.id,r=>r.current,call);expect(result.revoked).toEqual(['one']);expect(result.failed).toEqual(['two']);expect(result.skipped).toEqual(['current']);expect(result.currentRevoked).toBe(false);});
 it.each([401,403,429,503])('stops without retry after %s',async(status)=>{const call=vi.fn(async()=>{throw{status};});const result=await revokeSessionBatch(rows,r=>r.id,r=>r.current,call);expect(call).toHaveBeenCalledOnce();expect(result.failed).toEqual(['one']);expect(result.skipped).toEqual(['two','current']);});
 it('does not claim a revocation when the server reports no matching session',async()=>{const result=await revokeSessionBatch([rows[1]!],r=>r.id,r=>r.current,async()=>false);expect(result.revoked).toEqual([]);expect(result.unchanged).toEqual(['one']);});
 it('stops after a successful response if its authentication generation changed',async()=>{let current=true;const call=vi.fn(async()=>{current=false;});const result=await revokeSessionBatch(rows,r=>r.id,r=>r.current,call,()=>current);expect(call).toHaveBeenCalledOnce();expect(result.revoked).toEqual(['one']);expect(result.skipped).toEqual(['two','current']);});
});
