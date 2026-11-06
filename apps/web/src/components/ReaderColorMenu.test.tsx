import {beforeEach,describe,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {ReaderColorMenu,colorActions} from './ReaderColorMenu.tsx';
import {READER_COLOR_CHOICES,messageColorOverride,resetMessageColorOverrides,setMessageColorOverride,type ReaderColorChoice} from '../viewers/readerColorMode.ts';
import {READER_CARDS} from '../screens/Settings/ReaderAppearance.tsx';

function Harness(props:{setting?:ReaderColorChoice;locked?:boolean;keyOf?:string;makeDefault?:(c:ReaderColorChoice)=>void}){
 const key=()=>props.keyOf??'a:m1';
 return <ReaderColorMenu scope={key()} setting={props.setting??'auto'} locked={props.locked===true} value={messageColorOverride(key())??props.setting??'auto'} onChange={c=>setMessageColorOverride(key(),c)} onMakeDefault={props.makeDefault??(()=>{})}/>;
}
const items=():HTMLElement[]=>Array.from(screen.getByRole('menu').querySelectorAll<HTMLElement>('[role^="menuitem"]'));
const noop=()=>{};

describe('reader per-message color menu',()=>{
 beforeEach(()=>{resetMessageColorOverrides();});
 it('lists exactly the modes the settings offer, with the current one checked',async()=>{
  expect(READER_CARDS[0]!.keys).toContain('colorMode');
  expect([...READER_COLOR_CHOICES]).toEqual(['auto','original','adaptive','invert']);
  render(()=><Harness/>);
  const button=screen.getByRole('button',{name:/^Message colors: /});
  expect(button).toHaveAttribute('aria-haspopup','menu');
  fireEvent.click(button);
  const list=items();
  await waitFor(()=>expect(items().map(i=>i.textContent).slice(0,4)).toEqual(['Automatic with theme','Original message colors','Adapt to theme','Invert message colors']));
  expect(list.map(i=>i.getAttribute('aria-checked')).slice(0,4)).toEqual(['true','false','false','false']);
  expect(list.map(i=>i.getAttribute('role')).slice(0,4)).toEqual(Array(4).fill('menuitemradio'));
  expect(document.activeElement).toBe(list[0]);
 });
 it('choosing applies to this message only and reset returns to the default',async()=>{
  render(()=><Harness setting="original"/>);
  fireEvent.click(screen.getByRole('button',{name:/^Message colors/}));
  await waitFor(()=>expect(items().some(i=>i.textContent==='Invert message colors')).toBe(true));
  fireEvent.click(items().find(i=>i.textContent==='Invert message colors')!);
  expect(messageColorOverride('a:m1')).toBe('invert');expect(messageColorOverride('a:m2')).toBeUndefined();
  expect(screen.getByRole('button',{name:/^Message colors/})).toHaveAttribute('data-mode','invert');
  fireEvent.click(screen.getByRole('button',{name:/^Message colors/}));
  const reset=items().find(i=>/^Use my default/.test(i.textContent??''))!;
  expect(reset.textContent).toContain('Original');
  fireEvent.click(reset);
  expect(messageColorOverride('a:m1')).toBeUndefined();
 });
 it('an override survives switching to another message and back',()=>{
  const [id,setId]=createSignal('a:m1');
  render(()=><Harness keyOf={id()} setting="original"/>);
  setMessageColorOverride('a:m1','adaptive');
  expect(screen.getByRole('button',{name:/^Message colors/})).toHaveAttribute('data-mode','adaptive');
  setId('a:m2');expect(screen.getByRole('button',{name:/^Message colors/})).toHaveAttribute('data-mode','original');
  setId('a:m1');expect(screen.getByRole('button',{name:/^Message colors/})).toHaveAttribute('data-mode','adaptive');
 });
 it('make-default hands the current choice over; reset and make-default are off when nothing differs',()=>{
  const make=vi.fn();
  setMessageColorOverride('a:m1','invert');
  render(()=><Harness setting="auto" makeDefault={make}/>);
  fireEvent.click(screen.getByRole('button',{name:/^Message colors/}));
  fireEvent.click(items().find(i=>i.textContent==='Make this my default')!);
  expect(make).toHaveBeenCalledWith('invert');
  expect(colorActions({value:'auto',setting:'auto',locked:false,onChange:noop,onMakeDefault:noop}).filter(a=>a.disabled).map(a=>a.id)).toEqual(['color:reset','color:make-default']);
 });
 it('an admin lock disables the button with the lock tooltip and no menu opens',()=>{
  render(()=><Harness locked/>);
  const button=screen.getByRole('button',{name:/^Message colors/});
  expect(button).toHaveAttribute('aria-disabled','true');expect(button.title).toBe('Locked by your administrator');
  fireEvent.click(button);expect(screen.queryByRole('menu')).toBeNull();
  expect(colorActions({value:'auto',setting:'auto',locked:true,onChange:noop,onMakeDefault:noop}).every(a=>a.disabled)).toBe(true);
 });
 it('Escape closes and returns focus to the button',()=>{
  render(()=><Harness/>);
  const button=screen.getByRole('button',{name:/^Message colors/});fireEvent.click(button);
  fireEvent.keyDown(screen.getByRole('menu'),{key:'Escape'});
  expect(screen.queryByRole('menu')).toBeNull();expect(document.activeElement).toBe(button);
 });
});
