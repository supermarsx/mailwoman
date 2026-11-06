import {createRoot,createSignal} from 'solid-js';
import {describe,expect,it,vi} from 'vitest';
import type {AccountUiPreferences} from '../api/accountPreferences.ts';
import {createFolderArranger} from './folderArrange.ts';
import {DEFAULT_POLICY,type DndFolder,type DndPolicy} from './folderDnd.ts';
const f=(id:string,parentId:string|null=null,extra:Partial<DndFolder>={}):DndFolder=>({id,name:id.toUpperCase(),parentId,role:null,totalEmails:0,unreadEmails:0,...extra});
const server=[f('inbox',null,{role:'inbox'}),f('a'),f('b'),f('c'),f('d','a')];
function setup(over:{prefs?:AccountUiPreferences;policy?:Partial<DndPolicy>;serverOk?:boolean;allowServer?:boolean}={}){
 const [prefs,setPrefs]=createSignal<AccountUiPreferences>(over.prefs??{});
 const recorded:{label:string;undo:()=>Promise<void>;redo:()=>Promise<void>}[]=[],failures:string[]=[],calls:[string,string|null][]=[];
 const arranger=createFolderArranger({
  folders:()=>server,prefs,update:patch=>setPrefs(value=>({...value,...patch})),policy:()=>({...DEFAULT_POLICY,...over.policy}),
  serverMoveAllowed:()=>over.allowServer??true,
  moveOnServer:async(id,parent)=>{calls.push([id,parent]);return over.serverOk??true;},
  record:action=>recorded.push(action),failed:problem=>failures.push(problem),label:(n,name)=>`moved ${n} ${name}`,
 });
 return {arranger,prefs,recorded,failures,calls};
}
const parent=(s:ReturnType<typeof setup>,id:string)=>s.arranger.visual().find(x=>x.id===id)?.parentId??null;
describe('folder arranger',()=>{
 it('reorders among siblings in preferences only, with undo and redo',async()=>{
  await createRoot(async dispose=>{
   const s=setup();
   expect(s.arranger.place(['c'],{parentId:null,beforeId:'a'})).toBe('applied');
   expect(s.arranger.visual().filter(x=>!x.parentId&&!x.role).map(x=>x.id)).toEqual(['c','a','b']);
   expect(s.calls).toEqual([]);expect(s.recorded).toHaveLength(1);
   await s.recorded[0]!.undo();
   expect(s.arranger.visual().filter(x=>!x.parentId&&!x.role).map(x=>x.id)).toEqual(['a','b','c']);
   await s.recorded[0]!.redo();
   expect(s.arranger.visual().filter(x=>!x.parentId&&!x.role).map(x=>x.id)).toEqual(['c','a','b']);
   dispose();
  });
 });
 it('refuses an invalid drop and tells why',()=>{
  createRoot(dispose=>{
   const s=setup({policy:{locked:true}});
   expect(s.arranger.place(['c'],{parentId:null,beforeId:'a'})).toBe('invalid');
   expect(s.failures).toEqual(['locked']);expect(s.recorded).toHaveLength(0);
   dispose();
  });
 });
 it('asks before changing the hierarchy and applies the choice, remembering it',async()=>{
  await createRoot(async dispose=>{
   const s=setup();
   expect(s.arranger.place(['c'],{parentId:'b',beforeId:null})).toBe('asked');
   expect(s.arranger.pending()?.moving).toEqual(['c']);expect(parent(s,'c')).toBeNull();
   s.arranger.choose('local',true);
   expect(parent(s,'c')).toBe('b');expect(s.prefs().folderNestMode).toBe('local');expect(s.prefs().folderParent).toEqual({c:'b'});expect(s.calls).toEqual([]);
   // Remembered: the next nest does not ask.
   expect(s.arranger.place(['d'],{parentId:null,beforeId:null})).toBe('applied');
   expect(s.arranger.pending()).toBeNull();
   dispose();
  });
 });
 it('moves on the server when chosen, clears the visual override and undoes on the server too',async()=>{
  await createRoot(async dispose=>{
   const s=setup({prefs:{folderNestMode:'server'}});
   expect(s.arranger.place(['c'],{parentId:'b',beforeId:null})).toBe('applied');
   await vi.waitFor(()=>expect(s.recorded).toHaveLength(1));
   expect(s.calls).toEqual([['c','b']]);expect(s.prefs().folderParent).toEqual({});
   await s.recorded[0]!.undo();
   expect(s.calls.at(-1)).toEqual(['c',null]);
   dispose();
  });
 });
 it('rolls back the optimistic move when the server refuses',async()=>{
  await createRoot(async dispose=>{
   const s=setup({prefs:{folderNestMode:'server'},serverOk:false});
   s.arranger.place(['c'],{parentId:'b',beforeId:null});
   await vi.waitFor(()=>expect(s.failures).toEqual(['server']));
   expect(s.prefs().folderParent).toEqual({});expect(parent(s,'c')).toBeNull();expect(s.recorded).toHaveLength(0);
   dispose();
  });
 });
 it('keeps nests local when the account or the administrator forbids server moves',()=>{
  createRoot(dispose=>{
   const s=setup({allowServer:false});
   expect(s.arranger.place(['c'],{parentId:'b',beforeId:null})).toBe('applied');
   expect(s.arranger.pending()).toBeNull();expect(s.prefs().folderParent).toEqual({c:'b'});
   dispose();
  });
 });
 it('refuses a server move onto a name already used there',()=>{
  createRoot(dispose=>{
   const clash=[...server,f('x','b',{name:'C'})];
   const [prefs]=createSignal<AccountUiPreferences>({folderNestMode:'server'});
   const failures:string[]=[];
   const arranger=createFolderArranger({folders:()=>clash,prefs,update:()=>undefined,policy:()=>DEFAULT_POLICY,serverMoveAllowed:()=>true,moveOnServer:async()=>true,record:()=>undefined,failed:p=>failures.push(p),label:()=>''});
   expect(arranger.place(['c'],{parentId:'b',beforeId:null})).toBe('invalid');
   expect(failures).toEqual(['collision']);
   dispose();
  });
 });
 it('moves a multi-selection together, keeping its order, and resets',()=>{
  createRoot(dispose=>{
   const s=setup();
   s.arranger.place(['b','c'],{parentId:null,beforeId:'a'});
   expect(s.arranger.visual().filter(x=>!x.parentId&&!x.role).map(x=>x.id)).toEqual(['b','c','a']);
   s.arranger.reset();
   expect(s.prefs().folderOrder).toEqual([]);expect(s.prefs().folderParent??{}).toEqual({});
   dispose();
  });
 });
 it('tolerates stored order and parents naming folders that no longer exist',()=>{
  createRoot(dispose=>{
   const s=setup({prefs:{folderOrder:['gone','c','a'],folderParent:{b:'gone',c:'_top'}}});
   expect(s.arranger.visual().filter(x=>!x.parentId&&!x.role).map(x=>x.id)).toEqual(['c','a','b']);
   dispose();
  });
 });
});
