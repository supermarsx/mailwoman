import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render} from '@solidjs/testing-library';
import {useFloatingLayer} from './floatingLayer.ts';
afterEach(()=>{cleanup();vi.restoreAllMocks();});
it('preserves caller-owned coordinates when promoting a menu into the top layer',()=>{
 const show=vi.fn(),hide=vi.fn();
 Object.defineProperty(HTMLElement.prototype,'showPopover',{configurable:true,value:show});
 Object.defineProperty(HTMLElement.prototype,'hidePopover',{configurable:true,value:hide});
 function Menu(){let element!:HTMLDivElement;useFloatingLayer(()=>element);return <div ref={element} style={{position:'fixed',top:'8px',left:'8px'}}/>;}
 const view=render(()=><Menu/>);const menu=view.container.firstElementChild as HTMLElement;
 expect(show).toHaveBeenCalledOnce();expect(menu.style.top).toBe('8px');expect(menu.style.left).toBe('8px');expect(menu.style.boxSizing).toBe('border-box');
});
