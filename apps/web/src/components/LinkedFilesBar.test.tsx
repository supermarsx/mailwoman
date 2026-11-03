import {beforeAll,it,expect,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {loadCatalog} from '../i18n/index.ts';
import {LinkedFilesBar} from './LinkedFilesBar.tsx';
beforeAll(()=>loadCatalog('privacy'));
it.each([false,true])('exposes direct busy disabled consent until readiness, then permits it (compact=%s)',compact=>{
 const [ready,setReady]=createSignal(false);const [enabled,setEnabled]=createSignal(false);const toggled=vi.fn(()=>setEnabled(value=>!value));
 render(()=><LinkedFilesBar compact={compact} emailId="message" scope="account:message" urls={[]} links={['https://example.org']} linksReady={ready()} linksEnabled={enabled()} onToggleLinks={toggled}/>);
 const pending=screen.getByRole('button',{name:'Preparing links…'});expect(pending).toBeDisabled();expect(pending).toHaveAttribute('aria-busy','true');pending.click();expect(toggled).not.toHaveBeenCalled();
 setReady(true);const enable=screen.getByRole('button',{name:'Enable once'});expect(enable.querySelector('span')).toHaveTextContent('Enable once');expect(enable).not.toHaveAttribute('aria-haspopup');fireEvent.click(enable);expect(toggled).toHaveBeenCalledTimes(1);expect(screen.queryByRole('button',{name:'Enable once'})).toBeNull();
 setReady(false);if(compact){fireEvent.click(screen.getByTestId('message-links-toggle'));expect(screen.getByRole('menuitem',{name:'Disable links'})).not.toHaveAttribute('aria-disabled','true');}else expect(screen.getByRole('button',{name:'Disable links'})).toBeEnabled();
});

it('prevents direct consent during a sender-trust mutation without coupling the downloads menu',()=>{
 const [pending,setPending]=createSignal(true);const toggled=vi.fn();render(()=><LinkedFilesBar compact emailId="message" scope="account:message" urls={[]} links={['https://example.org']} linksReady senderTrustPending={pending()} onToggleLinks={toggled}/>);
 const enable=screen.getByRole('button',{name:'Enable once'});expect(enable).toBeDisabled();expect(enable).toHaveAttribute('aria-busy','true');fireEvent.click(enable);expect(toggled).not.toHaveBeenCalled();expect(screen.getByRole('button',{name:'Link options'})).toBeEnabled();setPending(false);fireEvent.click(enable);expect(toggled).toHaveBeenCalledOnce();
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
