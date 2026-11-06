import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {render,screen} from '@solidjs/testing-library';
import {RetainedSettingsSection} from '../RetainedSettingsSection.tsx';
import {ReaderAppearanceSettings} from '../ReaderAppearance.tsx';
import {MailFeaturePreferences} from '../MailFeaturePreferences.tsx';
import {MailInteractionPreferences} from '../MailInteractionPreferences.tsx';
import {ComposePreferences} from '../ComposePreferences.tsx';
import {DraftSavePreferences} from '../DraftSavePreferences.tsx';
import {clearSettings} from '../../../api/settings.ts';
import {t} from '../../../i18n';
import {SETTINGS_REGISTRY,searchRegistry} from './registry.ts';

beforeEach(()=>{clearSettings();localStorage.clear();vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({settings:{}}))));});
afterEach(()=>{vi.unstubAllGlobals();clearSettings();});

const SECTIONS:Record<string,()=>unknown>={
 reader:()=><ReaderAppearanceSettings/>,
 shortcuts:()=><><MailFeaturePreferences/><MailInteractionPreferences/></>,
 compose:()=><><ComposePreferences/><DraftSavePreferences/></>,
};
// Rows that exist only for some values of another setting.
const CONDITIONAL=new Set(['draft-interval']);

for(const [name,body] of Object.entries(SECTIONS)){
 it(`${name}: every registry row is rendered exactly once and each label appears once`,async()=>{
  render(()=><RetainedSettingsSection name={name} active accountBound={false}>{body as ()=>never}</RetainedSettingsSection>);
  await screen.findAllByRole('heading',{level:4});
  const specs=SETTINGS_REGISTRY.filter(c=>c.section===name);
  expect(specs.length).toBeGreaterThan(0);
  for(const card of specs){
   expect(document.querySelectorAll(`[id="${card.id}"]`)).toHaveLength(1);
   for(const row of card.rows){
    if(CONDITIONAL.has(row.id))continue;
    expect(document.querySelectorAll(`[id="${row.id}"]`),row.id).toHaveLength(1);
    expect(screen.getAllByText(t(row.label),{selector:'label'}),row.label).toHaveLength(1);
   }
  }
 });
}
it('finds rows of other sections through the static registry',()=>{
 const hits=searchRegistry('forward header','reader');
 expect(hits.some(h=>h.section==='compose'&&h.row==='quote-style')).toBe(true);
 expect(searchRegistry('forward header','compose').some(h=>h.section==='compose')).toBe(false);
 expect(searchRegistry('','reader')).toEqual([]);
});
