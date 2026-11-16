import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {ThemedSelects} from './ThemedSelects.tsx';
import {installPointerEvent} from './accounts/pointerEventPolyfill.test-util.ts';
installPointerEvent();
afterEach(cleanup);
it('uses the themed menu and fires the real form change for a pointer selection',async()=>{
 const change=vi.fn();render(()=><><ThemedSelects/><label>Choice<select onChange={change}><option value="one">One</option><option value="two">Two</option></select></label></>);
 const select=screen.getByRole('combobox',{name:'Choice'});fireEvent.pointerDown(select,{button:0});
 const other=await screen.findByRole('menuitemradio',{name:'Two'});expect(other.querySelectorAll('svg')).toHaveLength(0);expect(screen.getByRole('menuitemradio',{name:'One'}).querySelectorAll('svg')).toHaveLength(1);expect(screen.queryByRole('menuitemcheckbox')).toBeNull();
 fireEvent.click(other);
 expect(select).toHaveValue('two');expect(change).toHaveBeenCalledTimes(1);
 await waitFor(()=>expect(screen.queryByRole('menu')).not.toBeInTheDocument());
});
it('opens by keyboard and keeps fieldset-disabled choices unavailable',async()=>{
 render(()=><><ThemedSelects/><label>Enabled<select><option>One</option><option disabled>Two</option></select></label><fieldset disabled><label>Locked<select><option>Three</option></select></label></fieldset></>);
 fireEvent.keyDown(screen.getByRole('combobox',{name:'Enabled'}),{key:'ArrowDown'});
 expect(await screen.findByRole('menuitemradio',{name:'Two'})).toHaveAttribute('aria-disabled','true');
 fireEvent.keyDown(screen.getByRole('menuitemradio',{name:'One'}),{key:'Escape'});
 fireEvent.pointerDown(screen.getByRole('combobox',{name:'Locked'}),{button:0});
 expect(screen.queryByRole('menu')).not.toBeInTheDocument();
});
