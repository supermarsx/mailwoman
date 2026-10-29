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
// t39 R7: trusted link domains.
import {clearSettings,loadSettings,type Fetcher} from '../api/settings.ts';
import {afterEach,describe} from 'vitest';
import {waitFor} from '@solidjs/testing-library';
const jsonResponse=(status:number,body:unknown)=>new Response(JSON.stringify(body),{status,headers:{'content-type':'application/json'}});
async function seed(entries:Record<string,unknown>,puts:unknown[]=[]):Promise<void>{await loadSettings({fetcher:(async(_url,init)=>{if(init?.method==='PUT'){const body=JSON.parse(String(init.body)) as {value:unknown};puts.push(body.value);return jsonResponse(200,{value:body.value,source:'user',locked:false,userValue:body.value});}return jsonResponse(200,{settings:entries});}) as Fetcher});}
describe('trusted link domains',()=>{
 afterEach(()=>clearSettings());
 const links=()=>['https://docs.partner.test/a','https://partner.test.evil.example/','https://evilpartner.test/','http://partner.test/'].map(href=>inspectMessageLink(href,href)!);
 it('opens a trusted destination from the list and keeps lookalikes and plain HTTP under review',async()=>{
  await seed({'links.trustedDomains':{value:['partner.test'],source:'global',locked:true},'links.trustedSubdomains':{value:true,source:'global',locked:false}});
  render(()=><MessageLinks links={links()} truncated={false} scope="a" active/>);fireEvent.click(screen.getByRole('button',{name:'Review links (4)'}));
  const direct=screen.getByRole('link',{name:'https://docs.partner.test/a'});expect(direct.getAttribute('href')).toBe('https://docs.partner.test/a');expect(direct.getAttribute('rel')).toBe('noopener noreferrer');expect(direct.getAttribute('referrerpolicy')).toBe('no-referrer');
  expect(screen.getAllByRole('link')).toHaveLength(1);expect(screen.getAllByText('Trusted domain: opens without review')).toHaveLength(1);
  for(const name of ['https://partner.test.evil.example/','https://evilpartner.test/','http://partner.test/'])expect(screen.getByRole('button',{name})).toBeVisible();
  fireEvent.click(screen.getByRole('button',{name:'https://evilpartner.test/'}));expect(screen.queryByRole('button',{name:/^Trust /})).toBeNull();
 });
 it('without subdomains only the exact domain is trusted',async()=>{
  await seed({'links.trustedDomains':{value:['partner.test'],source:'user',locked:false},'links.trustedSubdomains':{value:false,source:'default',locked:false}});
  render(()=><MessageLinks links={[inspectMessageLink('https://partner.test/x','x')!,inspectMessageLink('https://docs.partner.test/a','a')!]} truncated={false} scope="a" active/>);fireEvent.click(screen.getByRole('button',{name:'Review links (2)'}));
  expect(screen.getByRole('link',{name:'x'})).toBeVisible();expect(screen.getByRole('button',{name:'a'})).toBeVisible();
 });
 it('trusts an exact host from the destination step, keeping the administrator default list',async()=>{
  const puts:unknown[]=[];await seed({'links.trustedDomains':{value:['corp.test'],source:'global',locked:false},'links.trustedSubdomains':{value:false,source:'default',locked:false}},puts);
  render(()=><MessageLinks links={[inspectMessageLink('https://Docs.Partner.test/a','Docs')!]} truncated={false} scope="a" active/>);fireEvent.click(screen.getByRole('button',{name:'Review links (1)'}));fireEvent.click(screen.getByRole('button',{name:'Docs'}));
  fireEvent.click(screen.getByRole('button',{name:'Trust docs.partner.test'}));await waitFor(()=>expect(screen.getByRole('link',{name:'Docs'})).toBeVisible());expect(puts).toEqual([['corp.test','docs.partner.test']]);
 });
 it('offers no trust action when the administrator locks the list or the server does not know the key',async()=>{
  await seed({'links.trustedDomains':{value:[],source:'global',locked:true}});
  render(()=><MessageLinks links={[inspectMessageLink('https://partner.test/a','a')!]} truncated={false} scope="a" active/>);fireEvent.click(screen.getByRole('button',{name:'Review links (1)'}));fireEvent.click(screen.getByRole('button',{name:'a'}));
  expect(screen.queryByRole('button',{name:/^Trust /})).toBeNull();expect(screen.getByRole('link',{name:'Open in a new tab'})).toBeVisible();
 });
});
