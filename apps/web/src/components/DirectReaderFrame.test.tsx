import {it,expect,vi,afterEach} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {DirectReaderFrame} from './DirectReaderFrame.tsx';
afterEach(()=>vi.useRealTimers());
it('retries a blocked bridge once with a fresh token and ignores obsolete acknowledgements',()=>{
 vi.useFakeTimers();const displayed=vi.fn();
 const mounted=render(()=><DirectReaderFrame html="<p>Private message</p>" title="Content" onDisplayed={displayed}/>);
 const frame=screen.getByTitle('Content') as HTMLIFrameElement;
 const token=()=>new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');const old=token();
 fireEvent.load(frame);vi.advanceTimersByTime(250);
 expect(new URL(frame.src).searchParams.get('coep')).toBe('require-corp');expect(token()).not.toBe(old);
 const send=(value:unknown)=>fireEvent(window,new MessageEvent('message',{source:frame.contentWindow,origin:'null',data:value}));
 send({type:'mw-reader-ready',token:old});send({type:'mw-reader-displayed',token:old,revision:1});expect(displayed).not.toHaveBeenCalled();
 send({type:'mw-reader-ready',token:token()});send({type:'mw-reader-displayed',token:token(),revision:1});expect(displayed).toHaveBeenCalledWith('<p>Private message</p>',undefined);
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
it('accepts only the current correlated display acknowledgement from its opaque wrapper',()=>{
 const displayed=vi.fn();const [key,setKey]=createSignal('first');
 render(()=><DirectReaderFrame html="<p>Same text</p>" title="Content" documentKey={key()} onDisplayed={displayed}/>);
 const frame=screen.getByTitle('Content') as HTMLIFrameElement;const token=new URLSearchParams(new URL(frame.src).hash.slice(1)).get('token');
 const message=(data:unknown,source:MessageEventSource|null=frame.contentWindow):void=>{fireEvent(window,new MessageEvent('message',{source,origin:'null',data}));};
 message({type:'mw-reader-ready',token});message({type:'mw-reader-displayed',token,revision:1},window);expect(displayed).not.toHaveBeenCalled();
 message({type:'mw-reader-displayed',token:'wrong',revision:1});expect(displayed).not.toHaveBeenCalled();
 setKey('second');message({type:'mw-reader-displayed',token,revision:1});expect(displayed).not.toHaveBeenCalled();
 message({type:'mw-reader-displayed',token,revision:2});expect(displayed).toHaveBeenCalledWith('<p>Same text</p>','second');
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
