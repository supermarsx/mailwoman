import { beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { KeyboardShortcuts } from './KeyboardShortcuts.tsx';
import { createSettingsNavigation } from './navigation.tsx';
import type { SettingsMap } from '../../api/settings.ts';

const data = vi.hoisted(() => ({ values: {} as Record<string, {value:unknown;source:'user';locked:boolean}>, save: vi.fn<(key:string,value:unknown)=>Promise<boolean>>() }));
vi.mock('../../api/settings.ts', () => ({ settings: () => data.values }));
vi.mock('./useSetting.ts', () => ({ useSetting: (key:string, fallback:unknown) => ({ value:()=>data.values[key]?.value??fallback,entry:()=>data.values[key],locked:()=>data.values[key]?.locked===true,saving:()=>false,error:()=>'',set:(value:unknown)=>data.save(key,value) }) }));
beforeEach(() => {
  localStorage.clear();
  data.values={'shortcuts.enabled':{value:true,source:'user',locked:false},'shortcuts.compose':{value:'inherit',source:'user',locked:false}} satisfies SettingsMap;
  data.save.mockReset().mockImplementation(async(key,value)=>{data.values[key]={value,source:'user',locked:false};return true;});
});

it('opens a dedicated editor and saves through the account settings authority',async()=>{
  const navigation=createSettingsNavigation({section:'shortcuts'});
  render(()=><KeyboardShortcuts navigation={navigation}/>);
  fireEvent.click(screen.getByRole('button',{name:'Edit Compose shortcut'}));
  const input=await screen.findByLabelText('Binding');
  expect(screen.queryByRole('table')).toBeNull();
  fireEvent.input(input,{target:{value:'Ctrl+Shift+K'}});
  fireEvent.click(screen.getByRole('button',{name:'Save shortcut'}));
  await waitFor(()=>expect(data.save).toHaveBeenCalledWith('shortcuts.compose','Ctrl+Shift+K'));
  await waitFor(()=>expect(screen.getByRole('table')).toBeVisible());
});
it('refuses a conflict with an inherited action and respects admin locks',async()=>{
  const navigation=createSettingsNavigation({section:'shortcuts',editor:{kind:'edit',id:'compose'}});
  render(()=><KeyboardShortcuts navigation={navigation}/>);
  fireEvent.input(screen.getByLabelText('Binding'),{target:{value:'r'}});
  fireEvent.click(screen.getByRole('button',{name:'Save shortcut'}));
  expect(screen.getByRole('alert')).toHaveTextContent('same shortcut');expect(data.save).not.toHaveBeenCalled();
});
it('shows a locked binding without offering a writable control',()=>{
  data.values['shortcuts.compose']!.locked=true;
  const navigation=createSettingsNavigation({section:'shortcuts',editor:{kind:'edit',id:'compose'}});
  render(()=><KeyboardShortcuts navigation={navigation}/>);
  expect(screen.getByLabelText('Binding')).toBeDisabled();expect(screen.getByRole('button',{name:'Save shortcut'})).toBeDisabled();
});
