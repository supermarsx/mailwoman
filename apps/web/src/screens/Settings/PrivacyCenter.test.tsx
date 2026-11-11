import {expect,it,vi} from 'vitest';
import {fireEvent,render,screen} from '@solidjs/testing-library';
import {PrivacyCenter} from './PrivacyCenter.tsx';
import {createSettingsNavigation,SettingsNavigationContext} from './navigation.tsx';
vi.mock('../../i18n',()=>({loadCatalog:async()=>{},t:(key:string)=>key}));

it('preserves form entries and opens the requested tab for a settings search result',async()=>{
 const navigation=createSettingsNavigation({section:'privacy'});
 render(()=><SettingsNavigationContext.Provider value={navigation}><PrivacyCenter content={<p>Message privacy</p>} consent={<input aria-label="Consent draft"/>} diagnostics={<p>Diagnostics</p>} data={<p>Your data</p>} requests={<p>Requests</p>}/></SettingsNavigationContext.Provider>);
 fireEvent.click(screen.getByRole('tab',{name:'privacy-tab-consent'}));
 fireEvent.input(screen.getByLabelText('Consent draft'),{target:{value:'keep this'}});
 fireEvent.click(screen.getByRole('tab',{name:'privacy-tab-content'}));
 expect(screen.getByLabelText('Consent draft')).not.toBeVisible();
 await navigation.navigate({section:'privacy',card:'privacy-card-analytics',row:'analytics-consent'});
 expect(screen.getByRole('tab',{name:'privacy-tab-consent'})).toHaveAttribute('aria-selected','true');
 expect(screen.getByLabelText('Consent draft')).toBeVisible();
 expect(screen.getByLabelText('Consent draft')).toHaveValue('keep this');
 await navigation.navigate({section:'privacy',card:'your-data-requests'});
 expect(screen.getByRole('tab',{name:'privacy-tab-requests'})).toHaveAttribute('aria-selected','true');
 fireEvent.keyDown(screen.getByRole('tab',{name:'privacy-tab-requests'}),{key:'Home'});
 expect(screen.getByRole('tab',{name:'privacy-tab-content'})).toHaveFocus();
});
