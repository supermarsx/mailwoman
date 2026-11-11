import {beforeAll, afterEach, expect, it, vi} from 'vitest';
import {cleanup, fireEvent, render, screen, waitFor, within} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {SettingsNav} from './SettingsNav.tsx';
import {SETTINGS_CATEGORIES} from './settingsCategories.ts';
import {createSettingsNavigation} from './navigation.tsx';
import {loadCatalog,t} from '../../i18n';

beforeAll(()=>loadCatalog('settings'));
afterEach(cleanup);
const entries=SETTINGS_CATEGORIES.flatMap(category=>category.sections);
function fixture(initial='appearance',supportedIds?:string[]){
 const [current,setCurrent]=createSignal(initial),[query,setQuery]=createSignal(''),[supported,setSupported]=createSignal(supportedIds);
 const select=vi.fn((section:string)=>{setCurrent(section);});
 render(()=><SettingsNav current={current} query={query} supported={section=>supported()?.includes(section)??true} onSelect={select}/>);
 return {current,setCurrent,setQuery,setSupported,select};
}
function trigger():HTMLButtonElement{return screen.getByRole('button',{name:'Settings sections'}) as HTMLButtonElement;}
function open():HTMLElement{fireEvent.click(trigger());return screen.getByRole('menu',{name:'Settings sections'});}

it('makes every supported category section reachable in the compact picker and marks the current section',()=>{
 fixture('notifications');const menu=open();
 expect(within(menu).getAllByRole('menuitemradio')).toHaveLength(entries.length);
 for(const entry of entries)expect(within(menu).getByRole('menuitemradio',{name:t(entry.label)})).toHaveAttribute('aria-checked',String(entry.id==='notifications'));
 fireEvent.click(within(menu).getByRole('menuitemradio',{name:t('settings-nav-metadata')}));
 expect(screen.queryByRole('menu')).toBeNull();
 expect(trigger()).toHaveTextContent(t('settings-nav-metadata'));
 expect(within(open()).getByRole('menuitemradio',{name:t('settings-nav-metadata')})).toHaveAttribute('aria-checked','true');
});

it('routes the final category through the caller guard and retains the previous current marker when navigation is refused',async()=>{
 const navigation=createSettingsNavigation({section:'signatures',editor:{kind:'edit',id:'draft-signature'}});
 const guard=vi.fn(()=>false),removeGuard=navigation.beforeLeave(guard);
 let pendingNavigation=Promise.resolve(false);
 const select=vi.fn((section:string)=>{pendingNavigation=navigation.navigate({section});});
 render(()=><SettingsNav current={()=>navigation.route().section} query={()=>''} supported={()=>true} onSelect={select}/>);
 fireEvent.click(within(open()).getByRole('menuitemradio',{name:t('settings-nav-metadata')}));
 await expect(pendingNavigation).resolves.toBe(false);expect(guard).toHaveBeenCalledOnce();expect(select).toHaveBeenLastCalledWith('metadata');expect(navigation.route().editor?.id).toBe('draft-signature');
 expect(within(open()).getByRole('menuitemradio',{name:t('settings-sig-title')})).toHaveAttribute('aria-checked','true');
 fireEvent.keyDown(document.activeElement!,{key:'Escape'});removeGuard();
 fireEvent.click(within(open()).getByRole('menuitemradio',{name:t('settings-nav-metadata')}));
 await waitFor(()=>expect(navigation.route().section).toBe('metadata'));expect(navigation.route().editor).toBeUndefined();
});

it('filters compact sections by search terms and support without offering an unreachable empty picker',()=>{
 const {setQuery,setSupported}=fixture('appearance',['appearance','time','metadata']);
 expect(within(open()).getAllByRole('menuitemradio')).toHaveLength(3);
 expect(screen.queryByRole('menuitemradio',{name:t('settings-notif-title')})).toBeNull();
 fireEvent.keyDown(document.activeElement!,{key:'Escape'});
 setQuery('IANA');const matched=open();expect(within(matched).getAllByRole('menuitemradio')).toHaveLength(1);expect(within(matched).getByRole('menuitemradio',{name:t('settings-nav-regional')})).toBeVisible();
 fireEvent.keyDown(document.activeElement!,{key:'Escape'});
 setSupported(['appearance','metadata']);expect(trigger()).toBeDisabled();expect(screen.getAllByText('No matching settings sections.').length).toBeGreaterThan(0);
 setQuery('no-such-setting');expect(trigger()).toBeDisabled();expect(screen.queryByRole('menu')).toBeNull();
 setQuery('server metadata');expect(trigger()).toBeEnabled();expect(within(open()).getAllByRole('menuitemradio')).toHaveLength(1);
});

it('opens from the keyboard at the current section and restores trigger focus on Escape',()=>{
 const {select}=fixture('metadata');const button=trigger();button.focus();fireEvent.keyDown(button,{key:'ArrowDown'});
 const menu=screen.getByRole('menu',{name:'Settings sections'}),current=within(menu).getByRole('menuitemradio',{name:t('settings-nav-metadata')});
 expect(current).toHaveFocus();expect(current).toHaveAttribute('aria-checked','true');
 fireEvent.keyDown(current,{key:'Home'});expect(within(menu).getAllByRole('menuitemradio')[0]).toHaveFocus();
 fireEvent.keyDown(document.activeElement!,{key:'End'});expect(current).toHaveFocus();
 fireEvent.keyDown(current,{key:'Escape'});expect(screen.queryByRole('menu')).toBeNull();expect(button).toHaveFocus();expect(select).not.toHaveBeenCalled();
});

it('retains desktop arrow wrapping and Home/End navigation across category headings',()=>{
 fixture('appearance');const nav=screen.getByRole('navigation',{name:'Settings sections'});
 const buttons=Array.from(nav.querySelectorAll<HTMLButtonElement>('button[data-settings-entry]'));expect(buttons).toHaveLength(entries.length);
 buttons[0]!.focus();fireEvent.keyDown(buttons[0]!,{key:'ArrowUp'});expect(buttons.at(-1)).toHaveFocus();
 fireEvent.keyDown(document.activeElement!,{key:'ArrowDown'});expect(buttons[0]).toHaveFocus();
 fireEvent.keyDown(document.activeElement!,{key:'End'});expect(buttons.at(-1)).toHaveFocus();
 fireEvent.keyDown(document.activeElement!,{key:'Home'});expect(buttons[0]).toHaveFocus();
});
