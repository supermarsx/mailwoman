import type {WorkspaceMailbox} from '../../api/workspaceMail.ts';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {SettingsTransfer} from './SettingsTransfer.tsx';
import {defaultAppearance} from '../../theme/appearance.ts';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {clearSettings,loadSettings,type SettingEntry} from '../../api/settings.ts';
import {t} from '../../i18n';
import type {SettingsBackup} from './settingsTransferModel.ts';
const fake=vi.hoisted(()=>({targets:[] as WorkspaceMailbox[],source:vi.fn(),ui:{keyboardPreset:'vim'},saveUi:vi.fn(),appearance:vi.fn(),loadAppearance:vi.fn(),saveAppearance:vi.fn()}));
vi.mock('../../state/context.ts',()=>({useApp:()=>({appearancePrefs:fake.appearance,setAppearancePrefs:vi.fn(),linkedMailboxId:()=>null})}));
vi.mock('../../api/workspaceMail.ts',()=>({workspaceMailboxes:async()=>fake.targets}));
vi.mock('./settingsTransferSource.ts',()=>({readSettingsSource:fake.source}));
vi.mock('../../api/accountPreferences.ts',()=>({accountUiPreferences:()=>fake.ui,accountUiPreferenceStatus:()=> 'synced',importAccountUiPreferences:fake.saveUi}));
vi.mock('../../api/prefs.ts',()=>({appearanceSync:()=>null,createAppearancePrefsApi:()=>({load:fake.loadAppearance,save:fake.saveAppearance})}));
let entries:Record<string,SettingEntry>,writes:string[],reject:boolean;
const json=(x:unknown,status=200)=>new Response(JSON.stringify(x),{status,headers:{'content-type':'application/json'}});
const fixture:SettingsBackup={format:'mailwoman-user-settings',version:1,createdAt:'2026-11-10T00:00:00Z',source:{accountId:'a',origin:location.origin},items:[
 {kind:'setting',key:'reader.fontSize',value:18},
 {kind:'setting',key:'reader.zoom',value:125},
 {kind:'setting',key:'unknown.future',value:true},
]};
beforeEach(async()=>{
 clearSettings();activateAccountPreferences('a');fake.targets=[];fake.source.mockReset();reject=false;writes=[];fake.appearance.mockReturnValue(defaultAppearance());fake.saveUi.mockReset();fake.saveAppearance.mockReset();fake.loadAppearance.mockReset();
 entries={'reader.fontSize':{value:14,source:'user',locked:false,userValue:14},'reader.zoom':{value:100,source:'global',locked:true}};
 await loadSettings({fetcher:async(_url,init)=>{
  if(init?.method==='PUT'){writes.push(_url);if(reject)return json({code:'invalidValue'},422);const value=JSON.parse(String(init.body)).value;const key=decodeURIComponent(_url.split('/').at(-1)!);entries[key]={value,source:'user',locked:false,userValue:value};return json(entries[key]);}
  return json({settings:entries});
 }});
 vi.spyOn(HTMLAnchorElement.prototype,'click').mockImplementation(()=>undefined);
 Object.defineProperty(URL,'createObjectURL',{configurable:true,value:vi.fn(()=> 'blob:export')});
 Object.defineProperty(URL,'revokeObjectURL',{configurable:true,value:vi.fn()});
});
afterEach(()=>{cleanup();suspendAccountPreferences();clearSettings();vi.restoreAllMocks();});
async function choose(backup:SettingsBackup=fixture){
 fireEvent.click(screen.getByRole('button',{name:t('st-start-import')}));
 const file=Object.assign(new File(['preferences'],'setup.json',{type:'application/json'}),{text:async()=>JSON.stringify(backup)});
 fireEvent.change(screen.getByLabelText(t('st-file-label')),{target:{files:[file]}});
 await screen.findByText('setup.json');
}
it('guides export through categories and review before creating a download',async()=>{
 render(()=><SettingsTransfer/>);
 fireEvent.click(screen.getByRole('button',{name:t('st-start-export')}));
 expect(URL.createObjectURL).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:t('st-review-export')}));
 expect(screen.getByText('Theme, colours, font, and layout')).toBeVisible();
 expect(URL.createObjectURL).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:/Download preferences/}));
 expect(URL.createObjectURL).toHaveBeenCalledOnce();expect(screen.getByRole('status')).toHaveTextContent(t('st-export-done'));expect(writes).toEqual([]);
});
it('previews locks and unsupported settings, then applies only the selected eligible change',async()=>{
 render(()=><SettingsTransfer/>);await choose();
 expect(writes).toEqual([]);
 expect(screen.getByText(t('st-status-locked'))).toBeVisible();expect(screen.getByText(t('st-status-unknown'))).toBeVisible();
 expect(screen.getByRole('checkbox',{name:'Message zoom'})).toBeDisabled();
 const button=screen.getByRole('button',{name:t('st-apply',{count:1})});fireEvent.click(button);
 await waitFor(()=>expect(screen.getByRole('status')).toHaveTextContent(t('st-results',{count:1,failed:0})));
 expect(writes).toEqual(['/api/settings/reader.fontSize']);expect(entries['reader.fontSize']?.value).toBe(18);
});
it('shows refused changes as failures and allows reviewing the remaining selection',async()=>{
 reject=true;render(()=><SettingsTransfer/>);await choose();fireEvent.click(screen.getByRole('button',{name:t('st-apply',{count:1})}));
 await waitFor(()=>expect(screen.getByRole('status')).toHaveTextContent(t('st-results',{count:0,failed:1})));
 expect(screen.getByRole('alert')).toHaveTextContent(t('settings-policy-error-invalid'));
 fireEvent.click(screen.getByRole('button',{name:t('st-review-again')}));
 expect(screen.getByRole('button',{name:t('st-apply',{count:1})})).toBeEnabled();
});
it('drops the preview when switching accounts without applying any imported preferences',async()=>{
 render(()=><SettingsTransfer/>);await choose();activateAccountPreferences('b');
 expect(screen.queryByText('setup.json')).toBeNull();expect(writes).toEqual([]);expect(fake.saveUi).not.toHaveBeenCalled();
});
it('rejects malformed files without entering review',async()=>{
 render(()=><SettingsTransfer/>);fireEvent.click(screen.getByRole('button',{name:t('st-start-import')}));
 const file=Object.assign(new File(['bad'],'bad.json'),{text:async()=> 'bad'});
 fireEvent.change(screen.getByLabelText(t('st-file-label')),{target:{files:[file]}});
 expect(await screen.findByRole('alert')).toHaveTextContent(t('st-file-invalid'));expect(writes).toEqual([]);
});
it('honors unselecting an otherwise eligible change',async()=>{
 render(()=><SettingsTransfer/>);await choose();
 const preview=screen.getByRole('checkbox',{name:'Message font size'});fireEvent.click(preview);
 expect(screen.getByRole('button',{name:t('st-apply',{count:0})})).toBeDisabled();expect(writes).toEqual([]);
});

it('reviews a verified source account and only writes selected preferences after Apply',async()=>{
 fake.targets=[{owner:preferenceAccount()!,linkId:'work',accountId:'b',storageAuthority:'work-authority',label:'Work',current:false}];fake.source.mockResolvedValue({...fixture,source:{accountId:'b',storageAuthority:'work-authority',origin:location.origin}});
 render(()=><SettingsTransfer/>);fireEvent.click(screen.getByRole('button',{name:t('st-start-import')}));
 const source=await screen.findByRole('combobox',{name:t('st-source-account')});fireEvent.change(source,{target:{value:'work'}});fireEvent.click(screen.getByRole('button',{name:t('st-source-review')}));
 await screen.findByText('Work');expect(writes).toEqual([]);expect(fake.source).toHaveBeenCalledWith(fake.targets[0],expect.any(AbortSignal));
 expect(screen.getByText(t('st-status-locked'))).toBeVisible();fireEvent.click(screen.getByRole('button',{name:t('st-apply',{count:1})}));
 await waitFor(()=>expect(writes).toEqual(['/api/settings/reader.fontSize']));
});
