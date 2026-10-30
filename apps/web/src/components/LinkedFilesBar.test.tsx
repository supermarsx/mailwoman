import {beforeAll,it,expect,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {loadCatalog} from '../i18n/index.ts';
import {LinkedFilesBar} from './LinkedFilesBar.tsx';
beforeAll(()=>loadCatalog('privacy'));
it('exposes busy disabled consent until policy and body readiness, then permits the action',()=>{
 const [ready,setReady]=createSignal(false);const [enabled,setEnabled]=createSignal(false);const toggled=vi.fn(()=>setEnabled(value=>!value));
 render(()=><LinkedFilesBar emailId="message" scope="account:message" urls={[]} links={['https://example.org']} linksReady={ready()} linksEnabled={enabled()} onToggleLinks={toggled}/>);
 const pending=screen.getByRole('button',{name:'Preparing links…'});expect(pending).toBeDisabled();expect(pending).toHaveAttribute('aria-busy','true');pending.click();expect(toggled).not.toHaveBeenCalled();
 setReady(true);fireEvent.click(screen.getByRole('button',{name:'Enable links'}));expect(toggled).toHaveBeenCalledTimes(1);expect(screen.getByRole('button',{name:'Disable links'})).toBeEnabled();
 setReady(false);expect(screen.getByRole('button',{name:'Disable links'})).toBeEnabled();
});
