import {expect,it,vi} from 'vitest';
import {defaultAppearance} from '../../theme/appearance.ts';
import type {SettingsMap} from '../../api/settings.ts';
import {applyReviewed,buildBackup,parseBackup,reviewBackup,TRANSFER_GROUPS,TRANSFER_LIMIT,type SettingsBackup} from './settingsTransferModel.ts';
const source={accountId:'a',origin:'https://mail.example.org'};
const map:SettingsMap={
 'reader.actions.reply':{value:'toolbar',source:'global',locked:false},
 'reader.fontSize':{value:14,source:'user',locked:false,userValue:14},
 'reader.zoom':{value:100,source:'global',locked:true,userValue:125},
 'mail.focusedInbox':{value:false,source:'user',locked:false,userValue:false},
 'mail.readReceipts':{value:'ask',source:'user',locked:false,userValue:'ask'},
};
const file=(items:SettingsBackup['items']):SettingsBackup=>({format:'mailwoman-user-settings',version:1,createdAt:new Date().toISOString(),source,items});
it('exports personal values, including false and locked stored choices, without copying policy, consent, or executable customization',()=>{
 const backup=buildBackup(map,{keyboardPreset:'vim',analyticsConsent:true,analyticsTagConsent:'secret',folderProps:{f:{notes:'private note'}}},defaultAppearance(),source,TRANSFER_GROUPS);
 expect(backup.items).toContainEqual({kind:'setting',key:'mail.focusedInbox',value:false});
 expect(backup.items).toContainEqual({kind:'setting',key:'reader.zoom',value:125});
 expect(backup.items).not.toContainEqual(expect.objectContaining({key:'reader.actions.reply'}));
 expect(backup.items).not.toContainEqual(expect.objectContaining({key:'mail.readReceipts'}));
 expect(backup.items).toContainEqual({kind:'interface',key:'keyboardPreset',value:'vim'});
 expect(backup.items.some(x=>x.key==='analyticsConsent'||x.key==='analyticsTagConsent'||x.key==='folderProps')).toBe(false);
 expect(parseBackup(JSON.stringify(backup))).toEqual(backup);
});
it('includes folder and workspace settings only when explicitly selected and filters categories',()=>{
 const backup=buildBackup(map,{folderHidden:['f'],keyboardPreset:'vim'},defaultAppearance(),source,['folders'],true);
 expect(backup.items).toEqual([{kind:'interface',key:'folderHidden',value:['f']}]);
});
it.each([
 [JSON.stringify({format:'other',version:1,source,createdAt:'x',items:[]}), 'st-file-version'],
 [JSON.stringify({...file([]),version:2}),'st-file-version'],
 ['not json','st-file-invalid'],
 ['{"format":"mailwoman-user-settings","version":1,"__proto__":{}}','st-file-invalid'],
 [JSON.stringify(file([{kind:'setting',key:'x',value:1},{kind:'setting',key:'x',value:2}])),'st-file-invalid'],
 ['x'.repeat(TRANSFER_LIMIT+1),'st-file-large'],
])('rejects malformed, future, duplicate, and oversized files', (text,error)=>expect(()=>parseBackup(text)).toThrow(error));
it('classifies changed, unchanged, locked, unknown, and invalid preferences before applying anything',()=>{
 const rows=reviewBackup(file([
  {kind:'setting',key:'reader.fontSize',value:16},
  {kind:'setting',key:'mail.focusedInbox',value:false},
  {kind:'setting',key:'reader.zoom',value:125},
  {kind:'setting',key:'unknown.key',value:true},
  {kind:'setting',key:'reader.actions.reply',value:7},
 ]),map,{},defaultAppearance(),source);
 expect(rows.map(x=>x.status)).toEqual(['ready','unchanged','locked','unknown','invalid']);
});
it('prevents account-specific settings from being restored onto another account or server',()=>{
 for(const target of [{...source,accountId:'b'},{...source,origin:'https://other.example.org'}]) {
  const rows=reviewBackup(file([{kind:'interface',key:'folderHidden',value:['f']}]),map,{},defaultAppearance(),target);
  expect(rows[0]?.status).toBe('account');
 }
 expect(reviewBackup(file([{kind:'interface',key:'folderHidden',value:['f']}]),map,{},defaultAppearance(),source)[0]?.status).toBe('ready');
});
it('validates portable UI choices and appearance instead of silently defaulting malformed values',()=>{
 const rows=reviewBackup(file([
  {kind:'interface',key:'keyboardPreset',value:'unknown'},
  {kind:'interface',key:'messageListWidth',value:-5},
  {kind:'interface',key:'analyticsConsent',value:true},
  {kind:'appearance',key:'appearance',value:{...defaultAppearance(),theme:'made-up'}},
 ]),map,{},defaultAppearance(),source);
 expect(rows.map(x=>x.status)).toEqual(['invalid','invalid','unknown','invalid']);
});
it('holds the appearance bundle when it would change a locked appearance field',()=>{
 const rows=reviewBackup(file([{kind:'appearance',key:'appearance',value:{...defaultAppearance(),density:'compact'}}]),{'appearance.density':{value:'cozy',source:'global',locked:true}},{},defaultAppearance(),source);
 expect(rows[0]?.status).toBe('locked');
});
it('merges only selected eligible changes and reports partial failures without claiming all succeeded',async()=>{
 const rows=reviewBackup(file([{kind:'setting',key:'reader.fontSize',value:16},{kind:'setting',key:'reader.actions.reply',value:'menu'},{kind:'setting',key:'reader.zoom',value:125}]),map,{},defaultAppearance(),source);
 const write=vi.fn(async row=>{if(row.key==='reader.actions.reply')throw new Error('refused');});const result=vi.fn();
 await applyReviewed(rows,new Set(rows.map(x=>x.id)),()=>true,write,result);
 expect(write).toHaveBeenCalledTimes(2);expect(result).toHaveBeenCalledWith('setting:reader.fontSize');expect(result).toHaveBeenCalledWith('setting:reader.actions.reply',expect.any(Error));
});
it('stops on an account change and does not report an old in-flight write as saved for the new account',async()=>{
 let current=true;
 const rows=reviewBackup(file([{kind:'setting',key:'reader.fontSize',value:16},{kind:'setting',key:'reader.actions.reply',value:'menu'}]),map,{},defaultAppearance(),source);
 const result=vi.fn(),write=vi.fn(async()=>{current=false;});
 await applyReviewed(rows,new Set(rows.map(x=>x.id)),()=>current,write,result);
 expect(write).toHaveBeenCalledOnce();expect(result).not.toHaveBeenCalled();
});

it('allows layout restoration when unrelated appearance settings are locked',()=>{
 const rows=reviewBackup(file([{kind:'appearance',key:'appearance',value:{...defaultAppearance(),layout:'ribbon'}}]),{'appearance.theme':{value:defaultAppearance().theme,source:'global',locked:true},'appearance.fontSize':{value:14,source:'global',locked:true}},{},defaultAppearance(),source);
 expect(rows[0]?.status).toBe('ready');
});

it('does not freeze inherited values into exported local choices',()=>{
 const backup=buildBackup({'reader.zoom':{value:125,source:'default',locked:false,inherited:true}}, {},defaultAppearance(),source,TRANSFER_GROUPS);
 expect(backup.items.some(item=>item.key==='reader.zoom')).toBe(false);
});
it('rejects folder IDs from a different verified authority even when the account id matches',()=>{
 const backup={...file([{kind:'interface',key:'folderHidden',value:['f']}]),source:{...source,storageAuthority:'authority-one'}};
 expect(reviewBackup(backup,map,{},defaultAppearance(),{...source,storageAuthority:'authority-two'})[0]?.status).toBe('account');
});
