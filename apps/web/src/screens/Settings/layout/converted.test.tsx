import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen} from '@solidjs/testing-library';
import type {JSX} from 'solid-js';
import {InterfacePreferences,APPEARANCE_TEXT_ROWS} from '../InterfacePreferences.tsx';
import {FolderCountPreferences} from '../FolderCountPreferences.tsx';
import {FocusedInboxPreference} from '../FocusedInboxPreference.tsx';
import {CachePreferences} from '../CachePreferences.tsx';
import {ContactAppearance} from '../ContactAppearance.tsx';
import {UndoRedoPreferences} from '../UndoRedoPreferences.tsx';
import {clearSettings} from '../../../api/settings.ts';
import {t} from '../../../i18n';
import {SETTINGS_REGISTRY} from './registry.ts';

vi.mock('../../../state/context.ts',()=>({useApp:()=>({clearMailCache:()=>undefined})}));
beforeEach(()=>{clearSettings();vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({settings:{'mail.folderCounts':{value:'both',source:'global',locked:true}}}))));});
afterEach(()=>{cleanup();vi.unstubAllGlobals();clearSettings();});

// Every converted component, the registry cards it renders, and extra labels (not
// in the registry) the previous implementation also showed exactly once.
const CASES:{name:string;view:()=>JSX.Element;cards:string[];extra?:string[]}[]=[
 {name:'appearance interface rows',view:()=><InterfacePreferences/>,cards:['settings-appearance-text-card']},
 {name:'folder counters',view:()=><FolderCountPreferences/>,cards:['folders-card-counts']},
 {name:'focused inbox',view:()=><FocusedInboxPreference/>,cards:[],extra:['settings-focused-inbox']},
 {name:'offline cache',view:()=><CachePreferences/>,cards:['cache-card-store','cache-card-prefetch']},
 {name:'avatars',view:()=><ContactAppearance/>,cards:['avatars-card']},
 {name:'undo history',view:()=><UndoRedoPreferences/>,cards:['undo-card']},
];

const RENDERED_ELSEWHERE=new Set<string>(APPEARANCE_TEXT_ROWS.map(r=>r.id));
for(const c of CASES){
 it(`${c.name}: each previous setting label appears exactly once`,async()=>{
  render(c.view);
  await screen.findAllByText(/./,{selector:'label'});
  for(const id of c.cards){
   const card=SETTINGS_REGISTRY.find(x=>x.id===id)!;
   expect(card,id).toBeDefined();
   for(const row of card.rows){
    if(RENDERED_ELSEWHERE.has(row.id))continue;
    expect(document.querySelectorAll(`[id="${row.id}"]`),row.id).toHaveLength(1);
    expect(screen.getAllByText(t(row.label),{selector:'label'}),row.label).toHaveLength(1);
   }
  }
  for(const key of c.extra??[])expect(screen.getAllByText(t(key),{selector:'label'}),key).toHaveLength(1);
 });
}
it('keeps the administrator lock note on a held converted setting',async()=>{
 render(()=><FolderCountPreferences/>);
 expect(await screen.findByTestId('setting-lock-note')).toBeInTheDocument();
 expect(screen.getByLabelText(t('settings-folder-counts'))).toBeDisabled();
});
