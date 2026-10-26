import {expect,it,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {ReaderAction} from './ReaderAction.tsx';
const state=vi.hoisted(()=>({style:():string=> 'icons'}));
vi.mock('../api/settings.ts',()=>({settings:()=>({'reader.actionStyle':{value:state.style()}})}));
it('keeps descriptive accessible actions in compact mode and reacts to label policy',()=>{
 const [style,setStyle]=createSignal('icons');state.style=style;const action=vi.fn();render(()=><ReaderAction icon="archive" label="Archive" help="Move to your archive folder" onClick={action}/>);
 const button=screen.getByRole('button',{name:'Archive'});expect(button.title).toBe('Move to your archive folder');expect(button.className).toContain('reader-action--compact');fireEvent.click(button);expect(action).toHaveBeenCalledTimes(1);setStyle('labels');expect(button.className).not.toContain('reader-action--compact');expect(button.querySelector('span')?.className).toBe('');
});
it('preserves pressed and disabled state without dispatching unavailable actions',()=>{const action=vi.fn();render(()=><ReaderAction icon="pin" label="Unpin" help="Remove pin" pressed disabled onClick={action}/>);const button=screen.getByRole('button',{name:'Unpin'});expect(button.getAttribute('aria-pressed')).toBe('true');fireEvent.click(button);expect(action).not.toHaveBeenCalled();});
