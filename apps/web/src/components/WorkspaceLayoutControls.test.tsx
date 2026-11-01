import {render,screen,fireEvent} from '@solidjs/testing-library';
import {expect,it,vi} from 'vitest';
import {WorkspaceLayoutControls} from './WorkspaceLayoutControls.tsx';
import {ReadingPaneControl} from './ReadingPaneControl.tsx';
it('offers five view arrangements in one labelled dropdown',()=>{
 const change=vi.fn();render(()=><WorkspaceLayoutControls value="stack" onChange={change}/>);
 const control=screen.getByRole('button',{name:'View arrangement'});
 expect(control.textContent).toBe('');fireEvent.click(control);
 const choices=screen.getAllByRole('menuitemradio');expect(choices).toHaveLength(5);
 expect(choices.every(choice=>choice.querySelector('[data-icon]'))).toBe(true);
 expect(screen.getByRole('menuitemradio',{name:'Stacked'})).toHaveAttribute('aria-checked','true');
 fireEvent.click(screen.getByRole('menuitemradio',{name:'Masonry side by side'}));expect(change).toHaveBeenCalledWith('masonry-side-by-side');
 expect(screen.queryByRole('menu')).toBeNull();expect(control).toHaveFocus();
});
it('toggles the popup, closes other dropdowns and dismisses outside',()=>{
 render(()=><><WorkspaceLayoutControls value="tabs" onChange={()=>{}}/><ReadingPaneControl/></>);
 const arrangement=screen.getByRole('button',{name:'View arrangement'}),pane=screen.getByRole('button',{name:'Reading pane'});
 fireEvent.click(arrangement);expect(screen.getByRole('menu',{name:'View arrangement'})).toBeInTheDocument();
 fireEvent.pointerDown(arrangement);fireEvent.click(arrangement);expect(screen.queryByRole('menu')).toBeNull();
 fireEvent.click(arrangement);fireEvent.click(pane);expect(screen.getAllByRole('menu')).toHaveLength(1);
 expect(arrangement).toHaveAttribute('aria-expanded','false');expect(pane).toHaveAttribute('aria-expanded','true');
 fireEvent.pointerDown(document.body);expect(screen.queryByRole('menu')).toBeNull();expect(pane).toHaveAttribute('aria-expanded','false');
});
