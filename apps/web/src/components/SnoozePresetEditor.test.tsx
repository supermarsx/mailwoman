import {expect,it} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {SnoozePresetEditor} from './SnoozePresetEditor.tsx';
import {DEFAULT_SNOOZE_PRESETS,type SnoozePreset} from '../state/snooze.ts';
it('edits, hides, reorders and creates presets without writing before the parent saves',()=>{
 const [rows,setRows]=createSignal<SnoozePreset[]>(DEFAULT_SNOOZE_PRESETS.map(p=>({...p})));
 render(()=><SnoozePresetEditor value={rows()} onChange={setRows}/>);
 fireEvent.click(screen.getByRole('button',{name:'In 2 hours'}));
 fireEvent.input(screen.getByLabelText('Name (optional)'),{target:{value:'After my meeting'}});
 fireEvent.input(screen.getByLabelText('Amount'),{target:{value:'6'}});
 expect(rows().find(p=>p.id==='2-hours')).toMatchObject({label:'After my meeting',amount:6});
 fireEvent.click(screen.getByRole('checkbox',{name:'Show in menu: After my meeting'}));expect(rows().find(p=>p.id==='2-hours')?.enabled).toBe(false);
 fireEvent.click(screen.getByRole('button',{name:'Move up: After my meeting'}));expect(rows()[2]?.label).toBe('After my meeting');
 fireEvent.click(screen.getByRole('button',{name:'Add preset'}));
 fireEvent.change(screen.getByLabelText('Schedule rule'),{target:{value:'end-month'}});
 expect(rows().at(-1)).toMatchObject({kind:'end-month',time:'09:00'});expect(rows().at(-1)).not.toHaveProperty('amount');
});
