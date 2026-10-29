import {expect,it,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal,Show} from 'solid-js';
import {createLinkReview} from './MessageLinks.tsx';
import type {MessageLink} from '../viewers/messageLinks.ts';
import type {ComposeInitial} from './compose/reply.ts';
/** The trigger now lives in the reader action row (ReaderToolbar.tsx); this harness stands in for it. */
function MessageLinks(props:{links:MessageLink[];truncated:boolean;scope:string;active:boolean;onCompose?:(draft:ComposeInitial)=>void}){const r=createLinkReview({links:()=>props.links,truncated:()=>props.truncated,scope:()=>props.scope,active:()=>props.active,...(props.onCompose?{onCompose:props.onCompose}:{})});return <><Show when={r.count()}><button type="button" onClick={()=>r.open()}>{`Review links (${r.count()})`}</button></Show>{r.dialog()}</>;}
import {inspectMessageLink} from '../viewers/messageLinks.ts';
it('reveals a no-referrer destination only after review and closes on account/message replacement',()=>{
 const [message,setMessage]=createSignal({scope:'a:one',read:false});const [links,setLinks]=createSignal([inspectMessageLink('https://destination.example','https://bank.example')!]);render(()=><MessageLinks links={links()} truncated={false} scope={message().scope} active/>);
 expect(screen.queryByRole('link')).toBeNull();fireEvent.click(screen.getByRole('button',{name:'Review links (1)'}));expect(screen.queryByRole('link')).toBeNull();fireEvent.click(screen.getByRole('button',{name:'https://bank.example'}));
 const open=screen.getByRole('link',{name:'Open in a new tab'});expect(open.getAttribute('href')).toBe('https://destination.example/');expect(open.getAttribute('rel')).toBe('noopener noreferrer');expect(open.getAttribute('referrerpolicy')).toBe('no-referrer');expect(screen.getByText(/different host/)).toBeVisible();setMessage({...message(),read:true});expect(screen.getByRole('dialog')).toBeVisible();setLinks(links().map(link=>({...link})));expect(screen.getByRole('dialog')).toBeVisible();setLinks([]);expect(screen.getByRole('dialog')).toBeVisible();expect(open.getAttribute('href')).toBe('https://destination.example/');setMessage({...message(),scope:'b:two'});expect(screen.queryByRole('dialog')).toBeNull();
});
it('mailto creates only a reviewable escaped draft after explicit confirmation',()=>{
 const compose=vi.fn();const links=[inspectMessageLink('mailto:user@example.org?body=%3Cscript%3Ehello%3C%2Fscript%3E','Write')!];render(()=><MessageLinks links={links} truncated={false} scope="a" active onCompose={compose}/>);fireEvent.click(screen.getByRole('button',{name:'Review links (1)'}));fireEvent.click(screen.getByRole('button',{name:'Write'}));expect(compose).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Review a new message'}));expect(compose).toHaveBeenCalledWith(expect.objectContaining({to:'user@example.org',mode:'new',cc:'',bodyHtml:'<p>&lt;script&gt;hello&lt;/script&gt;</p>'}));expect(screen.queryByRole('dialog')).toBeNull();
});
