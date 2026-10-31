import {render,screen,fireEvent} from '@solidjs/testing-library';
import {expect,it,vi} from 'vitest';
import {WorkspaceLayoutControls} from './WorkspaceLayoutControls.tsx';
it('offers five view arrangements in one labelled dropdown',()=>{
 const change=vi.fn();render(()=><WorkspaceLayoutControls value="stack" onChange={change}/>);
 const control=screen.getByRole('combobox',{name:'View arrangement'});
 expect(screen.getAllByRole('option')).toHaveLength(5);expect(control).toHaveValue('stack');
 fireEvent.change(control,{target:{value:'masonry-side-by-side'}});expect(change).toHaveBeenCalledWith('masonry-side-by-side');
});
