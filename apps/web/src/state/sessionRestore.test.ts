import {afterEach,describe,expect,it} from 'vitest';
import {SESSION_RESTORE_KEY,forgetSession,parseSession,readSession,readUrlTarget,rememberActiveMailbox,rememberView,restoreMailbox,restoreView,savedView,writeUrlTarget} from './sessionRestore.ts';

const USER='msa1_parent_authority';
const OTHER='msa1_someone_else_authority';
const PRIMARY={id:'acct-parent',storageAuthority:USER};
const CHILD={id:'acct-child',storageAuthority:'msa1_child_authority'};
afterEach(()=>{sessionStorage.clear();history.replaceState(null,'','/');});

describe('t109 session record',()=>{
  it('keeps the active mailbox and a view per mailbox for the signed-in person',()=>{
    rememberActiveMailbox(USER,'link-1');
    rememberView(USER,CHILD,{folder:'f1',message:'m1',anchor:'m9',search:'from:a'});
    rememberView(USER,PRIMARY,{folder:'inbox'});
    const record=readSession(USER)!;
    expect(record.active).toBe('link-1');
    expect(record.views['acct-child']).toEqual({authority:CHILD.storageAuthority,folder:'f1',message:'m1',anchor:'m9',search:'from:a'});
    expect(savedView(USER,PRIMARY)).toEqual({authority:USER,folder:'inbox'});
    // A view update keeps the active mailbox; an active update keeps the views.
    rememberActiveMailbox(USER,null);
    expect(readSession(USER)?.views['acct-child']?.folder).toBe('f1');
    expect(readSession(USER)?.active).toBeNull();
  });
  it('a field left out is cleared, so a closed pane or a cleared search does not come back',()=>{
    rememberView(USER,CHILD,{folder:'f1',message:'m1',search:'x'});
    rememberView(USER,CHILD,{folder:'f1'});
    expect(savedView(USER,CHILD)).toEqual({authority:CHILD.storageAuthority,folder:'f1'});
  });
  it('another sign-in never reads it, and the stale record is removed',()=>{
    rememberActiveMailbox(USER,'link-1');
    expect(readSession(OTHER)).toBeNull();
    expect(sessionStorage.getItem(SESSION_RESTORE_KEY)).toBeNull();
    expect(restoreMailbox(OTHER,null)).toBeUndefined();
  });
  it('a view saved under another authority for the same account id is not used',()=>{
    rememberView(USER,{id:'same',storageAuthority:'msa1_realm_a_authority'},{folder:'f1'});
    expect(savedView(USER,{id:'same',storageAuthority:'msa1_realm_b_authority'})).toBeNull();
  });
  it('rejects malformed records and fields',()=>{
    for(const raw of ['{','null','[]',JSON.stringify({version:2,user:USER,active:null,views:{}}),JSON.stringify({version:1,user:USER,active:'',views:{}}),JSON.stringify({version:1,user:USER,active:'bad\u0001id',views:{}}),'x'.repeat(70000)])
      expect(parseSession(raw,USER)).toBeNull();
    const record=parseSession(JSON.stringify({version:1,user:USER,active:null,views:{a:{authority:USER,folder:'ok',message:'\u0000',anchor:7,search:' '},b:{authority:'short'},'':{authority:USER}}}),USER)!;
    expect(record.views).toEqual({a:{authority:USER,folder:'ok'}});
    expect(parseSession('{}','not an authority')).toBeNull();
    // Writes with invalid ids are ignored.
    rememberActiveMailbox(USER,'');rememberView(USER,{id:'x'},{folder:'f'});
    expect(sessionStorage.getItem(SESSION_RESTORE_KEY)).toBeNull();
  });
  it('keeps at most 16 mailbox views, dropping the oldest',()=>{
    for(let index=0;index<20;index++)rememberView(USER,{id:`acct-${index}`,storageAuthority:USER},{folder:'f'});
    const ids=Object.keys(readSession(USER)!.views);
    expect(ids).toHaveLength(16);expect(ids[0]).toBe('acct-4');expect(ids.at(-1)).toBe('acct-19');
  });
  it('storage that throws is no error',()=>{
    const original=Object.getOwnPropertyDescriptor(globalThis,'sessionStorage')!;
    Object.defineProperty(globalThis,'sessionStorage',{configurable:true,get(){throw new Error('denied');}});
    try{
      expect(()=>rememberActiveMailbox(USER,'l')).not.toThrow();
      expect(readSession(USER)).toBeNull();
      expect(()=>forgetSession()).not.toThrow();
    }finally{Object.defineProperty(globalThis,'sessionStorage',original);}
  });
  it('forgetSession removes the record and the address-bar target, keeping the route',()=>{
    rememberActiveMailbox(USER,'link-1');history.replaceState(null,'','/?mailbox=link-1&folder=f&keep=1#/calendar');
    forgetSession();
    expect(sessionStorage.getItem(SESSION_RESTORE_KEY)).toBeNull();
    expect(location.search).toBe('?keep=1');expect(location.hash).toBe('#/calendar');
  });
});

describe('t109 address bar',()=>{
  it('writes the mailbox id, folder and message in place and reads them back',()=>{
    history.replaceState(null,'','/#/mail');const length=history.length;
    writeUrlTarget({mailbox:'link-1',folder:'f1',message:'m1'});
    expect(location.search).toBe('?mailbox=link-1&folder=f1&message=m1');expect(location.hash).toBe('#/mail');
    expect(history.length).toBe(length);
    expect(readUrlTarget()).toEqual({mailbox:'link-1',folder:'f1',message:'m1'});
    writeUrlTarget({mailbox:null,folder:'f2'});
    expect(location.search).toBe('?folder=f2');
    expect(readUrlTarget()).toEqual({mailbox:null,folder:'f2'});
  });
  it('names no target without its parameters, and drops malformed ids',()=>{
    expect(readUrlTarget('')).toBeNull();expect(readUrlTarget('?other=1')).toBeNull();
    expect(readUrlTarget('?mailbox=&folder=%00')).toEqual({mailbox:null});
  });
  it('the address bar wins over the session for the mailbox; the session applies without it',()=>{
    rememberActiveMailbox(USER,'link-1');
    expect(restoreMailbox(USER,{mailbox:'link-2'})).toEqual({linkId:'link-2',source:'url'});
    expect(restoreMailbox(USER,{mailbox:null,folder:'f'})).toEqual({linkId:null,source:'url'});
    expect(restoreMailbox(USER,null)).toEqual({linkId:'link-1',source:'session'});
    expect(restoreMailbox(undefined,null)).toBeUndefined();
  });
  it('merges the address bar folder and message over the saved view of the same mailbox',()=>{
    rememberView(USER,CHILD,{folder:'f1',message:'m1',anchor:'a1',search:'s'});
    // Same folder: the anchor and search stay; the message is the address bar's.
    expect(restoreView(USER,CHILD,'link-1',{mailbox:'link-1',folder:'f1',message:'m2'})).toEqual({folder:'f1',message:'m2',anchor:'a1',search:'s'});
    // Another folder: its anchor and search were for the old one.
    expect(restoreView(USER,CHILD,'link-1',{mailbox:'link-1',folder:'f2'})).toEqual({folder:'f2'});
    // The address bar names another mailbox: only the saved view counts.
    expect(restoreView(USER,CHILD,'link-1',{mailbox:'link-9',folder:'x',message:'y'})).toEqual({folder:'f1',message:'m1',anchor:'a1',search:'s'});
    expect(restoreView(USER,CHILD,'link-1',null)).toEqual({folder:'f1',message:'m1',anchor:'a1',search:'s'});
    expect(restoreView(OTHER,CHILD,'link-1',null)).toBeNull();
  });
});
