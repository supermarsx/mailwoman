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
