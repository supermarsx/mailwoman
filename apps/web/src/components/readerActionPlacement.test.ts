import {describe,it,expect} from 'vitest';
import {READER_ACTIONS,READER_ACTION_PLACEMENTS,isReaderActionPlacement,readerActionKey,readerActionPlacement,readerMenuActionId} from './readerActionPlacement.ts';
import {PRIORITY} from './ReaderToolbar.tsx';
describe('reader action placement contract',()=>{
 it('has unique registry-safe keys and preserves all existing defaults',()=>{
  expect(READER_ACTIONS).toHaveLength(33);expect(new Set(READER_ACTIONS.map(item=>item.id)).size).toBe(33);
  expect(READER_ACTIONS.filter(item=>item.defaultPlacement==='toolbar')).toHaveLength(23);
  expect(READER_ACTIONS.filter(item=>item.defaultPlacement==='menu')).toHaveLength(9);
  expect(READER_ACTIONS.filter(item=>item.defaultPlacement==='hidden').map(item=>item.id)).toEqual(['delete-sender']);
  for(const item of READER_ACTIONS){expect(item.id).toMatch(/^[a-z][a-z0-9-]*$/);expect(readerActionKey(item.id)).toBe(`reader.actions.${item.id}`);expect(readerActionPlacement(item.id,{})).toBe(item.defaultPlacement);}
  expect(Object.keys(PRIORITY).sort()).toEqual(READER_ACTIONS.filter(item=>item.defaultPlacement==='toolbar').map(item=>item.id).sort());
 });
 it('honors effective user/admin values and falls back on malformed or absent values',()=>{
  const adminValue={'reader.actions.reply':{value:'hidden',source:'global',locked:true}};expect(readerActionPlacement('reply',adminValue)).toBe('hidden');
  for(const value of READER_ACTION_PLACEMENTS){expect(isReaderActionPlacement(value)).toBe(true);expect(readerActionPlacement('reply',{'reader.actions.reply':{value}})).toBe(value);}
  for(const value of [null,undefined,true,0,'more','MENU',{},[]]){expect(isReaderActionPlacement(value)).toBe(false);expect(readerActionPlacement('reply',{'reader.actions.reply':{value}})).toBe('toolbar');expect(readerActionPlacement('save-contacts',{'reader.actions.save-contacts':{value}})).toBe('menu');}
 });
 it('maps colon menu IDs to stable preference keys while leaving unknown future actions in More',()=>{
  expect(readerMenuActionId('focus:focused')).toBe('focus-focused');expect(readerMenuActionId('focus:other')).toBe('focus-other');
  expect(readerMenuActionId('save-contacts')).toBe('save-contacts');expect(readerMenuActionId('future-action')).toBeUndefined();
  expect(readerActionPlacement('future-action',{'reader.actions.future-action':{value:'toolbar'}})).toBe('menu');
 });
});
