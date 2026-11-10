import {afterEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {ImageViewer,imageScale} from './ImageViewer.tsx';
import {clearSettings,loadSettings} from '../api/settings.ts';
const part={partId:null,blobId:'image',size:10,type:'image/png'};
afterEach(()=>{vi.restoreAllMocks();vi.unstubAllGlobals();clearSettings();});
it('fits wide, tall and rotated images while keeping actual and custom zoom explicit',()=>{
 const viewport={width:800,height:500};
 expect(imageScale({width:1600,height:800},viewport,0,'fit',1)).toBe(.5);
 expect(imageScale({width:1600,height:800},viewport,90,'fit',1)).toBe(.3125);
 expect(imageScale({width:100,height:100},viewport,0,'fit',1)).toBe(1);
 expect(imageScale({width:1600,height:800},viewport,0,'actual',1)).toBe(1);
 expect(imageScale({width:1600,height:800},viewport,0,'custom',20)).toBe(8);
 expect(imageScale({width:1600,height:800},viewport,0,'custom',.001)).toBe(.05);
});
async function setup(){
 vi.stubGlobal('Image',class {
  naturalWidth=1600;naturalHeight=800;onload:(()=>void)|null=null;onerror:(()=>void)|null=null;
  set src(_value:string){queueMicrotask(()=>this.onload?.());}
 });
 vi.stubGlobal('fetch',vi.fn(async()=>({ok:true,blob:async()=>new Blob(['pixels'],{type:'image/png'})})));
 return render(()=><ImageViewer part={part} blobUrl="blob:picture" mime="image/png" name="Picture.png"/>);
}
it('supports zoom, rotation, flips, reset and scoped keyboard actions without rebuilding the image frame',async()=>{
 const view=await setup();
 await waitFor(()=>expect(screen.getByRole('button',{name:'Zoom in'})).toBeEnabled());
 const frame=view.container.querySelector('iframe')!,doc=frame.srcdoc;
 const zoom=screen.getByRole('button',{name:'Zoom in'});zoom.focus();fireEvent.click(zoom);
 expect(screen.getByLabelText('Zoom level')).toHaveTextContent('125%');expect(zoom).toHaveFocus();
 fireEvent.click(screen.getByRole('button',{name:'Rotate right'}));expect(frame.style.transform).toContain('rotate(90deg)');
 fireEvent.click(screen.getByRole('button',{name:'Flip horizontally'}));expect(frame.style.transform).toContain('scale(-1, 1)');
 fireEvent.keyDown(view.container.querySelector('.image-preview__viewport')!,{key:'1'});expect(screen.getByLabelText('Zoom level')).toHaveTextContent('100%');
 fireEvent.click(screen.getByRole('button',{name:'Reset image view'}));expect(frame.style.transform).toContain('rotate(0deg) scale(1, 1)');
 expect(frame.srcdoc).toBe(doc);expect(frame.getAttribute('sandbox')).toBe('');
 expect(fetch).toHaveBeenCalledTimes(1);
});
it('keeps copying unavailable when download policy is disabled',async()=>{
 await loadSettings({fetcher:async()=>new Response(JSON.stringify({settings:{'mail.allowDownload':{value:false,source:'global',locked:true}}}))});
 await setup();await waitFor(()=>expect(screen.getByRole('button',{name:'Zoom in'})).toBeEnabled());
 expect(screen.queryByRole('button',{name:'Copy image'})).toBeNull();
});


it('copies original-resolution PNG pixels with the chosen rotation and confirms success',async()=>{
 const write=vi.fn(async(items:Array<{data:Record<string,Promise<Blob>>}>)=>{expect((await items[0]!.data['image/png']!).type).toBe('image/png');});
 vi.stubGlobal('ClipboardItem',class {constructor(readonly data:Record<string,Promise<Blob>>){}});
 vi.stubGlobal('navigator',{clipboard:{write}});
 const context={translate:vi.fn(),rotate:vi.fn(),scale:vi.fn(),drawImage:vi.fn()};
 vi.spyOn(HTMLCanvasElement.prototype,'getContext').mockReturnValue(context as unknown as CanvasRenderingContext2D);
 let dimensions:number[]=[];
 vi.spyOn(HTMLCanvasElement.prototype,'toBlob').mockImplementation(function(this:HTMLCanvasElement,callback){dimensions=[this.width,this.height];callback(new Blob(['png'],{type:'image/png'}));});
 await setup();await waitFor(()=>expect(screen.getByRole('button',{name:'Copy image'})).toBeEnabled());
 fireEvent.click(screen.getByRole('button',{name:'Rotate right'}));fireEvent.click(screen.getByRole('button',{name:'Copy image'}));
 await screen.findByText('Image copied.');
 expect(dimensions).toEqual([800,1600]);expect(context.rotate).toHaveBeenCalledWith(Math.PI/2);expect(write).toHaveBeenCalledOnce();
});
