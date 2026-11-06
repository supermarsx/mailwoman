import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {RetainedSettingsSection} from '../RetainedSettingsSection.tsx';
import {ReaderAppearanceSettings,READER_CARDS} from '../ReaderAppearance.tsx';
import {MailFeaturePreferences} from '../MailFeaturePreferences.tsx';
import {MailInteractionPreferences} from '../MailInteractionPreferences.tsx';
import {clearSettings} from '../../../api/settings.ts';
import {t} from '../../../i18n';

let locked=false;
beforeEach(()=>{
 clearSettings();locked=false;localStorage.clear();
 vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({settings:{'reader.fontSize':{value:16,source:'global',locked}}}))));
});
afterEach(()=>{vi.unstubAllGlobals();clearSettings();});
const mount=()=>render(()=><RetainedSettingsSection name="reader" active accountBound={false}>{()=><><ReaderAppearanceSettings/></>}</RetainedSettingsSection>);
const cardToggle=async()=>(await screen.findByRole('heading',{name:t('reader-card-loading')})).closest('header')!.querySelector<HTMLElement>('[data-kit-toggle]')!;
const ALL_KEYS=['colorMode','fontSize','lineHeight','contentWidth','zoom','actionStyle','attachmentsDensity','attachmentsCollapsed','loading','loadingDelayMs'];

it('shows every message display setting exactly once, grouped into cards',async()=>{
 mount();
 for(const key of ALL_KEYS)expect(await screen.findAllByText(t(`reader-${key}`),{selector:'label'})).toHaveLength(1);
 expect(READER_CARDS.flatMap(c=>[...c.keys,...(c.leading?[c.leading]:[])]).sort()).toEqual([...ALL_KEYS].sort());
 expect(screen.getAllByRole('heading',{level:4})).toHaveLength(READER_CARDS.length);
});
it('builds a jump bar with one chip per card',async()=>{
 mount();
 const nav=await screen.findByRole('list',{name:t('settings-kit-jump')});
 expect(nav.querySelectorAll('button')).toHaveLength(READER_CARDS.length);
});
it('filters rows and cards by label or hint, and reports no results',async()=>{
 mount();
 const box=await screen.findByRole('textbox',{name:t('settings-kit-search')});
 fireEvent.input(box,{target:{value:'line height'}});
 await waitFor(()=>expect(screen.getByLabelText(t('reader-lineHeight')).closest('[data-setting-row]')).not.toHaveAttribute('data-kit-hidden'));
 expect(screen.getByLabelText(t('reader-fontSize')).closest('[data-setting-row]')).toHaveAttribute('data-kit-hidden');
 fireEvent.input(box,{target:{value:'zzzzqq'}});
 await waitFor(()=>expect(screen.getByRole('status')).toHaveTextContent(t('settings-kit-no-results')));
 fireEvent.input(box,{target:{value:''}});
 await waitFor(()=>expect(document.querySelectorAll('[data-kit-hidden]')).toHaveLength(0));
});
it('collapses a card and remembers it across mounts',async()=>{
 const first=mount();
 const toggle=await cardToggle();
 fireEvent.click(toggle);
 expect(toggle).toHaveAttribute('aria-expanded','false');
 expect(JSON.parse(localStorage.getItem(Object.keys(localStorage).find(k=>k.startsWith('mw.settings.cards.collapsed'))!)!)).toHaveLength(1);
 first.unmount();
 mount();
 expect(await cardToggle()).toHaveAttribute('aria-expanded','false');
 fireEvent.click(screen.getByRole('button',{name:t('settings-kit-collapse-all')}));
 expect(screen.getByRole('button',{name:t('settings-kit-expand-all')})).toBeInTheDocument();
});
it('keeps the administrator lock note on a held setting',async()=>{
 locked=true;mount();
 expect(await screen.findByTestId('setting-lock-note')).toBeInTheDocument();
 expect(screen.getByLabelText(t('reader-fontSize'))).toBeDisabled();
});
it('lists the shortcuts-section cards without dropping a setting',async()=>{
 render(()=><RetainedSettingsSection name="shortcuts" active accountBound={false}>{()=><><MailFeaturePreferences/><MailInteractionPreferences/></>}</RetainedSettingsSection>);
 expect(await screen.findAllByText(t('settings-row-actions-mode'),{selector:'label'})).toHaveLength(1);
 expect(document.querySelectorAll('[data-setting-row]').length).toBeGreaterThanOrEqual(5);
});
