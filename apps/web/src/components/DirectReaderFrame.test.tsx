import {it,expect,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {DirectReaderFrame} from './DirectReaderFrame.tsx';
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
