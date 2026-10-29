import {describe,expect,it,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {ReaderViewMode} from './ReaderViewMode.tsx';
import type {SecurityMode} from '../viewers/max-security.ts';

describe('reader view-mode dropdown',()=>{
 it('is one button naming the current mode that opens a menu of the three modes',()=>{
  const [mode,setMode]=createSignal<SecurityMode>('full-sanitized');
  render(()=><ReaderViewMode value={mode()} floor={null} onChange={setMode} scope="a:m1"/>);
  const button=screen.getByRole('button',{name:'View: Full'});
  expect(button).toHaveAttribute('aria-haspopup','menu');expect(button).toHaveAttribute('aria-expanded','false');expect(button.title).toMatch(/^View: Full\./);
  fireEvent.click(button);
  const menu=screen.getByRole('menu',{name:'View mode'});expect(button).toHaveAttribute('aria-expanded','true');
  const items=Array.from(menu.querySelectorAll<HTMLElement>('[role="menuitemcheckbox"]'));
  expect(items.map(item=>item.textContent)).toEqual(['Full','No media','Plain text']);
  expect(items.map(item=>item.getAttribute('aria-checked'))).toEqual(['true','false','false']);
  expect(items[0]!.querySelectorAll('svg')).toHaveLength(2);// mode icon + the check on the current one
  expect(document.activeElement).toBe(items[0]);
  fireEvent.keyDown(menu,{key:'ArrowDown'});expect(document.activeElement).toBe(items[1]);
  fireEvent.click(items[2]!);
  expect(mode()).toBe('plain-text');expect(screen.queryByRole('menu')).toBeNull();expect(document.activeElement).toBe(button);
  expect(screen.getByRole('button',{name:'View: Plain text'})).toHaveAttribute('data-mode','plain-text');
 });
 it('Escape closes and returns focus; modes below the admin floor cannot be chosen',()=>{
  const change=vi.fn();
  render(()=><ReaderViewMode value="plain-text" floor="plain-text" onChange={change} scope="a:m1"/>);
  const button=screen.getByRole('button',{name:'View: Plain text'});fireEvent.click(button);
  const full=screen.getByRole('menuitemcheckbox',{name:'Full'});expect(full).toHaveAttribute('aria-disabled','true');expect(screen.getByRole('menuitemcheckbox',{name:'No media'})).toHaveAttribute('aria-disabled','true');
  fireEvent.click(full);expect(change).not.toHaveBeenCalled();
  fireEvent.click(button);fireEvent.keyDown(screen.getByRole('menu'),{key:'Escape'});expect(screen.queryByRole('menu')).toBeNull();expect(document.activeElement).toBe(button);
  // A click right after a keyboard close opens it again (no time-based guard).
  fireEvent.click(button);expect(screen.getByRole('menu')).toBeVisible();
  // Pressing the button while open closes it; the click that follows does not reopen it.
  fireEvent.pointerDown(button);fireEvent.click(button,{detail:1});expect(screen.queryByRole('menu')).toBeNull();
 });
});
