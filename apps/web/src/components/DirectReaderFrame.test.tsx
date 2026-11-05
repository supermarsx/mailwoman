import {it,expect,vi,afterEach} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {DirectReaderFrame} from './DirectReaderFrame.tsx';
afterEach(()=>vi.useRealTimers());
it('reports internally consistent image progress only from the current authenticated document',()=>{
 const progress=vi.fn();const [key,setKey]=createSignal('one');const [html,setHtml]=createSignal('<p>Images</p>');
 const mounted=render(()=><DirectReaderFrame html={html()} documentKey={key()} title="Progress" onImageProgress={progress}/>);
 const frame=screen.getByTitle('Progress') as HTMLIFrameElement;const token=new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');
 const send=(data:unknown,origin='null',source:MessageEventSource|null=frame.contentWindow):void=>{fireEvent(window,new MessageEvent('message',{source,origin,data}));};
 const value={type:'mw-reader-image-progress',token,revision:1,loaded:2,failed:1,pending:3,total:6};
 send(value);expect(progress).not.toHaveBeenCalled();send({type:'mw-reader-ready',token});
 send(value,'https://mail.example');send(value,'null',window);send({...value,token:'bad'});send({...value,revision:0});
 for(const invalid of [{...value,total:7},{...value,loaded:-1},{...value,pending:NaN},{...value,failed:0.5},{...value,total:Infinity},{...value,total:'6'},{...value,loaded:1_000_001,total:1_000_005}])send(invalid);
 expect(progress).not.toHaveBeenCalled();send(value);expect(progress).toHaveBeenCalledWith({loaded:2,failed:1,pending:3,total:6});
 setKey('two');send(value);expect(progress).toHaveBeenCalledTimes(1);send({...value,revision:2,loaded:3,pending:2});expect(progress).toHaveBeenCalledTimes(2);
 setHtml('<p>Different images</p>');send({...value,revision:2});expect(progress).toHaveBeenCalledTimes(2);send({...value,revision:3,loaded:0,failed:0,pending:0,total:0});expect(progress).toHaveBeenLastCalledWith({loaded:0,failed:0,pending:0,total:0});
 mounted.unmount();send({...value,revision:3});expect(progress).toHaveBeenCalledTimes(3);
});
it('retries a blocked bridge once with a fresh token and ignores obsolete acknowledgements',()=>{
 vi.useFakeTimers();const displayed=vi.fn();
 const mounted=render(()=><DirectReaderFrame html="<p>Private message</p>" title="Content" onDisplayed={displayed}/>);
 const frame=screen.getByTitle('Content') as HTMLIFrameElement;
 const token=()=>new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');const old=token();
 fireEvent.load(frame);vi.advanceTimersByTime(250);
 expect(new URL(frame.src).searchParams.get('coep')).toBe('require-corp');expect(token()).not.toBe(old);
 const send=(value:unknown)=>fireEvent(window,new MessageEvent('message',{source:frame.contentWindow,origin:'null',data:value}));
 send({type:'mw-reader-ready',token:old});send({type:'mw-reader-displayed',token:old,revision:1});expect(displayed).not.toHaveBeenCalled();
 send({type:'mw-reader-ready',token:token()});send({type:'mw-reader-displayed',token:token(),revision:1});vi.advanceTimersByTime(40);expect(displayed).toHaveBeenCalledWith('<p>Private message</p>',undefined);
 const current=frame.src;vi.advanceTimersByTime(20000);expect(frame.src).toBe(current);expect(screen.queryByRole('alert')).toBeNull();
 mounted.unmount();expect(vi.getTimerCount()).toBe(0);
});
it('keeps one absolute deadline and never retries indefinitely',()=>{
 vi.useFakeTimers();render(()=><DirectReaderFrame html="<p>Message</p>" title="Content"/>);
 const frame=screen.getByTitle('Content') as HTMLIFrameElement;
 vi.advanceTimersByTime(1500);expect(frame.src).toContain('?coep=require-corp');const current=frame.src;
 vi.advanceTimersByTime(8500);expect(screen.getByRole('alert')).toBeInTheDocument();
 fireEvent.load(frame);vi.advanceTimersByTime(20000);expect(frame.src).toBe(current);
});
it('accepts only the current correlated display acknowledgement from its opaque wrapper',async()=>{
 const displayed=vi.fn();const [key,setKey]=createSignal('first');
 render(()=><DirectReaderFrame html="<p>Same text</p>" title="Content" documentKey={key()} onDisplayed={displayed}/>);
 const frame=screen.getByTitle('Content') as HTMLIFrameElement;const token=new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');
 const message=(data:unknown,source:MessageEventSource|null=frame.contentWindow):void=>{fireEvent(window,new MessageEvent('message',{source,origin:'null',data}));};
 message({type:'mw-reader-ready',token});message({type:'mw-reader-displayed',token,revision:1},window);expect(displayed).not.toHaveBeenCalled();
 message({type:'mw-reader-displayed',token:'wrong',revision:1});expect(displayed).not.toHaveBeenCalled();
 setKey('second');message({type:'mw-reader-displayed',token,revision:1});expect(displayed).not.toHaveBeenCalled();
 message({type:'mw-reader-displayed',token,revision:2});await waitFor(()=>expect(displayed).toHaveBeenCalledWith('<p>Same text</p>','second'));
});
// t60: an iframe moved in the DOM reloads into a new, empty bridge document: a
// workspace pane reordered by <For>, or detached into a popup window and back.
// The new bridge must get the message again, or the message frame stays blank.
afterEach(()=>{for(const stray of document.body.querySelectorAll('body > iframe, body > div:not([data-testid])'))stray.remove();});
function bridge(html:string){
 render(()=><DirectReaderFrame html={html} title="Content"/>);
 const frame=screen.getByTitle('Content') as HTMLIFrameElement;const token=new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');
 const posted:{window:Window;type:string;html?:string}[]=[];
 const watch=():void=>{const win=frame.contentWindow!;vi.spyOn(win,'postMessage').mockImplementation(((data:{type:string;html?:string})=>{posted.push({window:win,type:data.type,...(data.html===undefined?{}:{html:data.html})});}) as never);};
 // The bridge announces itself to the window whose document holds the frame.
 const ready=():void=>{const host=frame.ownerDocument.defaultView!;host.dispatchEvent(new host.MessageEvent('message',{source:frame.contentWindow,origin:'null',data:{type:'mw-reader-ready',token}}));};
 const moveTo=(parent:HTMLElement):void=>{parent.append(frame);watch();};
 const writes=():{window:Window;html?:string}[]=>posted.filter(item=>item.type==='mw-reader-frame');
 watch();
 return {frame,ready,moveTo,writes,posted};
}
it('writes the message again into a bridge reloaded by a DOM move',()=>{
 const {frame,ready,moveTo,writes}=bridge('<p>Kept message</p>');
 ready();fireEvent.load(frame);
 expect(writes().map(item=>item.html)).toEqual(['<p>Kept message</p>']);
 const first=frame.contentWindow;
 const elsewhere=document.createElement('div');document.body.append(elsewhere);moveTo(elsewhere);
 expect(frame.contentWindow).not.toBe(first);
 ready();fireEvent.load(frame);
 expect(writes()).toHaveLength(2);expect(writes()[1]!.window).toBe(frame.contentWindow);
});
it('writes the message into a bridge detached into another window and back',()=>{
 const {frame,ready,moveTo,writes,posted}=bridge('<p>Detached message</p>');
 ready();fireEvent.load(frame);expect(writes()).toHaveLength(1);
 // A same-origin document standing in for the popup window.
 const holder=document.createElement('iframe');document.body.append(holder);
 const popup=holder.contentDocument!;
 moveTo(popup.body);expect(frame.ownerDocument).toBe(popup);
 // The new bridge's first announcement went to the popup before the app listened there.
 fireEvent.load(frame);
 const asked=posted.filter(item=>item.window===frame.contentWindow);
 expect(asked.length).toBeGreaterThan(0);
 if(writes().length===1){ready();}
 expect(writes()).toHaveLength(2);expect(writes()[1]!.window).toBe(frame.contentWindow);expect(writes()[1]!.html).toBe('<p>Detached message</p>');
 // Reattached to the main window: the bridge reloads again.
 moveTo(document.body);ready();fireEvent.load(frame);
 expect(writes()).toHaveLength(3);expect(writes()[2]!.window).toBe(frame.contentWindow);
});
it('does not write a freshly announced bridge twice when its load event follows',()=>{
 const {frame,ready,writes}=bridge('<p>Once</p>');
 ready();fireEvent.load(frame);fireEvent.load(frame);
 expect(writes()).toHaveLength(1);
});

it('passes explicit consent to the authenticated bridge and permits only popup navigation',()=>{
 const view=render(()=><DirectReaderFrame html="<p>Links</p>" title="Consent" linksEnabled/>);
 const frame=screen.getByTitle('Consent') as HTMLIFrameElement;
 expect(frame.getAttribute('sandbox')).toBe('allow-scripts allow-popups allow-popups-to-escape-sandbox');
 expect(frame.getAttribute('referrerpolicy')).toBe('no-referrer');
 const post=vi.spyOn(frame.contentWindow!,'postMessage');
 const token=new URL(frame.src).hash.slice(1);const value=new URLSearchParams(token).get('token');
 fireEvent(window,new MessageEvent('message',{source:frame.contentWindow,origin:'null',data:{type:'mw-reader-ready',token:value}}));
 expect(post).toHaveBeenCalledWith(expect.objectContaining({type:'mw-reader-frame',linksEnabled:true}),'*');
 view.unmount();render(()=><DirectReaderFrame html="<p>Other message</p>" title="No consent"/>);
 // The outer wrapper is never reloaded for consent; the nested document's own sandbox carries the consent.
 expect(screen.getByTitle('No consent')).toHaveAttribute('sandbox','allow-scripts allow-popups allow-popups-to-escape-sandbox');
 expect(post.mock.calls.some(call=>(call[0] as {linksEnabled?:boolean}).linksEnabled===false)).toBe(false);
});

it('accepts address context only for the current authenticated body and validates coordinates/mailbox before translating zoom',()=>{
 const opened=vi.fn();const [key,setKey]=createSignal('one');
 render(()=><DirectReaderFrame html="<p>Address</p>" documentKey={key()} title="Address" onAddressContext={opened}/>);
 const frame=screen.getByTitle('Address') as HTMLIFrameElement;
 vi.spyOn(frame,'getBoundingClientRect').mockReturnValue({left:10,top:20,width:400,height:300} as DOMRect);
 Object.defineProperties(frame,{clientWidth:{value:200},clientHeight:{value:150}});
 const token=new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');
 const send=(data:unknown,origin='null',source:MessageEventSource|null=frame.contentWindow):void=>{fireEvent(window,new MessageEvent('message',{source,origin,data}));};
 send({type:'mw-reader-ready',token});
 const data={type:'mw-reader-address-context',token,revision:1,address:'Case@EXAMPLE.ORG',x:20,y:30};
 send(data,'https://mail.example');send(data,'null',window);send({...data,token:'bad'});send({...data,address:'x@example.org,y@example.org'});send({...data,x:201});expect(opened).not.toHaveBeenCalled();
 send(data);expect(opened).toHaveBeenCalledWith({address:'Case@example.org',x:50,y:80,ownerDocument:document,trigger:frame});
 setKey('two');send(data);expect(opened).toHaveBeenCalledTimes(1);
 frame.setAttribute('inert','');send({...data,revision:2});expect(opened).toHaveBeenCalledTimes(1);
});

it('keeps the loading surface until the current parsed body has paint time, masks replacements, and cancels stale paints',()=>{
 const paints=new Map<number,FrameRequestCallback>();let next=0;
 const request=vi.spyOn(window,'requestAnimationFrame').mockImplementation(callback=>{paints.set(++next,callback);return next;});
 const cancel=vi.spyOn(window,'cancelAnimationFrame').mockImplementation(id=>{paints.delete(id);});
 const flush=():void=>{const pending=[...paints.values()];paints.clear();for(const callback of pending)callback(0);};
 const displayed=vi.fn();const [html,setHtml]=createSignal('<p>First</p>');
 const mounted=render(()=><DirectReaderFrame html={html()} title="Paint" loadingFallback={<span>Preparing current message</span>} onDisplayed={displayed}/>);
 const frame=screen.getByTitle('Paint') as HTMLIFrameElement;const token=new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');
 const send=(data:unknown):void=>{fireEvent(window,new MessageEvent('message',{source:frame.contentWindow,origin:'null',data}));};
 expect(frame).toHaveAttribute('aria-hidden','true');expect(screen.getByTestId('reader-frame-loading')).toHaveTextContent('Preparing current message');
 send({type:'mw-reader-ready',token});expect(screen.getByTestId('reader-frame-loading')).toBeInTheDocument();
 send({type:'mw-reader-displayed',token,revision:1});flush();expect(displayed).not.toHaveBeenCalled();
 setHtml('<p>Second</p>');flush();expect(displayed).not.toHaveBeenCalled();expect(frame).toHaveAttribute('aria-hidden','true');
 send({type:'mw-reader-displayed',token,revision:1});expect(paints.size).toBe(0);
 send({type:'mw-reader-displayed',token,revision:2});flush();flush();
 expect(displayed).toHaveBeenCalledWith('<p>Second</p>',undefined);expect(frame).toHaveAttribute('aria-hidden','false');expect(screen.queryByTestId('reader-frame-loading')).toBeNull();
 send({type:'mw-reader-displayed',token,revision:2});expect(paints.size).toBe(0);
 setHtml('<p>Third</p>');send({type:'mw-reader-displayed',token,revision:3});mounted.unmount();expect(paints.size).toBe(0);
 request.mockRestore();cancel.mockRestore();
});

const gif='data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
const signed=`${location.origin}/api/rc/i/handle1`;
function shown(initial:string,linksEnabled=false){
 const paints=new Map<number,FrameRequestCallback>();let next=0;
 vi.spyOn(window,'requestAnimationFrame').mockImplementation(callback=>{paints.set(++next,callback);return next;});
 vi.spyOn(window,'cancelAnimationFrame').mockImplementation(id=>{paints.delete(id);});
 const flush=():void=>{for(let n=0;n<2;n++){const pending=[...paints.values()];paints.clear();for(const callback of pending)callback(0);}};
 const [html,setHtml]=createSignal(initial);const [links,setLinks]=createSignal(linksEnabled);
 render(()=><DirectReaderFrame html={html()} linksEnabled={links()} documentKey="m1" title="Hot"/>);
 const frame=screen.getByTitle('Hot') as HTMLIFrameElement;const token=new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');
 const post=vi.spyOn(frame.contentWindow!,'postMessage');
 const send=(data:unknown):void=>{fireEvent(window,new MessageEvent('message',{source:frame.contentWindow,origin:'null',data}));};
 send({type:'mw-reader-ready',token});send({type:'mw-reader-displayed',token,revision:1});flush();
 post.mockClear();
 return {frame,token,post,send,flush,setHtml,setLinks};
}
it('patches image references in the shown document instead of reloading it',()=>{
 const h=shown(`<!doctype html><html><head></head><body><p>Hi</p><img src="${gif}"></body></html>`);
 expect(h.frame).toHaveAttribute('aria-hidden','false');
 h.setHtml(`<!doctype html><html><head></head><body><p>Hi</p><img src="${signed}"></body></html>`);
 expect(h.post).toHaveBeenCalledTimes(1);
 expect(h.post).toHaveBeenCalledWith({type:'mw-reader-patch',token:h.token,revision:1,id:1,ops:[{s:'b',i:1,a:'src',v:signed}]},'*');
 expect(h.frame).toHaveAttribute('aria-hidden','false');expect(screen.queryByTestId('reader-frame-loading')).toBeNull();
});
it('does not patch toward unsigned sources and refreshes behind the shown document instead',()=>{
 const h=shown(`<!doctype html><html><body><img src="${gif}"></body></html>`);
 h.setHtml('<!doctype html><html><body><img src="https://tracker.example/p.gif"></body></html>');
 expect(h.post).toHaveBeenCalledTimes(1);
 expect(h.post.mock.calls[0]![0]).toMatchObject({type:'mw-reader-frame',swap:true,revision:2});
 expect(h.frame).toHaveAttribute('aria-hidden','false');
});
it('a changed link consent swaps the nested document without hiding the message and without remounting',()=>{
 const h=shown('<!doctype html><html><body><a>Link</a></body></html>');
 h.setLinks(true);h.setHtml('<!doctype html><html><body><a href="https://x.test/" target="_blank">Link</a></body></html>');
 expect(h.post).toHaveBeenCalledWith(expect.objectContaining({type:'mw-reader-frame',swap:true,linksEnabled:true,revision:2}),'*');
 expect(h.frame).toHaveAttribute('aria-hidden','false');expect(screen.getByTitle('Hot')).toBe(h.frame);
 h.send({type:'mw-reader-displayed',token:h.token,revision:2});h.flush();expect(h.frame).toHaveAttribute('aria-hidden','false');
});
it('falls back to a swap when the document refuses a patch, and ignores stale acknowledgements',()=>{
 const h=shown(`<!doctype html><html><body><img src="${gif}"></body></html>`);
 h.setHtml(`<!doctype html><html><body><img src="${signed}"></body></html>`);h.post.mockClear();
 h.send({type:'mw-reader-patched',token:h.token,revision:1,id:99,ok:false});expect(h.post).not.toHaveBeenCalled();
 h.send({type:'mw-reader-patched',token:h.token,revision:1,id:1,ok:true});expect(h.post).not.toHaveBeenCalled();
 h.send({type:'mw-reader-patched',token:h.token,revision:1,id:1,ok:false});
 expect(h.post).toHaveBeenCalledWith(expect.objectContaining({type:'mw-reader-frame',swap:true}),'*');
});

// Show images -> Undo -> Show images again (and rapid toggles): the frame is always reconciled to the latest desired document.
const blockedDoc=`<!doctype html><html><head></head><body><p>Hi</p><img data-mw-ph="sized" data-mw-blocked-src="mw-rc:1"></body></html>`;
const shownDoc=`<!doctype html><html><head></head><body><p>Hi</p><img src="${signed}"></body></html>`;
const lastDoc=(h:ReturnType<typeof shown>):unknown=>h.post.mock.calls.filter(c=>(c[0] as {type:string}).type==='mw-reader-frame').at(-1)?.[0];
const settle=(h:ReturnType<typeof shown>):void=>{const rev=(lastDoc(h) as {revision:number}|undefined)?.revision;if(rev)h.send({type:'mw-reader-displayed',token:h.token,revision:rev});h.flush();};
it('show -> undo -> show again re-delivers the images each time',()=>{
 const h=shown(blockedDoc);
 for(let round=0;round<3;round++){
  h.setHtml(shownDoc);expect((lastDoc(h) as {html:string}).html).toBe(shownDoc);settle(h);
  h.setHtml(blockedDoc);expect((lastDoc(h) as {html:string}).html).toBe(blockedDoc);settle(h);
 }
 h.setHtml(shownDoc);expect((lastDoc(h) as {html:string}).html).toBe(shownDoc);settle(h);
 expect(h.frame).toHaveAttribute('aria-hidden','false');
});
it('show -> undo -> redo -> show with no acknowledgement between toggles ends on the latest document',()=>{
 const h=shown(blockedDoc);
 for(const doc of [shownDoc,blockedDoc,shownDoc,blockedDoc,shownDoc])h.setHtml(doc);
 expect((lastDoc(h) as {html:string}).html).toBe(shownDoc);
 settle(h);expect(h.frame).toHaveAttribute('aria-hidden','false');
 h.setHtml(blockedDoc);expect((lastDoc(h) as {html:string}).html).toBe(blockedDoc);
});
it('a patched toggle (same structure) also survives show -> undo -> show',()=>{
 const a=`<!doctype html><html><body><img src="${gif}"></body></html>`,b=`<!doctype html><html><body><img src="${signed}"></body></html>`;
 const h=shown(a);
 for(let n=0;n<3;n++){h.setHtml(b);h.send({type:'mw-reader-patched',token:h.token,revision:1,id:n*2+1,ok:true});h.setHtml(a);h.send({type:'mw-reader-patched',token:h.token,revision:1,id:n*2+2,ok:true});}
 h.post.mockClear();h.setHtml(b);
 expect(h.post.mock.calls.at(-1)![0]).toMatchObject({type:'mw-reader-patch',ops:[{a:'src',v:signed}]});
});
