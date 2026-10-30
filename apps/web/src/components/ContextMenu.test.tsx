import {afterEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {clampMenu,createContextMenu} from './ContextMenu.tsx';
afterEach(cleanup);
describe('context menu',()=>{
 it('opens without invoking the target, navigates enabled actions and restores focus',()=>{
  const run=vi.fn(),select=vi.fn();render(()=>{const menu=createContextMenu({label:()=> 'Actions',scope:()=> 'a:1',actions:()=>[{id:'a',label:'Open',icon:'mail',run},{id:'b',label:'Unavailable',icon:'delete',disabled:true,run},{id:'c',label:'Pin',icon:'pin',run}]});return <><button onClick={select} onContextMenu={menu.open} onKeyDown={menu.keydown}>Target</button>{menu.menu()}</>;});
  const target=screen.getByRole('button',{name:'Target'});fireEvent.contextMenu(target,{clientX:100,clientY:200});expect(select).not.toHaveBeenCalled();expect(screen.getByRole('menuitem',{name:'Open'})).toHaveFocus();fireEvent.keyDown(document.activeElement!,{key:'ArrowDown'});expect(screen.getByRole('menuitem',{name:'Pin'})).toHaveFocus();fireEvent.keyDown(document.activeElement!,{key:'Escape'});expect(screen.queryByRole('menu')).toBeNull();expect(target).toHaveFocus();expect(run).not.toHaveBeenCalled();
  fireEvent.keyDown(target,{key:'F10',shiftKey:true});fireEvent.click(screen.getByRole('menuitem',{name:'Open'}));expect(run).toHaveBeenCalledOnce();
 });
 it('closes when authority changes and rechecks action availability',()=>{
  const [scope,setScope]=createSignal('a'),[disabled,setDisabled]=createSignal(false);const run=vi.fn();render(()=>{const menu=createContextMenu({label:()=> 'Actions',scope,actions:()=>[{id:'a',label:'Delete',icon:'delete',disabled:disabled(),run}]});return <><button onContextMenu={menu.open}>Target</button>{menu.menu()}</>;});
  fireEvent.contextMenu(screen.getByText('Target'));setDisabled(true);fireEvent.click(screen.getByRole('menuitem'));expect(run).not.toHaveBeenCalled();setScope('b');expect(screen.queryByRole('menu')).toBeNull();
 });
 it('preserves native editable and selected-text menus',()=>{
  render(()=>{const menu=createContextMenu({label:()=> 'Actions',scope:()=> 'a',actions:()=>[{id:'a',label:'Open',icon:'mail',run:()=>{}}]});return <><div onContextMenu={menu.open}><input aria-label="Text"/><span>Selectable</span></div>{menu.menu()}</>;});
  expect(fireEvent.contextMenu(screen.getByRole('textbox'))).toBe(true);expect(screen.queryByRole('menu')).toBeNull();const selection=window.getSelection()!,range=document.createRange();range.selectNodeContents(screen.getByText('Selectable'));selection.removeAllRanges();selection.addRange(range);expect(fireEvent.contextMenu(screen.getByText('Selectable'))).toBe(true);expect(screen.queryByRole('menu')).toBeNull();selection.removeAllRanges();
 });
 it('exposes an on/off action as a menuitemcheckbox with its state, plain actions as menuitems',()=>{
  const [on,setOn]=createSignal(false);render(()=>{const menu=createContextMenu({label:()=> 'Actions',scope:()=> 'a',actions:()=>[{id:'a',label:'Open',icon:'mail',run:()=>{}},{id:'tag',label:'Label: Work',icon:on()?'check':'tag',checked:on(),run:()=>{}}]});return <><button onContextMenu={menu.open}>Target</button>{menu.menu()}</>;});
  fireEvent.contextMenu(screen.getByText('Target'));expect(screen.getByRole('menuitem',{name:'Open'})).not.toHaveAttribute('aria-checked');const work=screen.getByRole('menuitemcheckbox',{name:'Label: Work'});expect(work).toHaveAttribute('aria-checked','false');setOn(true);expect(work).toHaveAttribute('aria-checked','true');fireEvent.keyDown(screen.getByRole('menuitem',{name:'Open'}),{key:'ArrowDown'});expect(work).toHaveFocus();
 });
 it('clamps menus at narrow viewport edges',()=>{expect(clampMenu(410,790,240,300,420,800)).toEqual({left:172,top:492});expect(clampMenu(-20,-5,900,1000,320,600)).toEqual({left:8,top:8});});
});

it('opens a second level by click or arrow, returns focus with Left/Escape, and executes only its child',()=>{
 const run=vi.fn();render(()=>{const menu=createContextMenu({label:()=> 'Actions',scope:()=> 'a',actions:()=>[{id:'search',label:'Search',icon:'search',run:()=>{},children:[{id:'disabled',label:'Unavailable',icon:'search',disabled:true,run},{id:'domain',label:'Domain',icon:'domains',run}]}]});return <><button onContextMenu={menu.open}>Target</button>{menu.menu()}</>;});
 const target=screen.getByText('Target');fireEvent.contextMenu(target);
 const parent=screen.getByRole('menuitem',{name:'Search'});expect(parent).toHaveAttribute('aria-haspopup','menu');expect(parent).toHaveAttribute('aria-expanded','false');
 fireEvent.keyDown(parent,{key:'ArrowRight'});expect(screen.getByRole('menuitem',{name:'Domain'})).toHaveFocus();expect(parent).toHaveAttribute('aria-expanded','true');
 fireEvent.keyDown(document.activeElement!,{key:'ArrowLeft'});expect(parent).toHaveFocus();expect(screen.queryByRole('menuitem',{name:'Domain'})).toBeNull();
 fireEvent.click(parent);fireEvent.keyDown(document.activeElement!,{key:'Escape'});expect(parent).toHaveFocus();expect(screen.getAllByRole('menu')).toHaveLength(1);
 fireEvent.click(parent);fireEvent.click(screen.getByRole('menuitem',{name:'Domain'}));expect(run).toHaveBeenCalledOnce();expect(screen.queryByRole('menu')).toBeNull();expect(target).toHaveFocus();
});
it('invalidates an open child when its parent becomes disabled',()=>{
 const [disabled,setDisabled]=createSignal(false);const run=vi.fn();render(()=>{const menu=createContextMenu({label:()=> 'Actions',scope:()=> 'a',actions:()=>[{id:'search',label:'Search',icon:'search',disabled:disabled(),run:()=>{},children:[{id:'domain',label:'Domain',icon:'domains',run}]}]});return <><button onContextMenu={menu.open}>Target</button>{menu.menu()}</>;});
 fireEvent.contextMenu(screen.getByText('Target'));fireEvent.click(screen.getByRole('menuitem',{name:'Search'}));setDisabled(true);expect(screen.queryByRole('menuitem',{name:'Domain'})).toBeNull();expect(run).not.toHaveBeenCalled();
});
