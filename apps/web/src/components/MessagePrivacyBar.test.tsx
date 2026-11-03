import {beforeAll,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {loadCatalog} from '../i18n/index.ts';
import {MessagePrivacyBar} from './MessagePrivacyBar.tsx';
beforeAll(async()=>{await Promise.all([loadCatalog('privacy'),loadCatalog('remote-images'),loadCatalog('mail')]);});
const imageProps={emailId:'message',sender:'sender@example.org',mode:'ask' as const,report:{blockedHosts:['example.org'],blockedCount:1,trackerCount:1}};
it('combines image and link controls in one compact frame without coupling consent',async()=>{
 const grant=vi.fn(async()=>{}),links=vi.fn();render(()=><MessagePrivacyBar images={{...imageProps,onGrant:grant,trackerOverrideAllowed:true}} links={{emailId:'message',scope:'account:message',urls:[],links:['https://example.org'],linksReady:true,linksEnabled:false,onToggleLinks:links}}/>);
 expect(document.querySelectorAll('[data-notice-bar]')).toHaveLength(1);expect(screen.getByTestId('message-privacy-bar')).toBeInTheDocument();
 fireEvent.click(screen.getByRole('button',{name:'More image options'}));fireEvent.click(screen.getByRole('menuitem',{name:'Allow trackers for this message'}));await waitFor(()=>expect(grant).toHaveBeenCalledWith({kind:'trackers-single',value:'message'},expect.anything()));expect(links).not.toHaveBeenCalled();
 fireEvent.click(screen.getByTestId('message-links-toggle'));fireEvent.click(screen.getByRole('menuitem',{name:'Enable once'}));expect(links).toHaveBeenCalledOnce();expect(grant).toHaveBeenCalledOnce();
});
it('shows live pending progress without a premature failure headline and settles or resets coherently',()=>{
 const [progress,setProgress]=createSignal({loaded:4,total:21,pending:16,failed:1});render(()=><MessagePrivacyBar images={{...imageProps,imageProgress:progress()}}/>);
 expect(screen.getByRole('status')).toHaveTextContent('4/21 images shown · 16 loading · 1 failed');expect(screen.getByRole('status')).not.toHaveTextContent("couldn't load");
 setProgress({loaded:20,total:21,pending:0,failed:1});expect(screen.getByRole('status')).toHaveTextContent('20/21 images shown · 1 failed');
 setProgress({loaded:20,total:21,pending:1,failed:0});expect(screen.getByRole('status')).toHaveTextContent('20/21 images shown · 1 loading');
 setProgress({loaded:21,total:21,pending:0,failed:0});expect(screen.getByRole('status')).toHaveTextContent('21/21 images shown');
});
it('offers revocation only for explicit tracker consent and hides unavailable grants',async()=>{
 const revoke=vi.fn(async()=>{});const [allowed,setAllowed]=createSignal(true);render(()=><MessagePrivacyBar images={{...imageProps,trackerOverrideAllowed:allowed(),trackersAllowed:true,onRevoke:revoke}}/>);
 fireEvent.click(screen.getByRole('button',{name:'More image options'}));fireEvent.click(screen.getByRole('menuitem',{name:'Block trackers again'}));await waitFor(()=>expect(revoke).toHaveBeenCalledWith({kind:'trackers-single',value:'message'},expect.anything()));
 setAllowed(false);fireEvent.click(screen.getByRole('button',{name:'More image options'}));expect(screen.queryByRole('menuitem',{name:'Allow trackers for this message'})).toBeNull();expect(screen.queryByRole('menuitem',{name:'Block trackers again'})).toBeNull();
});
