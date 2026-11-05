import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {fireEvent,screen,within} from '@solidjs/testing-library';
import {clearSettings,loadSettings,type Fetcher} from '../api/settings.ts';
import {MessageActions} from './MessageActions.tsx';
import {mkEmail,renderWithApp} from './appHarness.tsx';
import * as css from './messageRow.css.ts';

// `mail.rowActionsMode`. jsdom evaluates no layout, so these check the structure,
// state and classes the stylesheet keys on (hover reveal itself is CSS, covered in a browser).
const json=(body:unknown):Response=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
async function mode(value:string):Promise<void>{
 await loadSettings({refresh:true,fetcher:(async()=>json({settings:{'mail.rowActionsMode':{value,source:'user',locked:false}}})) as Fetcher});
}
function stubTouch(touch:boolean):void{
 vi.stubGlobal('matchMedia',(query:string)=>({matches:touch&&query.includes('hover: none'),media:query,addEventListener:()=>undefined,removeEventListener:()=>undefined,addListener:()=>undefined,removeListener:()=>undefined}));
}
const mount=()=>renderWithApp(()=><MessageActions email={mkEmail('a')}/>);
const hidden=(el:HTMLElement)=>el.classList.contains(css.actionSecondary);

beforeEach(()=>{clearSettings();localStorage.clear();stubTouch(false);});
afterEach(()=>{clearSettings();vi.unstubAllGlobals();});

describe('mail.rowActionsMode',()=>{
 it('open (the default) keeps the whole bar and no More button',()=>{
  mount();
  const cluster=screen.getByRole('group',{name:'More actions'});
  expect(within(cluster).getAllByRole('button')).toHaveLength(6);
  expect(screen.queryByTestId('msg-actions-expand')).toBeNull();
  expect(cluster).not.toHaveClass(css.actionsOnClick);
 });

 it('hover collapses to flag, archive and delete plus a More button that expands the rest',async()=>{
  await mode('hover');
  mount();
  const cluster=screen.getByRole('group',{name:'More actions'});
  const more=screen.getByTestId('msg-actions-expand');
  expect(more).toHaveAttribute('aria-expanded','false');
  expect(more).toHaveAttribute('aria-haspopup','true');
  for(const name of ['Pin','Snooze','Label'])expect(hidden(within(cluster).getByRole('button',{name,hidden:true}).closest('.msg-actions > *') as HTMLElement)).toBe(true);
  for(const name of ['Flag for follow-up','Archive','Delete'])expect(hidden(within(cluster).getByRole('button',{name}))).toBe(false);
  fireEvent.click(more);
  expect(more).toHaveAttribute('aria-expanded','true');
  expect(hidden(within(cluster).getByRole('button',{name:'Pin'}))).toBe(false);
 });

 it('hover: focus on the row controls and Escape collapse the expanded bar',async()=>{
  await mode('hover');
  mount();
  const more=screen.getByTestId('msg-actions-expand');
  more.focus();
  fireEvent.click(more);
  expect(more).toHaveAttribute('aria-expanded','true');
  fireEvent.keyDown(more,{key:'Escape'});
  expect(more).toHaveAttribute('aria-expanded','false');
  expect(document.activeElement).toBe(more);
 });

 it('click shows the toggle on every pointer and opens the bar on click, Escape closes',async()=>{
  await mode('click');
  mount();
  const toggle=screen.getByRole('button',{name:'More actions'});
  const cluster=screen.getByRole('group',{name:'More actions'});
  expect(toggle).toHaveClass(css.moreToggleAlways);
  expect(cluster).toHaveClass(css.actionsOnClick);
  expect(cluster).not.toHaveClass(css.actionsOnClickOpen);
  fireEvent.click(toggle);
  expect(toggle).toHaveAttribute('aria-expanded','true');
  expect(cluster).toHaveClass(css.actionsOnClickOpen);
  fireEvent.keyDown(cluster,{key:'Escape'});
  expect(toggle).toHaveAttribute('aria-expanded','false');
  expect(cluster).not.toHaveClass(css.actionsOnClickOpen);
 });

 it('off renders no row controls',async()=>{
  await mode('off');
  mount();
  expect(screen.queryByTestId('msg-more-toggle')).toBeNull();
  expect(screen.queryByRole('group',{name:'More actions'})).toBeNull();
 });

 it('off keeps the follow-up mark',async()=>{
  await mode('off');
  renderWithApp(()=><MessageActions email={{...mkEmail('a'),followUpAt:'2030-01-01T00:00:00Z'}}/>);
  expect(screen.getByTestId('msg-followup-mark')).toBeInTheDocument();
 });

 it('hover behaves like click without a hovering pointer',async()=>{
  stubTouch(true);
  await mode('hover');
  mount();
  expect(screen.queryByTestId('msg-actions-expand')).toBeNull();
  expect(screen.getByRole('button',{name:'More actions'})).toHaveClass(css.moreToggleAlways);
  expect(screen.getByRole('group',{name:'More actions'})).toHaveClass(css.actionsOnClick);
 });

 it('an unknown value falls back to open',async()=>{
  await mode('sideways');
  mount();
  expect(screen.queryByTestId('msg-actions-expand')).toBeNull();
  expect(within(screen.getByRole('group',{name:'More actions'})).getAllByRole('button')).toHaveLength(6);
 });
});
