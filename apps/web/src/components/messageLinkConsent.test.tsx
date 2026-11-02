import {it,expect} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {createMessageLinkConsent} from './messageLinkConsent.ts';
it('clears consent on message, account and policy changes without restoring old grants',()=>{
 const [scope,setScope]=createSignal('account-a:message-a:full');
 render(()=>{const consent=createMessageLinkConsent(scope);return <button aria-pressed={consent.enabled()} onClick={consent.toggle}>Links</button>;});
 const button=screen.getByRole('button');expect(button).toHaveAttribute('aria-pressed','false');
 for(const next of ['account-a:message-b:full','account-b:message-b:full','account-b:message-b:no-media']){
  fireEvent.click(button);expect(button).toHaveAttribute('aria-pressed','true');
  const previous=scope();setScope(next);expect(button).toHaveAttribute('aria-pressed','false');
  setScope(previous);expect(button).toHaveAttribute('aria-pressed','false');setScope(next);
 }
 fireEvent.click(button);fireEvent.click(button);expect(button).toHaveAttribute('aria-pressed','false');
});

it('refuses early consent until initial policy and body are ready, then preserves the grant',()=>{
 const [scope,setScope]=createSignal('temporary-policy');const [ready,setReady]=createSignal(false);
 render(()=>{const consent=createMessageLinkConsent(scope,ready);return <button aria-pressed={consent.enabled()} onClick={consent.toggle}>Links</button>;});
 const button=screen.getByRole('button');fireEvent.click(button);expect(button).toHaveAttribute('aria-pressed','false');
 setScope('resolved-policy');setReady(true);fireEvent.click(button);expect(button).toHaveAttribute('aria-pressed','true');
 setReady(false);setReady(true);expect(button).toHaveAttribute('aria-pressed','true');
});

it('keeps consent through read-status updates that do not change the scope value',()=>{
 const [message,setMessage]=createSignal({id:'same-message',seen:false});
 render(()=>{const consent=createMessageLinkConsent(()=>`account:${message().id}:policy`);return <button aria-pressed={consent.enabled()} onClick={consent.toggle}>Links</button>;});
 const button=screen.getByRole('button');fireEvent.click(button);expect(button).toHaveAttribute('aria-pressed','true');
 setMessage({id:'same-message',seen:true});expect(button).toHaveAttribute('aria-pressed','true');
 setMessage({id:'other-message',seen:true});expect(button).toHaveAttribute('aria-pressed','false');
});

it('temporarily makes granted links inert during body refresh without forgetting consent',()=>{
 const [ready,setReady]=createSignal(true);
 render(()=>{const consent=createMessageLinkConsent(()=>'account:message:policy',ready);return <button aria-pressed={consent.enabled()} onClick={consent.toggle}>Links</button>;});
 const button=screen.getByRole('button');fireEvent.click(button);expect(button).toHaveAttribute('aria-pressed','true');
 setReady(false);expect(button).toHaveAttribute('aria-pressed','false');setReady(true);expect(button).toHaveAttribute('aria-pressed','true');
});

it('allows exact-sender trust only when ready and preserves a manual disable for the current message',()=>{
 const [scope,setScope]=createSignal('account:message:policy'),[ready,setReady]=createSignal(false),[trusted,setTrusted]=createSignal(true);
 render(()=>{const consent=createMessageLinkConsent(scope,ready,trusted);return <button aria-pressed={consent.enabled()} onClick={consent.toggle}>Links</button>;});
 const button=screen.getByRole('button');expect(button).toHaveAttribute('aria-pressed','false');setReady(true);expect(button).toHaveAttribute('aria-pressed','true');
 fireEvent.click(button);expect(button).toHaveAttribute('aria-pressed','false');setReady(false);setReady(true);setTrusted(false);setTrusted(true);expect(button).toHaveAttribute('aria-pressed','false');
 setScope('account:other-message:policy');expect(button).toHaveAttribute('aria-pressed','true');
 setTrusted(false);expect(button).toHaveAttribute('aria-pressed','false');
});

it('does not revive one-time consent when returning to an old account or authentication generation',()=>{
 const [scope,setScope]=createSignal('account:generation1:message');
 render(()=>{const consent=createMessageLinkConsent(scope);return <button aria-pressed={consent.enabled()} onClick={consent.toggle}>Links</button>;});
 const button=screen.getByRole('button');fireEvent.click(button);expect(button).toHaveAttribute('aria-pressed','true');
 setScope('account:generation2:message');expect(button).toHaveAttribute('aria-pressed','false');setScope('account:generation1:message');expect(button).toHaveAttribute('aria-pressed','false');
});
