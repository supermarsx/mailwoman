import {createRoot,createSignal} from 'solid-js';
import {expect,it} from 'vitest';
import {createWorkspaceViews,reconcileWorkspaceViews,workspaceLayout,WORKSPACE_LAYOUTS} from './workspaceLayout.ts';

it('recognizes five layouts and falls back safely for obsolete or malformed input',()=>{
  for(const mode of WORKSPACE_LAYOUTS)expect(workspaceLayout(mode)).toBe(mode);
  expect(workspaceLayout('unknown')).toBe('tabs');expect(workspaceLayout({})).toBe('tabs');
});
it('keeps exactly two selected views and puts a newly activated tab in the focused slot',()=>{
  expect(reconcileWorkspaceViews(['a','b'],['a','b','c'],'c',1)).toEqual(['a','c']);
  expect(reconcileWorkspaceViews(['a','b'],['a','b','c'],'c',0)).toEqual(['c','b']);
  expect(reconcileWorkspaceViews(['a','b'],['a','c'],'a',0)).toEqual(['a','c']);
  expect(reconcileWorkspaceViews(['a','a'],['a'],'a',1)).toEqual(['a',null]);
});
it('swaps assigned tabs, rejects foreign tabs, and clears assignments across authority generations',async()=>{
  let dispose!:()=>void;
  const model=createRoot(stop=>{
    dispose=stop;
    const [ids,setIds]=createSignal<readonly string[]>(['a','b','c']);
    const [active,setActive]=createSignal<string|null>('a');
    const [owner,setOwner]=createSignal<string|null>('first-generation');
    const views=createWorkspaceViews({ids,activeId:active,owner,activate:setActive});
    return {views,setIds,setOwner,setActive};
  });
  await Promise.resolve();
  expect(model.views.assignments()).toEqual(['a','b']);
  expect(model.views.assignView('a',1)).toBe(true);expect(model.views.assignments()).toEqual(['b','a']);
  expect(model.views.assignView('foreign',0)).toBe(false);
  model.views.assignView('c',0);expect(model.views.assignments()).toEqual(['c','a']);
  model.setOwner(null);await Promise.resolve();expect(model.views.assignments()).toEqual([null,null]);
  expect(model.views.assignView('a',0)).toBe(false);
  model.setIds(['x']);model.setActive('x');model.setOwner('second-generation');await Promise.resolve();
  expect(model.views.assignments()).toEqual(['x',null]);
  model.views.restore(['a','b']);expect(model.views.assignments()).toEqual(['x',null]);
  dispose();
});
