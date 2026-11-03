import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {BodyModeGroup,type BodyMode} from './EditorRibbon.tsx';
afterEach(cleanup);
it('uses themed icon choices instead of a native message-type select',()=>{
 const change=vi.fn();render(()=>{const [mode,setMode]=createSignal<BodyMode>('html');return <BodyModeGroup label="Message type" value={mode} onChange={next=>{setMode(next);change(next);}}/>;});
 const trigger=screen.getByRole('button',{name:'Message type'});expect(trigger).toHaveClass('btn');expect(trigger).toHaveTextContent('HTML');expect(trigger).toHaveAttribute('value','html');expect(screen.queryByRole('combobox')).toBeNull();
 fireEvent.click(trigger);const html=screen.getByRole('menuitemradio',{name:'HTML'});expect(html).toHaveAttribute('aria-checked','true');expect(html.querySelector('[data-icon="fileCode"]')).not.toBeNull();
 const rich=screen.getByRole('menuitemradio',{name:'Rich text'});expect(rich.querySelector('[data-icon="paint"]')).not.toBeNull();fireEvent.click(rich);expect(change).toHaveBeenCalledExactlyOnceWith('rich');expect(trigger).toHaveTextContent('Rich text');expect(trigger).toHaveFocus();expect(screen.queryByRole('menu')).toBeNull();
 fireEvent.keyDown(trigger,{key:'ArrowDown'});expect(screen.getByRole('menuitemradio',{name:'Rich text'})).toHaveFocus();fireEvent.keyDown(document.activeElement!,{key:'Home'});const plain=screen.getByRole('menuitemradio',{name:'Plain text'});expect(plain).toHaveFocus();fireEvent.click(plain);expect(trigger).toHaveAttribute('value','plain');expect(trigger).toHaveFocus();
});
