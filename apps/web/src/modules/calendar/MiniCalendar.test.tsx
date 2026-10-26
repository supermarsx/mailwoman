import {expect,it,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {MiniCalendar} from './MiniCalendar.tsx';
it('navigates across a year boundary and selects the exact local calendar day',()=>{
 const select=vi.fn();render(()=><MiniCalendar date={new Date(2026,11,15)} select={select}/>);
 fireEvent.click(screen.getByRole('button',{name:'Next month'}));
 fireEvent.click(screen.getByRole('button',{name:/January 12, 2027/}));
 const date=select.mock.calls[0]![0] as Date;
 expect([date.getFullYear(),date.getMonth(),date.getDate()]).toEqual([2027,0,12]);
});
