import {expect,it,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {SearchableSelect} from './SearchableSelect.tsx';
it('filters options and chooses with the keyboard without submitting the surrounding form',()=>{
 const submit=vi.fn(),[value,setValue]=createSignal('');
 render(()=><form onSubmit={submit}><SearchableSelect label="Zone" value={value()} onChange={setValue} options={[{value:'Europe/London',label:'Europe/London'},{value:'America/New_York',label:'America/New York'}]}/></form>);
 const input=screen.getByRole('combobox');fireEvent.input(input,{target:{value:'new york'}});
 expect(screen.getAllByRole('option')).toHaveLength(1);
 fireEvent.keyDown(input,{key:'Enter'});
 expect(value()).toBe('America/New_York');expect(submit).not.toHaveBeenCalled();expect(screen.queryByRole('listbox')).toBeNull();
});
