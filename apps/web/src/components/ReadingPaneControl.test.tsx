import {afterEach,expect,it} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {ReadingPaneControl} from './ReadingPaneControl.tsx';
import {setReadingPane} from './readingPane.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';

afterEach(()=>{cleanup();suspendAccountPreferences();setReadingPane('right');});
it('uses an icon-only native dropdown and reflects the selected position',()=>{
 suspendAccountPreferences();setReadingPane('right');
 const {container}=render(()=><ReadingPaneControl/>);
 const control=screen.getByRole('combobox',{name:'Reading pane'});
 expect(container.querySelector('[data-icon=paneRight]')).not.toBeNull();
 fireEvent.change(control,{target:{value:'bottom'}});
 expect(control).toHaveValue('bottom');expect(container.querySelector('[data-icon=paneBottom]')).not.toBeNull();
 fireEvent.change(control,{target:{value:'off'}});
 expect(control).toHaveValue('off');expect(container.querySelector('[data-icon=eyeOff]')).not.toBeNull();
 expect(localStorage.getItem('mw.mail.readingPane')).toBe('off');
});
