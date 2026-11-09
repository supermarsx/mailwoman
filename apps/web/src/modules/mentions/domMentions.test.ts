import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import {fireEvent,screen} from '@solidjs/testing-library';
import {attachDomMentions} from './domMentions.ts';
import {cardForAddress} from '../contacts/share.ts';
beforeAll(()=>{Object.defineProperty(Range.prototype,'getBoundingClientRect',{configurable:true,value:()=>({left:10,right:20,top:10,bottom:30,width:10,height:20})});});
afterEach(()=>document.body.replaceChildren());
it('replaces the selected native visual-editor token with portable markup, moves caret and emits through native history',()=>{
 const editor=document.createElement('div');editor.setAttribute('contenteditable','true');editor.tabIndex=0;editor.innerHTML='<p>Hello <b>@Ja</b></p>';document.body.append(editor);editor.focus();const text=editor.querySelector('b')!.firstChild!;const range=document.createRange();range.setStart(text,3);range.collapse(true);document.getSelection()!.removeAllRanges();document.getSelection()!.addRange(range);
 const exec=vi.fn(()=>{throw new TypeError('Trusted Types string sink');});Object.defineProperty(document,'execCommand',{configurable:true,value:exec});
 const pick=vi.fn(),emit=vi.fn(),dispose=attachDomMentions(editor,{cards:()=>[cardForAddress('jane@example.org','Jane Doe')],onPick:pick},emit);
 try{fireEvent.input(editor);expect(pick).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('option'));expect(editor.querySelector('a')?.getAttribute('href')).toBe('mailto:jane%40example.org');expect(editor.textContent).toBe('Hello @Jane Doe ');expect(pick).toHaveBeenCalledTimes(1);expect(emit).toHaveBeenCalledTimes(1);expect(exec).not.toHaveBeenCalled();expect(document.getSelection()?.isCollapsed).toBe(true);expect(screen.queryByRole('listbox')).toBeNull();}finally{dispose();}
});
it('does not activate while the visual-editor selection is in a quoted original',()=>{
 const editor=document.createElement('div');editor.setAttribute('contenteditable','true');editor.innerHTML='<blockquote><p>@Ja</p></blockquote>';document.body.append(editor);const range=document.createRange();range.setStart(editor.querySelector('p')!.firstChild!,3);range.collapse(true);document.getSelection()!.removeAllRanges();document.getSelection()!.addRange(range);const dispose=attachDomMentions(editor,{cards:()=>[cardForAddress('jane@example.org','Jane Doe')],onPick:vi.fn()},vi.fn());try{fireEvent.input(editor);expect(screen.queryByRole('listbox')).toBeNull();}finally{dispose();}
});
