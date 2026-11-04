import {describe,expect,it,vi} from 'vitest';
import type {JmapRequest,JmapResponse} from './jmap-types.ts';
import {BATCH,lastActivity,listFolderIds,markFolderRead,markUnread,transferFolder,undoTransfer} from './folderBulk.ts';
import {canDeleteTree,canMoveFolder,deleteFolderTree,moveFolderTo} from './folderStructure.ts';
import type {Mailbox} from './jmap-types.ts';

const rights={mayRename:true,mayDelete:true,mayCreateChild:true};
const box=(id:string,parentId:string|null,extra:Partial<Mailbox>={}):Mailbox=>({id,name:id,parentId,role:null,sortOrder:0,totalEmails:0,unreadEmails:0,myRights:rights,...extra});
const auth={current:()=>true};
/** A fake server: `ids` are the messages of one folder; Email/set marks updated (or refuses the ids in `refuse`). */
function server(count:number,refuse:Set<string>=new Set()){
 const ids=Array.from({length:count},(_,i)=>`m${i}`);const sets:Record<string,unknown>[]=[];const queries:Record<string,unknown>[]=[];
 const jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
  const [name,args,id]=request.methodCalls[0]!;const call=args as Record<string,unknown>;
  if(name==='Email/query'){queries.push(call);const position=(call['position'] as number|undefined)??0,limit=call['limit'] as number;return {methodResponses:[['Email/query',{accountId:'a',ids:ids.slice(position,position+limit)},id]],sessionState:'s'};}
  if(name==='Email/set'){sets.push(call);const update=call['update'] as Record<string,unknown>;const updated:Record<string,null>={},notUpdated:Record<string,{type:string}>={};for(const key of Object.keys(update)){if(refuse.has(key))notUpdated[key]={type:'forbidden'};else updated[key]=null;}return {methodResponses:[['Email/set',{accountId:'a',updated,notUpdated},id]],sessionState:'s'};}
  throw new Error(name);
 });
 return {jmap,sets,queries,ids};
}

describe('whole-folder batches',()=>{
 it('pages through a folder in bounded queries',async()=>{
  const s=server(250);
  const ids=await listFolderIds(s,'a','box',auth);
  expect(ids).toHaveLength(250);expect(s.queries.map(q=>[q['position'],q['limit']])).toEqual([[0,BATCH],[100,BATCH],[200,BATCH]]);
 });
 it('marks everything unread as read in batches of at most 100, then reports the counts',async()=>{
  const s=server(250);const progress=vi.fn();
  const result=await markFolderRead(s,'a',['box'],auth,progress);
  expect(result).toMatchObject({processed:250,refused:0,total:250,stopped:false});
  expect(s.sets.map(set=>Object.keys(set['update'] as object).length)).toEqual([100,100,50]);
  expect((s.sets[0]!['update'] as Record<string,unknown>)['m0']).toEqual({'keywords/$seen':true});
  expect(s.queries[0]!['filter']).toEqual({inMailbox:'box',notKeyword:'$seen'});
  expect(progress).toHaveBeenLastCalledWith(250,250);
 });
 it('counts refusals instead of hiding them',async()=>{
  const s=server(5,new Set(['m1','m3']));
  const result=await markFolderRead(s,'a',['box'],auth);
  expect(result).toMatchObject({processed:3,refused:2});expect(result.changed).toEqual(['m0','m2','m4']);
 });
 it('includes subfolders by marking each folder\'s unread once',async()=>{
  const s=server(3);
  const result=await markFolderRead(s,'a',['box','sub'],auth);
  expect(result.total).toBe(3);expect(s.queries.map(q=>(q['filter'] as {inMailbox:string}).inMailbox)).toEqual(['box','sub']);
 });
 it('stops between batches when asked, and when the account changes',async()=>{
  const s=server(250);let calls=0;
  const stopped=await markFolderRead(s,'a',['box'],{current:()=>true,stopRequested:()=>++calls>1});
  expect(stopped.stopped).toBe(true);expect(s.sets.length).toBe(1);
  const gone=server(10);
  await expect(markFolderRead(gone,'a',['box'],{current:()=>false})).rejects.toThrow();expect(gone.jmap).not.toHaveBeenCalled();
 });
 it('copies by adding the destination only, and moves by swapping folders',async()=>{
  const copy=server(2);await transferFolder(copy,'a','src','dst','copy',auth);
  expect((copy.sets[0]!['update'] as Record<string,unknown>)['m0']).toEqual({'mailboxIds/dst':true});
  const move=server(2);const result=await transferFolder(move,'a','src','dst','move',auth);
  expect((move.sets[0]!['update'] as Record<string,unknown>)['m1']).toEqual({'mailboxIds/dst':true,'mailboxIds/src':null});
  expect(result.changed).toEqual(['m0','m1']);
  await expect(transferFolder(move,'a','src','src','copy',auth)).rejects.toThrow();
 });
 it('undoes a move and a mark-read from the ids they changed',async()=>{
  const s=server(0);
  await undoTransfer(s,'a','src','dst',['m0'],auth);
  expect((s.sets[0]!['update'] as Record<string,unknown>)['m0']).toEqual({'mailboxIds/src':true,'mailboxIds/dst':null});
  await markUnread(s,'a',['m0'],auth);
  expect((s.sets[1]!['update'] as Record<string,unknown>)['m0']).toEqual({'keywords/$seen':null});
 });
 it('escapes ids that contain JSON-pointer characters',async()=>{
  const s=server(1);await transferFolder(s,'a','s/1','d~1','copy',auth);
  expect((s.sets[0]!['update'] as Record<string,unknown>)['m0']).toEqual({'mailboxIds/d~01':true});
 });
 it('reads the newest message time of each folder in one request per 25 folders',async()=>{
  const jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>({methodResponses:request.methodCalls.filter(call=>call[0]==='Email/get').map(call=>['Email/get',{accountId:'a',list:call[2]==='g0'?[{receivedAt:'2026-01-02T00:00:00Z'}]:[]},call[2]] as never),sessionState:'s'}));
  const times=await lastActivity({jmap},'a',['x','y'],auth);
  expect(jmap).toHaveBeenCalledTimes(1);expect(times.get('x')).toBe(Date.parse('2026-01-02T00:00:00Z'));expect(times.has('y')).toBe(false);
 });
});

describe('folder structure',()=>{
 const ability={manages:true,maxLength:255};
 const boxes=[box('inbox',null,{role:'inbox'}),box('work',null),box('q3','work'),box('deep','q3'),box('home',null)];
 it('moves a folder under another one or to the top level, never into itself or below',()=>{
  expect(canMoveFolder(boxes[3]!,boxes,ability,'home')).toBe(true);
  expect(canMoveFolder(boxes[3]!,boxes,ability,null)).toBe(true);
  expect(canMoveFolder(boxes[1]!,boxes,ability,'deep')).toBe(false);
  expect(canMoveFolder(boxes[1]!,boxes,ability,'work')).toBe(false);
  expect(canMoveFolder(boxes[0]!,boxes,ability,'home')).toBe(false);
  expect(canMoveFolder(boxes[4]!,boxes,{manages:false,maxLength:255},'work')).toBe(false);
 });
 it('deletes a tree only when every folder in it is ordinary and deletable',()=>{
  expect(canDeleteTree(boxes[1]!,boxes,ability)).toBe(true);
  const guarded=boxes.map(value=>value.id==='deep'?{...value,myRights:{...rights,mayDelete:false}}:value);
  expect(canDeleteTree(guarded[1]!,guarded,ability)).toBe(false);
 });
 it('sends the new parent to Mailbox/set and reports a refusal',async()=>{
  const jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>({methodResponses:[['Mailbox/set',{accountId:'a',updated:(request.methodCalls[0]![1] as {update:{work:{parentId:string|null}}}).update.work.parentId==='home'?{work:null}:null,notUpdated:{work:{type:'forbidden'}}},'move-folder']],sessionState:'s'}));
  expect(await moveFolderTo({jmap},'a','work','home',auth)).toEqual({ok:true});
  expect(await moveFolderTo({jmap},'a','work',null,auth)).toEqual({ok:false,problem:'forbidden'});
 });
 it('deletes the deepest folder first and stops at a refusal',async()=>{
  const order:string[]=[];
  const jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{const call=request.methodCalls[0]![1] as {destroy:string[];onDestroyRemoveEmails:boolean};const id=call.destroy[0]!;order.push(id);return {methodResponses:[['Mailbox/set',id==='work'?{accountId:'a',destroyed:[],notDestroyed:{work:{type:'mailboxHasEmail'}}}:{accountId:'a',destroyed:[id]},'delete-folder']],sessionState:'s'};});
  const result=await deleteFolderTree({jmap},'a',boxes[1]!,boxes,true,auth);
  expect(order).toEqual(['deep','q3','work']);expect(result).toEqual({ok:false,problem:'mailboxHasEmail',deleted:2});
  expect(jmap.mock.calls[0]![0].methodCalls[0]![1]).toMatchObject({onDestroyRemoveEmails:true});
 });
});
