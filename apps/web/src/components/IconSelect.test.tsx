import {afterEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {IconSelect,type IconSelectOption} from './IconSelect.tsx';

const options:readonly IconSelectOption[]=[
 {value:'right',label:'Right',icon:'layoutColumns'},
 {value:'bottom',label:'Bottom',icon:'layoutRows'},
 {value:'off',label:'Off',icon:'mail'},
];
afterEach(()=>{cleanup();suspendAccountPreferences();});

describe('icon dropdown button',()=>{
 it('shares the app button styling and chooses a labelled icon option by click',()=>{
  const change=vi.fn();
  render(()=>{const [value,setValue]=createSignal('right');return <IconSelect label="Reading pane" value={value()} options={options} onChange={next=>{setValue(next);change(next);}}/>;});
  const trigger=screen.getByRole('button',{name:'Reading pane'});
  expect(trigger).toHaveClass('btn');
  expect(trigger).toHaveAttribute('aria-expanded','false');
  expect(trigger.querySelector('[data-icon="layoutColumns"]')).not.toBeNull();
  fireEvent.click(trigger);
  expect(trigger).toHaveAttribute('aria-expanded','true');
  expect(screen.getByRole('menuitemradio',{name:'Right'})).toHaveAttribute('aria-checked','true');
  fireEvent.click(screen.getByRole('menuitemradio',{name:'Bottom'}));
  expect(change).toHaveBeenCalledExactlyOnceWith('bottom');
  expect(screen.queryByRole('menu')).toBeNull();
  expect(trigger).toHaveFocus();
  expect(trigger).toHaveAttribute('title','Reading pane: Bottom');
  expect(trigger.querySelector('[data-icon="layoutRows"]')).not.toBeNull();
 });
 it('opens at the selected item with the keyboard and closes on account change',()=>{
  activateAccountPreferences('account-one');
  const change=vi.fn();
  render(()=><IconSelect label="Reading pane" value="bottom" options={options} onChange={change}/>);
  const trigger=screen.getByRole('button',{name:'Reading pane'});
  trigger.focus();
  fireEvent.keyDown(trigger,{key:'ArrowDown'});
  expect(screen.getByRole('menuitemradio',{name:'Bottom'})).toHaveFocus();
  fireEvent.keyDown(document.activeElement!,{key:'Escape'});
  expect(trigger).toHaveFocus();
  expect(screen.queryByRole('menu')).toBeNull();
  fireEvent.keyDown(trigger,{key:'ArrowUp'});
  activateAccountPreferences('account-two');
  expect(screen.queryByRole('menu')).toBeNull();
  expect(trigger).toHaveAttribute('aria-expanded','false');
  expect(change).not.toHaveBeenCalled();
 });
});
