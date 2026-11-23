import {it,expect,vi,afterEach} from 'vitest';
import {render,fireEvent,screen} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {createMobileScrollChrome} from './mobileScrollChrome.ts';
afterEach(()=>vi.restoreAllMocks());
it('compacts only its mobile viewport and restores on upward scroll, top and scope changes',()=>{
 let root:HTMLDivElement|undefined;const [scope,setScope]=createSignal('inbox');const [active,setActive]=createSignal(true);let now=0;
 vi.spyOn(performance,'now').mockImplementation(()=>now);
 render(()=>{const state=createMobileScrollChrome({root:()=>root,selector:'.viewport',active,scope});return <div ref={root}><div class="viewport" data-testid="viewport"/><div data-testid="unrelated"/><output>{state.hidden()?'compact':'expanded'}</output></div>;});
 const scroll=(id:string,y:number)=>{now+=200;const node=screen.getByTestId(id);Object.defineProperty(node,'scrollTop',{value:y,configurable:true});fireEvent.scroll(node);};
 scroll('unrelated',200);expect(screen.getByText('expanded')).toBeInTheDocument();
 scroll('viewport',100);expect(screen.getByText('compact')).toBeInTheDocument();
 scroll('viewport',70);expect(screen.getByText('expanded')).toBeInTheDocument();
 scroll('viewport',130);expect(screen.getByText('compact')).toBeInTheDocument();
 setScope('sent');expect(screen.getByText('expanded')).toBeInTheDocument();
 scroll('viewport',180);expect(screen.getByText('compact')).toBeInTheDocument();
 scroll('viewport',0);expect(screen.getByText('expanded')).toBeInTheDocument();
 setActive(false);scroll('viewport',100);expect(screen.getByText('expanded')).toBeInTheDocument();
});
