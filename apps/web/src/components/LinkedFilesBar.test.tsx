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
 setReady(true);fireEvent.click(screen.getByRole('button',{name:'Enable links'}));expect(toggled).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('menuitem',{name:'Enable once'}));expect(toggled).toHaveBeenCalledTimes(1);expect(screen.getByRole('button',{name:'Disable links'})).toBeEnabled();
 setReady(false);expect(screen.getByRole('button',{name:'Disable links'})).toBeEnabled();
});

it('offers sender trust and revoke only through the compact menu and respects administrator locks',()=>{
 const [trusted,setTrusted]=createSignal(false),[editable,setEditable]=createSignal(true),[scope,setScope]=createSignal('account:message');
 const trust=vi.fn(()=>{setTrusted(true);}),revoke=vi.fn(()=>{setTrusted(false);});
 render(()=><LinkedFilesBar emailId="message" scope={scope()} urls={[]} links={['https://example.org']} linksReady linksEnabled={trusted()} senderTrusted={trusted()} senderTrustEditable={editable()} onToggleLinks={()=>{}} onTrustSender={trust} onRevokeSender={revoke}/>);
 expect(screen.queryByRole('button',{name:'Always allow links from this sender'})).toBeNull();
 fireEvent.click(screen.getByTestId('message-links-toggle'));fireEvent.click(screen.getByRole('menuitem',{name:'Always allow links from this sender'}));expect(trust).toHaveBeenCalledOnce();
 setEditable(false);fireEvent.click(screen.getByTestId('message-links-toggle'));const locked=screen.getByRole('menuitem',{name:'Stop always allowing links from this sender'});expect(locked).toHaveAttribute('aria-disabled','true');fireEvent.click(locked);expect(revoke).not.toHaveBeenCalled();
 setEditable(true);fireEvent.click(screen.getByTestId('message-links-toggle'));setScope('account:other-message');expect(screen.queryByRole('menu')).toBeNull();
 fireEvent.click(screen.getByTestId('message-links-toggle'));fireEvent.click(screen.getByRole('menuitem',{name:'Stop always allowing links from this sender'}));expect(revoke).toHaveBeenCalledOnce();
});
