import {expect,it} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {ValueInput} from './ValueInput.tsx';
import {parseDraft,type Draft} from './value.ts';
import {DEFAULT_SNOOZE_PRESETS} from '../../../state/snooze.ts';
it('edits administrator snooze defaults with the friendly preset editor and validates before save',()=>{
 const [draft,setDraft]=createSignal<Draft>(JSON.stringify(DEFAULT_SNOOZE_PRESETS));
 render(()=><ValueInput kind={{type:'snoozePresets'}} draft={draft()} onDraft={setDraft} wiring={{id:'snooze-policy',describedBy:undefined,invalid:false}}/>);
 fireEvent.click(screen.getByRole('button',{name:'In 2 hours'}));fireEvent.input(screen.getByLabelText('Amount'),{target:{value:'12'}});
 expect(parseDraft({type:'snoozePresets'},draft(),false).ok).toBe(true);
 expect(JSON.parse(String(draft()))).toEqual(expect.arrayContaining([expect.objectContaining({id:'2-hours',amount:12})]));
 expect(parseDraft({type:'snoozePresets'},'[{"id":"a","kind":"hours","amount":-1,"enabled":true}]',false).ok).toBe(false);
});
