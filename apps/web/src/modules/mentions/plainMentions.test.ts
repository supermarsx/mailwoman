import {afterEach,expect,it,vi} from 'vitest';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {attachPlainMentions} from './plainMentions.ts';
import {cardForAddress} from '../contacts/share.ts';
import type {ContactSuggestion} from '../contacts/autocomplete.ts';
afterEach(()=>document.body.replaceChildren());
function setup(lookup?:(q:string)=>Promise<ContactSuggestion[]>){const area=document.createElement('textarea');document.body.append(area);area.focus();const pick=vi.fn(),change=vi.fn();const destroy=attachPlainMentions(area,{cards:()=>[cardForAddress('jane@example.org','Jane Doe')],onPick:pick,...(lookup?{lookup}:{})},change);return {area,pick,change,destroy};}
it('keyboard selection inserts a portable mention and calls recipient insertion only after explicit selection',()=>{
 const {area,pick,change,destroy}=setup();try{area.value='Hi @Ja';area.setSelectionRange(6,6);fireEvent.input(area);expect(screen.getByRole('option')).toHaveTextContent('Jane Doe');expect(pick).not.toHaveBeenCalled();fireEvent.keyDown(area,{key:'Enter'});expect(area.value).toBe('Hi @jane@example.org ');expect(change).toHaveBeenCalledWith(area.value);expect(pick).toHaveBeenCalledTimes(1);expect(screen.queryByRole('listbox')).toBeNull();}finally{destroy();}
});
it('IME, Escape and stale async lookups never insert or reopen dismissed suggestions',async()=>{
 let release!:(value:ContactSuggestion[])=>void;const pending=new Promise<ContactSuggestion[]>(resolve=>{release=resolve;});const {area,pick,destroy}=setup(()=>pending);
 try{fireEvent.compositionStart(area);area.value='@Ja';area.setSelectionRange(3,3);fireEvent.input(area);expect(screen.queryByRole('listbox')).toBeNull();fireEvent.compositionEnd(area);expect(screen.getByRole('listbox')).toBeVisible();fireEvent.keyDown(area,{key:'Escape'});release([{cardId:'remote',name:'Jack',email:'jack@example.org',display:'Jack',score:1,isFavorite:false}]);await waitFor(()=>expect(screen.queryByRole('listbox')).toBeNull());expect(pick).not.toHaveBeenCalled();}finally{destroy();}
});
