// Navigation is the section list (SettingsNav) and a section's own tab row; the
// layout kit adds no navigation of its own (no jump bar, rail, toolbar or collapse).
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {RetainedSettingsSection} from '../RetainedSettingsSection.tsx';
import {ReaderAppearanceSettings,READER_CARDS} from '../ReaderAppearance.tsx';
import {MailFeaturePreferences} from '../MailFeaturePreferences.tsx';
import {MailInteractionPreferences} from '../MailInteractionPreferences.tsx';
import {FolderRoles} from '../FolderRoles.tsx';
import {SettingsNav} from '../SettingsNav.tsx';
import {SettingsNavigationContext,createSettingsNavigation} from '../navigation.tsx';
import {clearSettings} from '../../../api/settings.ts';
import {t} from '../../../i18n';

let locked=false;
beforeEach(()=>{
 clearSettings();locked=false;localStorage.clear();
 vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({settings:{'reader.fontSize':{value:16,source:'global',locked}}}))));
});
afterEach(()=>{vi.unstubAllGlobals();clearSettings();});
const mount=()=>render(()=><RetainedSettingsSection name="reader" active accountBound={false}>{()=><ReaderAppearanceSettings/>}</RetainedSettingsSection>);
const ALL_KEYS=['colorMode','fontSize','lineHeight','contentWidth','zoom','actionStyle','attachmentsDensity','attachmentsCollapsed','loading','loadingDelayMs'];

it('shows every message display setting exactly once, as stacked cards with no extra navigation',async()=>{
 mount();
 for(const key of ALL_KEYS)expect(await screen.findAllByText(t(`reader-${key}`),{selector:'label'})).toHaveLength(1);
 expect(READER_CARDS.flatMap(c=>[...c.keys,...(c.leading?[c.leading]:[])]).sort()).toEqual([...ALL_KEYS].sort());
 expect(screen.getAllByRole('heading',{level:4})).toHaveLength(READER_CARDS.length+3);
 expect(screen.queryByRole('search')).toBeNull();
 expect(screen.queryByRole('textbox',{name:/search/i})).toBeNull();
 expect(document.querySelector('[data-kit-toggle],[data-kit-card],[data-kit-grid]')).toBeNull();
 expect(screen.queryByRole('button',{name:/collapse all/i})).toBeNull();
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
it('scrolls to and outlines the card named by the route',async()=>{
 const nav=createSettingsNavigation({section:'reader',card:'reader-card-loading'});
 const scroll=vi.fn();Element.prototype.scrollIntoView=scroll;
 render(()=><SettingsNavigationContext.Provider value={nav}><RetainedSettingsSection name="reader" active accountBound={false}>{()=><ReaderAppearanceSettings/>}</RetainedSettingsSection></SettingsNavigationContext.Provider>);
 await waitFor(()=>expect(scroll).toHaveBeenCalled());
 expect(document.getElementById('reader-card-loading')).toHaveAttribute('data-card-flash');
});
it('the section search lists matching settings rows from every section and opens the card',async()=>{
 const open=vi.fn();
 render(()=><SettingsNav current={()=>'reader'} query={()=>'forward header'} supported={()=>true} onSelect={()=>undefined} onOpenCard={open}/>);
 const hit=await screen.findByRole('button',{name:new RegExp(t('settings-quote-style'))});
 fireEvent.click(hit);
 expect(open).toHaveBeenCalledWith('compose','compose-card-addressing',hit.getAttribute('data-settings-hit'));
});
it('says nothing matches when neither a section nor a row does',()=>{
 render(()=><SettingsNav current={()=>'reader'} query={()=>'zzzzqq'} supported={()=>true} onSelect={()=>undefined} onOpenCard={()=>undefined}/>);
 expect(screen.getByRole('status')).toHaveTextContent(t('settings-search-no-sections'));
});
it('Special folders is a native collapsible block, closed until opened, and a deep link opens it',async()=>{
 const nav=createSettingsNavigation({section:'folders',card:'folders-card-roles'});
 Element.prototype.scrollIntoView=vi.fn();
 const folders=()=>[{id:'inbox',name:'Inbox',role:'inbox',mailwomanUpstreamName:'INBOX'}];
 render(()=><SettingsNavigationContext.Provider value={nav}><RetainedSettingsSection name="folders" active accountBound={false}>{()=><FolderRoles folders={folders}/>}</RetainedSettingsSection></SettingsNavigationContext.Provider>);
 const details=document.querySelector<HTMLDetailsElement>('#folders-card-roles > details')!;
 expect(details.querySelector('summary')).toHaveTextContent(t('settings-folder-role-title'));
 await waitFor(()=>expect(details.open).toBe(true));
});

it('search targets scroll to and highlight the exact setting row, and repeat on a second selection',async()=>{
 const nav=createSettingsNavigation({section:'reader',card:'reader-card-text',row:'reader-fontSize'});
 const scroll=vi.fn();Element.prototype.scrollIntoView=scroll;
 render(()=><SettingsNavigationContext.Provider value={nav}><RetainedSettingsSection name="reader" active accountBound={false}>{()=><ReaderAppearanceSettings/>}</RetainedSettingsSection></SettingsNavigationContext.Provider>);
 const control=await screen.findByLabelText(t('reader-fontSize'));
 await waitFor(()=>expect(control.closest('[data-setting-row]')).toHaveAttribute('data-card-flash'));
 expect(document.getElementById('reader-card-text')).not.toHaveAttribute('data-card-flash');
 expect(document.activeElement).toBe(control);
 const calls=scroll.mock.calls.length;
 await nav.navigate({section:'reader',card:'reader-card-text',row:'reader-fontSize'});
 await waitFor(()=>expect(scroll.mock.calls.length).toBeGreaterThan(calls));
 expect(control.closest('[data-setting-row]')).toHaveAttribute('data-card-flash');
});
