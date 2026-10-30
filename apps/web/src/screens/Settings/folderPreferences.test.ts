import {expect,it} from 'vitest';
import {FOLDER_ICONS} from '../../api/accountPreferences.ts';
import {folderIcon,folderPathLabel,moveFolder,orderedFolders} from './folderPreferenceModel.ts';
it('offers fifty native custom icons and role-specific defaults',()=>{
 expect(new Set(FOLDER_ICONS).size).toBe(50);
 expect(folderIcon({id:'a',name:'Inbox',role:'inbox'},{})).toBe('inbox');
 expect(folderIcon({id:'b',name:'Sent',role:'sent'},{})).toBe('sent');
 expect(folderIcon({id:'b',name:'Sent',role:'sent'},{folderIcons:{b:'star'}})).toBe('star');
});
it('pins before explicit order without changing source folders or losing new ones',()=>{
 const folders=[{id:'a',name:'A'},{id:'b',name:'B'},{id:'c',name:'C'},{id:'new',name:'New'}];
 expect(orderedFolders(folders,{folderPinned:['c'],folderOrder:['b','a']}).map(v=>v.id)).toEqual(['c','b','a','new']);
 expect(folders.map(v=>v.id)).toEqual(['a','b','c','new']);
});
it('moves in either direction and ignores unknown drag targets',()=>{
 expect(moveFolder(['a','b','c'],'a','b')).toEqual(['b','a','c']);
 expect(moveFolder(['a','b','c'],'c','b')).toEqual(['a','c','b']);
 expect(moveFolder(['a','b','c'],'unknown','b')).toEqual(['a','b','c']);
});

it('uses a search icon for virtual folders while honoring custom icons', () => {
 expect(folderIcon({id:'s',name:'Invoices',mailwomanSearchQuery:'{}'},{})).toBe('search');
 expect(folderIcon({id:'s',name:'Invoices',mailwomanSearchQuery:'{}'},{folderIcons:{s:'star'}})).toBe('star');
});
it('labels a folder by its decoded path, never the modified UTF-7 server spelling (t92)', () => {
 expect(folderPathLabel({id:'d',name:'Ältere',mailwomanUpstreamName:'Entw&APw-rfe/&AMQ-ltere',mailwomanPath:'Entwürfe/Ältere'})).toBe('Entwürfe/Ältere');
 expect(folderPathLabel({id:'k',name:'Корзина',mailwomanUpstreamName:'&BBoEPgRABDcEOAQ9BDA-'})).toBe('Корзина');
});
