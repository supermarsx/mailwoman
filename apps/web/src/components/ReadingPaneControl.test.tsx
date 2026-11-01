import {afterEach,expect,it} from 'vitest';
import {cleanup,fireEvent,render,screen,within} from '@solidjs/testing-library';
import {ReadingPaneControl} from './ReadingPaneControl.tsx';
import {setReadingPane} from './readingPane.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';

afterEach(()=>{cleanup();suspendAccountPreferences();setReadingPane('right');});
it('shows themed icon choices and reflects the selected position',()=>{
 suspendAccountPreferences();setReadingPane('right');
 const {container}=render(()=><ReadingPaneControl/>);
 const control=screen.getByRole('button',{name:'Reading pane'});
 expect(control).toHaveAttribute('aria-haspopup','menu');expect(control.textContent).toBe('');
 fireEvent.click(control);
 const options=screen.getAllByRole('menuitemradio');expect(options).toHaveLength(3);
 expect(within(options[0]!).getByText('Right')).toBeInTheDocument();
 expect(options.every(option=>option.querySelector('[data-icon]'))).toBe(true);
 expect(screen.getByRole('menuitemradio',{name:'Right'})).toHaveAttribute('aria-checked','true');
 expect(container.querySelector('[data-icon=paneRight]')).not.toBeNull();
 fireEvent.click(screen.getByRole('menuitemradio',{name:'Bottom'}));
 expect(control).toHaveAttribute('title','Reading pane: Bottom');expect(control).toHaveFocus();expect(container.querySelector('[data-icon=paneBottom]')).not.toBeNull();
 fireEvent.click(control);fireEvent.click(screen.getByRole('menuitemradio',{name:'Off'}));
 expect(control).toHaveAttribute('aria-expanded','false');expect(container.querySelector('[data-icon=eyeOff]')).not.toBeNull();
 expect(localStorage.getItem('mw.mail.readingPane')).toBe('off');
});

it('supports keyboard navigation and dismisses on Escape or an account change',()=>{
 suspendAccountPreferences();setReadingPane('bottom');render(()=><ReadingPaneControl/>);
 const control=screen.getByRole('button',{name:'Reading pane'});
 fireEvent.keyDown(control,{key:'ArrowDown'});expect(screen.getByRole('menuitemradio',{name:'Bottom'})).toHaveFocus();
 fireEvent.keyDown(document.activeElement!,{key:'ArrowDown'});expect(screen.getByRole('menuitemradio',{name:'Off'})).toHaveFocus();
 fireEvent.keyDown(document.activeElement!,{key:'Escape'});expect(screen.queryByRole('menu')).toBeNull();expect(control).toHaveFocus();
 fireEvent.click(control);suspendAccountPreferences();expect(screen.queryByRole('menu')).toBeNull();
});
