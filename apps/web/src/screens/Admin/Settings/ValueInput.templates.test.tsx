import {afterEach,expect,it} from 'vitest';
import {createSignal} from 'solid-js';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {ValueInput} from './ValueInput.tsx';
import type {Draft} from './value.ts';
afterEach(cleanup);
it('uses the same friendly library editor for administrator policy rather than raw JSON',()=>{const [draft,setDraft]=createSignal<Draft>(JSON.stringify([{id:'shared',title:'Shared',body:'Original'}]));render(()=><ValueInput kind={{type:'messageTemplates'}} draft={draft()} onDraft={setDraft} wiring={{id:'shared-library',describedBy:undefined,invalid:false}}/>);fireEvent.click(screen.getByRole('button',{name:'Shared'}));fireEvent.input(screen.getByLabelText('Message text'),{target:{value:'Literal <script> text'}});expect(JSON.parse(String(draft()))).toEqual([{id:'shared',title:'Shared',body:'Literal <script> text'}]);expect(document.querySelector('script')).toBeNull();});
