import {expect,it} from 'vitest';
import {fireEvent,render,screen} from '@solidjs/testing-library';
import {SettingsSubtabs} from './SettingsSubtabs.tsx';
import {createSettingsNavigation,SettingsNavigationContext} from './navigation.tsx';

it('preserves unfinished edits and opens a hidden panel for a settings search result',async()=>{
 const navigation=createSettingsNavigation({section:'compose'});
 render(()=><SettingsNavigationContext.Provider value={navigation}><SettingsSubtabs section="compose" label="settings-compose-title" tabs={[
  {id:'addressing',label:'settings-compose-card-addressing',cards:['compose-card-addressing'],content:<p>Addressing options</p>},
  {id:'checks',label:'settings-compose-card-checks',cards:['compose-card-checks'],content:<textarea aria-label="Unfinished domains"/>},
 ]}/></SettingsNavigationContext.Provider>);
 const tabs=screen.getAllByRole('tab');
 fireEvent.click(tabs[1]!);fireEvent.input(screen.getByLabelText('Unfinished domains'),{target:{value:'example.test'}});
 fireEvent.click(tabs[0]!);expect(screen.getByLabelText('Unfinished domains')).not.toBeVisible();
 await navigation.navigate({section:'compose',card:'compose-card-checks',row:'trusted-domains'});
 expect(tabs[1]).toHaveAttribute('aria-selected','true');
 expect(screen.getByLabelText('Unfinished domains')).toBeVisible();expect(screen.getByLabelText('Unfinished domains')).toHaveValue('example.test');
 fireEvent.keyDown(tabs[1]!,{key:'Home'});expect(tabs[0]).toHaveFocus();
 fireEvent.keyDown(tabs[0]!,{key:'ArrowLeft'});expect(tabs[1]).toHaveFocus();
 await navigation.navigate({section:'reader',card:'compose-card-addressing'});expect(tabs[1]).toHaveAttribute('aria-selected','true');
});
