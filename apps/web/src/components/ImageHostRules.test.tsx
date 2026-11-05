import {expect,it,vi} from 'vitest';
import {render,screen,fireEvent,within} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {ImageHostRules} from './ImageHostRules.tsx';
import {setImageHostState,type ImageHostLists,type ImageHostState} from '../viewers/imageHostRules.ts';
it('shows exactly one state per host, and choosing one replaces the other',()=>{
 const [lists,setLists]=createSignal<ImageHostLists>({trusted:['cdn.shop.test'],blocked:['ads.news.test']});
 const change=vi.fn((host:string,state:ImageHostState)=>setLists(setImageHostState(host,state,lists())!));
 render(()=><ImageHostRules hosts={['cdn.shop.test','ads.news.test','img.other.test']} lists={lists()} onChange={change}/>);
 const groups=screen.getAllByRole('radiogroup');expect(groups).toHaveLength(3);
 const checked=(i:number)=>within(groups[i]!).getAllByRole('radio').filter(r=>r.getAttribute('aria-checked')==='true').map(r=>r.textContent);
 expect([0,1,2].map(checked)).toEqual([['Always show'],['Never show'],['Ask']]);
 fireEvent.click(within(groups[0]!).getByText('Never show'));
 expect(change).toHaveBeenCalledWith('cdn.shop.test','block');expect(checked(0)).toEqual(['Never show']);
 expect(lists().trusted).toEqual([]);
 fireEvent.click(within(groups[0]!).getByText('Never show'));expect(change).toHaveBeenCalledTimes(1);
 fireEvent.click(within(groups[2]!).getByText('Always show'));expect(lists().trusted).toEqual(['img.other.test']);
});
it('is inert while the settings are locked',()=>{
 render(()=><ImageHostRules hosts={['a.test']} lists={{trusted:[],blocked:[]}} disabled onChange={()=>undefined}/>);
 for(const radio of screen.getAllByRole('radio'))expect(radio).toBeDisabled();
});
