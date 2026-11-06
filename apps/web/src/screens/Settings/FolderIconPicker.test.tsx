import {expect,it,vi} from 'vitest';
import {fireEvent,render,screen} from '@solidjs/testing-library';
import {FOLDER_ICONS} from '../../api/accountPreferences.ts';
import {FolderIconPicker} from './FolderIconPicker.tsx';
const mount=(onChange=vi.fn(),value:never|''='' as never)=>render(()=><FolderIconPicker label="Icon for X" value={value} automaticIcon="folder" onChange={onChange}/>);
it('lists every icon once under named groups, each with an accessible name',()=>{
 mount();
 const radios=screen.getAllByRole('radio');expect(radios).toHaveLength(FOLDER_ICONS.length+1);
 for(const r of radios)expect(r.getAttribute('aria-label')).toBeTruthy();
 expect(screen.getAllByRole('group').map(g=>g.getAttribute('aria-label'))).toEqual(['General','Mail','Work','Finance and shopping','Travel and home','Personal and fun','Tech and tools']);
});
it('every new id is selectable',()=>{
 const onChange=vi.fn();mount(onChange);
 for(const id of ['camera','music','film','lightbulb','rocket','trophy','leaf','globe','cloud','bookmark','wallet','medical','bell','key','receipt','cart','package','wrench','chart','team','store','office','laptop','crown','family']){
  fireEvent.click(document.querySelector(`[data-icon-id="${id}"]`)!);expect(onChange).toHaveBeenLastCalledWith(id);
 }
});
it('filters by name, shows an empty state, and keeps one tab stop',()=>{
 mount();
 const search=screen.getByRole('searchbox',{name:'Search icons'});
 fireEvent.input(search,{target:{value:'troph'}});
 expect(screen.getAllByRole('radio').map(r=>r.getAttribute('aria-label'))).toEqual(['Trophy']);
 expect(screen.getAllByRole('radio')[0]).toHaveAttribute('tabindex','0');
 fireEvent.input(search,{target:{value:'zzzz'}});expect(screen.queryAllByRole('radio')).toHaveLength(0);expect(screen.getByRole('status')).toHaveTextContent('No icons match');
});
it('arrow keys move across groups',()=>{
 const onChange=vi.fn();mount(onChange);
 const first=screen.getAllByRole('radio')[0]!;first.focus();fireEvent.keyDown(first,{key:'ArrowRight'});
 expect(onChange).toHaveBeenLastCalledWith('folder');expect(document.activeElement).toBe(screen.getAllByRole('radio')[1]);
});
