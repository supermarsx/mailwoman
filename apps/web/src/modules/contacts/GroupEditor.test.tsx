import {describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {GroupEditor} from './GroupEditor.tsx';

describe('dedicated group editor',()=>{
 it('retains a failed draft, blocks duplicate saves and confirms discard',async()=>{
  let finish:(id:string|null)=>void=()=>undefined;const save=vi.fn(()=>new Promise<string|null>(resolve=>{finish=resolve;})),done=vi.fn(),cancel=vi.fn();
  render(()=><GroupEditor books={[{id:'book',name:'Personal'}]} initialBook="book" onSave={save} onSaved={done} onCancel={cancel}/>);
  fireEvent.input(screen.getByLabelText('New group name'),{target:{value:'Friends'}});fireEvent.click(screen.getByRole('button',{name:'Create'}));expect(screen.getByRole('button',{name:'Create'})).toBeDisabled();expect(screen.getByRole('button',{name:'Cancel'})).toBeDisabled();finish(null);
  expect(await screen.findByRole('alert')).toHaveTextContent('could not be saved');expect(screen.getByLabelText('New group name')).toHaveValue('Friends');expect(done).not.toHaveBeenCalled();expect(save).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole('button',{name:'Cancel'}));expect(await screen.findByRole('alertdialog')).toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'Keep editing'}));expect(cancel).not.toHaveBeenCalled();
 });
});
