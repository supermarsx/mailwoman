import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {makeClient,MAILBOXES} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {CAP_MAIL,type JmapRequest,type JmapResponse} from '../api/jmap-types.ts';
import {FolderContextMenu} from './FolderContextMenu.tsx';
import {toasts,resetToasts} from './toast/toast.ts';

const server=vi.hoisted(()=>({ability:{moveToTrash:true,destroy:true} as unknown,total:3,calls:[] as Record<string,unknown>[]}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({
 session:async()=>({capabilities:{},accounts:{acct1:{name:'T',isPersonal:true,isReadOnly:false,accountCapabilities:{[CAP_MAIL]:{mailwomanEmptyMailbox:server.ability}}}},primaryAccounts:{},username:'me',apiUrl:'',downloadUrl:'',uploadUrl:'',eventSourceUrl:'',state:'s'}),
 jmap:vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
  const [name,args,id]=request.methodCalls[0]!;const call=args as Record<string,unknown>;
  if(name!=='Mailbox/empty')throw new Error(`unexpected ${name}`);
  server.calls.push(call);
  const mailboxId=call['mailboxId'] as string,outcome=mailboxId==='trash'?'destroy':'trash',limit=call['limit'] as number;
  if(call['outcome']!==outcome)return {methodResponses:[['error',{type:'stateMismatch',mailwomanOutcome:outcome},id]],sessionState:'s'};
  if(call['expectedTotal']!==server.total)return {methodResponses:[['error',{type:'stateMismatch',mailwomanTotal:server.total},id]],sessionState:'s'};
  const processed=Math.min(limit,server.total);const total=server.total;server.total-=processed;
  return {methodResponses:[['Mailbox/empty',{accountId:'acct1',mailboxId,outcome,trashId:outcome==='trash'?'trash':null,total,processed,remaining:server.total},id]],sessionState:'s'};
 }),
})}));
afterEach(()=>{cleanup();resetToasts();localStorage.clear();server.ability={moveToTrash:true,destroy:true};server.total=3;server.calls=[];});

async function menuFor(index:number){
 const app=createAppState(makeClient());const box=MAILBOXES[index]!;
 render(()=><AppContext.Provider value={app}><FolderContextMenu box={box} onOpen={()=>{}} onSettings={()=>{}} onShare={()=>{}} canShare={()=>false} onProbe={()=>{}}><button>{`${box.name} target`}</button></FolderContextMenu></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});await waitFor(()=>expect(app.mailboxes()).toHaveLength(3));
 return box;
}
const openMenu=(name:string)=>fireEvent.keyDown(screen.getByText(`${name} target`),{key:'F10',shiftKey:true});

it('offers Empty folder… for an ordinary folder and names it, its count and the move to Trash',async()=>{
 await menuFor(1);
 // The ability comes from the session; the menu waits for it.
 await waitFor(()=>{openMenu('Archive');expect(screen.getByRole('menuitem',{name:'Empty folder…'})).toBeInTheDocument();});
 fireEvent.click(screen.getByRole('menuitem',{name:'Empty folder…'}));
 const dialog=await screen.findByRole('alertdialog');
 expect(dialog).toHaveTextContent(/Empty .*Archive.*\?/);
 expect(dialog).toHaveTextContent(/All 3 messages in .*Archive.* will be moved to Trash/);
 expect(dialog).toHaveTextContent('Subfolders and the messages in them are not touched.');
 expect(screen.getByRole('button',{name:'Move to Trash'})).toBeEnabled();
 // The preview asked the server and changed nothing; Cancel runs nothing.
 expect(server.calls.every(call=>call['limit']===0)).toBe(true);
 fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
 expect(screen.queryByRole('alertdialog')).toBeNull();
 expect(server.calls.every(call=>call['limit']===0)).toBe(true);expect(server.total).toBe(3);
});

it('says plainly that emptying Trash is permanent and makes a large one type the name first',async()=>{
 server.total=150;await menuFor(2);
 await waitFor(()=>{openMenu('Trash');expect(screen.getByRole('menuitem',{name:'Empty folder…'})).toBeInTheDocument();});
 fireEvent.click(screen.getByRole('menuitem',{name:'Empty folder…'}));
 const dialog=await screen.findByRole('alertdialog');
 expect(dialog).toHaveTextContent('All 150 messages in Trash will be deleted permanently. This cannot be undone.');
 const confirm=screen.getByRole('button',{name:'Delete permanently'});expect(confirm).toBeDisabled();
 fireEvent.input(screen.getByRole('textbox'),{target:{value:'Tras'}});expect(confirm).toBeDisabled();
 fireEvent.input(screen.getByRole('textbox'),{target:{value:'Trash'}});expect(confirm).toBeEnabled();
 fireEvent.click(confirm);
 await waitFor(()=>expect(screen.queryByRole('alertdialog')).toBeNull());
 await waitFor(()=>expect(toasts().find(item=>item.testId==='empty-folder-result')?.message).toContain('Deleted 150 messages from'));
 // Two bounded batches after the preview, each with the count the last one left.
 expect(server.calls.filter(call=>call['limit']!==0).map(call=>[call['expectedTotal'],call['limit'],call['outcome']])).toEqual([[150,100,'destroy'],[50,57,'destroy']]);
});

it('hides Empty folder… where the backend cannot do it (POP3) and for a folder with nothing in it says so',async()=>{
 server.ability={moveToTrash:false,destroy:false};await menuFor(1);
 openMenu('Archive');await waitFor(()=>expect(screen.getByRole('menuitem',{name:'Pin'})).toBeInTheDocument());
 await new Promise(resolve=>setTimeout(resolve,20));
 fireEvent.keyDown(document.activeElement!,{key:'Escape'});openMenu('Archive');
 expect(screen.queryByRole('menuitem',{name:'Empty folder…'})).toBeNull();
});

it('does not open a confirmation for an empty folder',async()=>{
 server.total=0;await menuFor(1);
 await waitFor(()=>{openMenu('Archive');expect(screen.getByRole('menuitem',{name:'Empty folder…'})).toBeInTheDocument();});
 fireEvent.click(screen.getByRole('menuitem',{name:'Empty folder…'}));
 await waitFor(()=>expect(toasts().some(item=>/already empty/.test(item.message))).toBe(true));
 expect(screen.queryByRole('alertdialog')).toBeNull();
});
