import {afterEach,expect,it,vi} from 'vitest';
import type {AccountUiPreferences} from '../api/accountPreferences.ts';
const h=vi.hoisted(()=>({prefs:{} as AccountUiPreferences,map:{} as Record<string,{value:unknown;locked:boolean;source:string}>}));
vi.mock('../api/settings.ts',()=>({settings:()=>h.map}));
vi.mock('../api/accountPreferences.ts',()=>({accountUiPreferences:()=>h.prefs,setAccountUiPreferences:(patch:AccountUiPreferences)=>{h.prefs={...h.prefs,...patch};}}));
import {folderViewValue,folderColumnWidth,resetFolderColumnWidth,setFolderView,folderViewFilter,setActiveFolderViewId} from './folderViewPreferences.ts';
afterEach(()=>{h.prefs={};h.map={};setActiveFolderViewId(undefined);});
it('isolates each folder, inherits account defaults, and applies administrator locks immediately',()=>{
 h.map['mail.messageListFontSize']={value:16,source:'global',locked:false};
 setFolderView('a',{fontSize:22,group:'sender',filter:'unread',columns:['size','subject']});
 expect(folderViewValue('fontSize','a')).toBe(22);expect(folderViewValue('fontSize','b')).toBe(16);
 expect(folderViewValue('columns','a')).toEqual(['size','subject']);expect(folderViewFilter('a')).toEqual({notKeyword:'$seen'});
 h.map['mail.messageListFontSize']!.locked=true;expect(folderViewValue('fontSize','a')).toBe(16);
 setFolderView('a',{fontSize:24});expect(h.prefs.folderProps?.['a']?.view?.fontSize).toBe(22);
 setFolderView('a',{group:undefined});expect(folderViewValue('group','a')).toBe('none');
});
it('policy disables folder overrides without deleting them, including widths',()=>{
 setFolderView('a',{columnWidths:{sender:220},readingPane:'bottom'});
 expect(folderColumnWidth('sender','a')).toBe(220);resetFolderColumnWidth('a','sender');expect(folderColumnWidth('sender','a')).toBe(180);setFolderView('a',{columnWidths:{sender:220}});
 h.map['mail.messageListColumnWidth.sender']={value:150,source:'global',locked:true};expect(folderColumnWidth('sender','a')).toBe(150);
 h.map['properties.folder.overrides.allowed']={value:[],source:'global',locked:true};
 expect(folderViewValue('readingPane','a')).toBe('right');setFolderView('a',{readingPane:'off'});
 expect(h.prefs.folderProps?.['a']?.view?.readingPane).toBe('bottom');
});
