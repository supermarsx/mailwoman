import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,fireEvent} from '@solidjs/testing-library';
import {FolderPreferences} from '../FolderPreferences.tsx';
import {FolderRoles} from '../FolderRoles.tsx';
import {clearSettings} from '../../../api/settings.ts';
import {t} from '../../../i18n';
import {SETTINGS_REGISTRY} from './registry.ts';

beforeEach(()=>{clearSettings();vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({settings:{}}))));});
afterEach(()=>{cleanup();vi.unstubAllGlobals();clearSettings();});

const FOLDERS=()=>[{id:'inbox',name:'Inbox',role:'inbox',mailwomanUpstreamName:'INBOX'},{id:'a',name:'Projects',mailwomanUpstreamName:'Projects'}];
it('Folders: every previous preference label appears once, in registered cards',async()=>{
 render(()=><><FolderPreferences folders={FOLDERS} preferences={()=>({})} onChange={()=>undefined}/><FolderRoles folders={FOLDERS}/></>);
 await screen.findByRole('table');
 const cards=SETTINGS_REGISTRY.filter(c=>c.section==='folders'&&['folders-card-display','folders-arrange-card','folders-card-list','folders-card-roles'].includes(c.id));
 expect(cards).toHaveLength(4);
 for(const card of cards){
  expect(document.querySelectorAll(`[id="${card.id}"]`),card.id).toHaveLength(1);
  for(const row of card.rows){
   expect(document.querySelectorAll(`[id="${row.id}"]`),row.id).toHaveLength(1);
   expect(screen.getAllByText(t(row.label),{selector:'label,span[id]'}),row.label).toHaveLength(1);
  }
 }
 // previous controls that are not rows: the create button is gated on admin rights; the table stays.
 fireEvent.click(screen.getByRole('tab',{name:t('folder-dnd-arrange-title')}));
 expect(screen.getByRole('button',{name:t('folder-dnd-reset')})).toBeInTheDocument();
});
it('every kit card rendered in Folders is registered',async()=>{
 render(()=><><FolderPreferences folders={FOLDERS} preferences={()=>({})} onChange={()=>undefined}/><FolderRoles folders={FOLDERS}/></>);
 await screen.findByRole('table');
 const ids=new Set(SETTINGS_REGISTRY.map(c=>c.id));
 for(const el of document.querySelectorAll('section[id][aria-labelledby]'))expect(ids.has(el.id),el.id).toBe(true);
});
