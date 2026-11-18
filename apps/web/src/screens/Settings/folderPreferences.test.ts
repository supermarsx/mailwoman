import {expect,it} from 'vitest';
import {FOLDER_ICONS} from '../../api/accountPreferences.ts';
import {folderIcon,folderIsHidden,shownFolderIds,folderPathLabel,moveFolder,orderedFolders} from './folderPreferenceModel.ts';
it('offers 159 native custom icons and role-specific defaults',()=>{
 expect(new Set(FOLDER_ICONS).size).toBe(159);
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
it('hides descendants by parent IDs, protects Inbox and terminates cyclic folder metadata',()=>{
 const folders=[{id:'i',name:'Inbox',role:'inbox'},{id:'a',name:'A'},{id:'b',name:'B',parentId:'a'},{id:'c',name:'C',parentId:'b'},{id:'x',name:'A/Other'},{id:'cycle',name:'Cycle',parentId:'cycle'}];
 const prefs={folderHidden:['a','i'],folderPinned:['b']};
 expect(folders.map(folder=>folderIsHidden(folder,folders,prefs))).toEqual([false,true,true,true,false,false]);
 expect(shownFolderIds(folders[3]!,folders,['a','b','c','x'])).toEqual(['x']);
});
import {FOLDER_ICON_GROUPS} from './folderPreferenceModel.ts';
import {ICON_NAMES} from '../../components/Icon.tsx';
const NEW_ICONS=['camera','music','film','lightbulb','rocket','trophy','leaf','globe','cloud','bookmark','wallet','medical','bell','key','receipt','cart','package','wrench','chart','team','store','office','laptop','crown','family'];
it('adds 25 new icons, each drawn, grouped exactly once, and unknown stored ids fall back',()=>{
 expect(NEW_ICONS).toHaveLength(25);
 for(const id of NEW_ICONS){expect(FOLDER_ICONS as readonly string[]).toContain(id);}
 for(const id of FOLDER_ICONS)expect(ICON_NAMES).toContain(id);
 const grouped=FOLDER_ICON_GROUPS.flatMap(group=>group.icons);
 expect([...grouped].sort()).toEqual([...FOLDER_ICONS].sort());
 expect(folderIcon({id:'a',name:'A'},{folderIcons:{a:'nope' as never}})).toBe('folder');
 expect(folderIcon({id:'a',name:'A',role:'trash'},{folderIcons:{a:'rocket'}})).toBe('rocket');
});

import {readFileSync,readdirSync} from 'node:fs';
import {resolve,join} from 'node:path';
it('keeps persisted server choices and every translated picker label in sync',()=>{
 const server=readFileSync(resolve(process.cwd(),'../../crates/mw-server/src/prefs_routes/ui_preferences.rs'),'utf8');
 const list=server.match(/const FOLDER_ICONS:[\s\S]*?= &\[([\s\S]*?)\];/)![1]!;
 expect([...list.matchAll(/"([^"\n]+)"/g)].map(match=>match[1])).toEqual([...FOLDER_ICONS]);
 const locales=resolve(process.cwd(),'locales');
 for(const locale of readdirSync(locales,{withFileTypes:true}).filter(entry=>entry.isDirectory())){
  const text=readFileSync(join(locales,locale.name,'settings.ftl'),'utf8');
  for(const id of FOLDER_ICONS)expect(text,`${locale.name}: ${id}`).toMatch(new RegExp(`^settings-folder-icon-${id} = .+`,'m'));
  for(const group of FOLDER_ICON_GROUPS)expect(text,`${locale.name}: ${group.id}`).toMatch(new RegExp(`^settings-folder-icon-group-${group.id} = .+`,'m'));
 }
});
