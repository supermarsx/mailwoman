import {render,screen,waitFor,cleanup,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import type {JmapRequest} from '../api/jmap-types.ts';
const doubles=vi.hoisted(()=>({actions:{} as Record<string,()=>boolean>,jmap:vi.fn(),refreshFolders:vi.fn(async()=>{}),trashJob:vi.fn(()=>true)}));
vi.mock('../keymap/index.ts',()=>({useActions:(actions:Record<string,()=>boolean>)=>{doubles.actions=actions;}}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:doubles.jmap})}));
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=>null,mailboxes:()=>[{id:'folder',name:'Folder'},{id:'trash',name:'Trash',role:'trash'},{id:'archive',name:'Archive',role:'archive'}],refreshFolders:doubles.refreshFolders})}));
vi.mock('./emptyTrashJob.ts',()=>({startTrashJob:doubles.trashJob}));
import {FolderPane} from './FolderPane.tsx';
afterEach(()=>{cleanup();suspendAccountPreferences();vi.clearAllMocks();});
it('routes active shortcuts to its own folder and leaves inactive panes alone',async()=>{
 activateAccountPreferences('same','msa1_verified_parent');
 doubles.jmap.mockResolvedValue({sessionState:'s',methodResponses:[['Email/query',{accountId:'same',ids:['one','two'],position:0,total:2},'q'],['Email/get',{accountId:'same',list:[{id:'one',subject:'First',from:[],keywords:{}},{id:'two',subject:'Second',from:[],keywords:{$seen:true}}]},'g']]});
 const [active,setActive]=createSignal(true),open=vi.fn();
 render(()=><FolderPane mailboxId="folder" active={active()} onOpen={open}/>);
 await screen.findByText('First');
 expect(doubles.actions['next']?.()).toBe(true);expect(open).toHaveBeenLastCalledWith('one');
 expect(doubles.actions['next']?.()).toBe(true);expect(open).toHaveBeenLastCalledWith('two');
 expect(screen.getByText('First').closest('li')).toHaveAttribute('data-unread','true');
 expect(screen.getByText('Second').closest('li')).toHaveAttribute('data-unread','false');
 setActive(false);open.mockClear();expect(doubles.actions['next']?.()).toBe(false);expect(open).not.toHaveBeenCalled();
 setActive(true);expect(doubles.actions['refresh']?.()).toBe(true);await waitFor(()=>expect(doubles.jmap).toHaveBeenCalledTimes(2));
});
it('uses only this pane selection and refreshes rows and counts after a confirmed update',async()=>{
 activateAccountPreferences('same','msa1_verified_parent');let read=false;
 const rows=()=>[{id:'one',subject:'First',from:[],mailboxIds:{folder:true},keywords:read?{$seen:true}:{}},{id:'two',subject:'Second',from:[],mailboxIds:{folder:true},keywords:{}}];
 doubles.jmap.mockImplementation(async(request:JmapRequest)=>({sessionState:'s',methodResponses:request.methodCalls.map(([name,args,id])=>{
  if(name==='Email/query')return [name,{accountId:'same',ids:['one','two'],position:0,total:2},id];
  if(name==='Email/get')return [name,{accountId:'same',state:'before',list:(args as {ids?:string[]}).ids?rows().filter(row=>(args as {ids:string[]}).ids.includes(row.id)):rows()},id];
  if(name==='Email/set'){expect(args).toMatchObject({accountId:'same',ifInState:'before',update:{one:{'keywords/$seen':true}}});expect(Object.keys((args as {update:object}).update)).toEqual(['one']);read=true;return [name,{accountId:'same',updated:{one:null}},id];}
  throw new Error('Unexpected method');
 })}));
 render(()=><FolderPane mailboxId="folder" active onOpen={()=>{}}/>);await screen.findByText('First');fireEvent.click(screen.getAllByRole('checkbox')[0]!);
 fireEvent.click(await screen.findByRole('button',{name:'Mark read'}));
 await waitFor(()=>expect(doubles.refreshFolders).toHaveBeenCalledOnce());
 await waitFor(()=>expect(screen.getByText('First').closest('li')).toHaveAttribute('data-unread','false'));
 expect(screen.getByText('Second').closest('li')).toHaveAttribute('data-unread','true');
});
it('requires confirmation for Delete in Trash and starts only the captured selected IDs',async()=>{
 activateAccountPreferences('same','msa1_verified_parent');
 doubles.jmap.mockResolvedValue({sessionState:'s',methodResponses:[['Email/query',{accountId:'same',ids:['one','two'],position:0,total:2},'q'],['Email/get',{accountId:'same',list:[{id:'one',subject:'First',from:[],mailboxIds:{trash:true}},{id:'two',subject:'Second',from:[],mailboxIds:{trash:true}}]},'g']]});
 const reattach=vi.fn();render(()=><FolderPane mailboxId="trash" active onOpen={()=>{}} beforeGlobalAction={reattach}/>);
 await screen.findByText('First');fireEvent.click(screen.getAllByRole('checkbox')[0]!);
 expect(doubles.actions['trash']?.()).toBe(true);
 await screen.findByRole('alertdialog',{name:'Permanently delete selected messages'});expect(doubles.trashJob).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Cancel'}));expect(doubles.trashJob).not.toHaveBeenCalled();
 doubles.actions['trash']?.();fireEvent.click(await screen.findByRole('button',{name:'Delete permanently'}));
 expect(doubles.trashJob).toHaveBeenCalledWith(expect.anything(),{accountId:'same',mailboxId:'trash',ids:['one'],more:false},expect.objectContaining({current:expect.any(Function),refresh:expect.any(Function)}));
 expect(reattach).toHaveBeenCalledTimes(2);expect(screen.queryByRole('alertdialog')).toBeNull();
});
it('a reload keeps the focused row: rows are keyed by message id, not by object (t51)',async()=>{
 activateAccountPreferences('same','msa1_verified_parent');let read=false;
 doubles.jmap.mockImplementation(async()=>({sessionState:'s',methodResponses:[['Email/query',{accountId:'same',ids:['one','two'],position:0,total:2},'q'],['Email/get',{accountId:'same',list:[{id:'one',subject:'First',from:[],keywords:{}},{id:'two',subject:'Second',from:[],keywords:read?{$seen:true}:{}}]},'g']]}));
 render(()=><FolderPane mailboxId="folder" active onOpen={()=>{}}/>);
 const row=(await screen.findByText('Second')).closest('button')!;row.focus();
 read=true;expect(doubles.actions['refresh']?.()).toBe(true);
 await waitFor(()=>expect(screen.getByText('Second').closest('li')).toHaveAttribute('data-unread','false'));
 expect(screen.getByText('Second').closest('button')).toBe(row);
 expect(row.isConnected).toBe(true);expect(document.activeElement).toBe(row);
});
