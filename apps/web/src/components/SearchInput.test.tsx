import {expect,it,vi} from 'vitest';
import {fireEvent,render,screen} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {SearchInput} from './SearchInput.tsx';
it('clears the controlled query through real input events and keeps focus',()=>{
 const changed=vi.fn();const [value,setValue]=createSignal('report');render(()=><SearchInput aria-label="Search documents" value={value()} onInput={event=>{changed(event.currentTarget.value);setValue(event.currentTarget.value);}}/>);
 fireEvent.click(screen.getByRole('button',{name:'Clear: Search documents'}));expect(value()).toBe('');expect(changed).toHaveBeenCalledWith('');expect(screen.getByRole('searchbox')).toHaveFocus();expect(screen.queryByRole('button')).not.toBeInTheDocument();
});
