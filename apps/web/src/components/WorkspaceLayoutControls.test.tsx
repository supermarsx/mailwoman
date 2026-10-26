import {render,screen,fireEvent} from '@solidjs/testing-library';
import {expect,it,vi} from 'vitest';
import {WorkspaceLayoutControls} from './WorkspaceLayoutControls.tsx';

it('offers five icon-only labelled controls with explanatory themed tooltips',()=>{
  const change=vi.fn();render(()=><WorkspaceLayoutControls value="stack" onChange={change}/>);
  const controls=screen.getAllByRole('button');expect(controls).toHaveLength(5);
  for(const control of controls){expect(control.textContent).toBe('');expect(control.querySelector('svg')).not.toBeNull();expect(control.getAttribute('aria-label')).toBeTruthy();expect(control.getAttribute('data-tooltip')).toBeTruthy();}
  expect(controls.filter(control=>control.getAttribute('aria-pressed')==='true')).toHaveLength(1);
  fireEvent.click(controls[3]!);expect(change).toHaveBeenCalledWith('masonry-side-by-side');
});
