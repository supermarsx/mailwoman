import {expect,it,vi} from 'vitest';
import {createRoot} from 'solid-js';
import {makeClient,mkEmail} from '../../components/appHarness.tsx';
import {createMailSlice} from './mail.ts';

it('moves verified off-list sender messages and Undo restores each original folder',async()=>{
  const client=makeClient({emails:[mkEmail('visible')]});
  await createRoot(async dispose=>{
    try{
      const mail=createMailSlice({client,showToast:vi.fn()});await mail.login({username:'me@example.org',password:'p'});
      const offList=mkEmail('unloaded',{mailboxIds:{archive:true}});
      await mail.trashMessage(['unloaded'],[offList]);
      const sets=()=>vi.mocked(client.jmap).mock.calls.flatMap(([request])=>request.methodCalls.filter(call=>call[0]==='Email/set').map(call=>call[1]['update']));
      expect(sets().at(-1)).toEqual({unloaded:{mailboxIds:{trash:true}}});
      expect(mail.messages().some(email=>email.id==='unloaded')).toBe(false);
      await mail.undoNow();
      expect(sets().at(-1)).toEqual({unloaded:{mailboxIds:{archive:true}}});
      expect(mail.messages().some(email=>email.id==='unloaded')).toBe(false);
    }finally{dispose();}
  });
});
