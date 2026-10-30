import {afterEach,expect,it} from 'vitest';
import {createSignal} from 'solid-js';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {TemplateLibraryEditor} from './TemplateLibraryEditor.tsx';
import type {ResponseTemplate} from './responseTemplateModel.ts';
afterEach(cleanup);
it('adds, edits, searches and removes plain text without interpreting HTML',()=>{
 const [rows,setRows]=createSignal<ResponseTemplate[]>([]);render(()=><TemplateLibraryEditor value={rows()} onChange={setRows}/>);
 fireEvent.click(screen.getByRole('button',{name:'Add template'}));
 fireEvent.input(screen.getByLabelText('Template name'),{target:{value:'Follow up'}});
 fireEvent.input(screen.getByLabelText('Message text'),{target:{value:'<img src=x>\nThanks'}});
 expect(rows()[0]?.body).toBe('<img src=x>\nThanks');expect(document.querySelector('img')).toBeNull();
 fireEvent.input(screen.getByLabelText('Find a template'),{target:{value:'Thanks'}});expect(screen.getByRole('button',{name:'Follow up'})).toBeVisible();
 fireEvent.click(screen.getByRole('button',{name:'Remove template'}));expect(rows()).toEqual([]);
});
it('keeps a locked library readable but disables mutations',()=>{render(()=><TemplateLibraryEditor value={[{id:'one',title:'Read only',body:'Original'}]} onChange={()=>{throw Error('write');}} disabled/>);expect(screen.getByRole('button',{name:'Add template'})).toBeDisabled();fireEvent.click(screen.getByRole('button',{name:'Read only'}));expect(screen.getByLabelText('Message text')).toBeDisabled();expect(screen.getByRole('button',{name:'Remove template'})).toBeDisabled();});
