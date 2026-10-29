import {beforeAll,expect,it,vi} from 'vitest';
import {render,fireEvent} from '@solidjs/testing-library';
import {loadCatalog} from '../../../i18n';
import {formatMeasured,formatMs,formatMsSet} from './format.ts';
import {Sparkline} from '../shell/Sparkline.tsx';
const MU='μ';
beforeAll(async()=>{await loadCatalog('admin');});
it('shows sub-millisecond times in μs and never a non-zero value as 0 ms',()=>{
 for(const [value,expected] of [[0,'0 ms'],[0.001,`1 ${MU}s`],[0.123,`123 ${MU}s`],[0.999,`999 ${MU}s`],[1,'1 ms'],[1.25,'1.25 ms'],[25.7,'25.7 ms'],[2400,'2.4 s']] as const)expect(formatMs(value)).toBe(expected);
 expect(formatMs(0.0004)).toBe('400 ns');
 expect(formatMs(0.0004,0.001)).toBe(`<1 ${MU}s`);
});
it('prefers exact microseconds and reads a truncated whole-ms 0 as <1 ms',()=>{
 expect(formatMeasured(0,312)).toBe(`312 ${MU}s`);
 expect(formatMeasured(12,12_480)).toBe('12.5 ms');
 expect(formatMeasured(0,0)).toBe(`<1 ${MU}s`);
 expect(formatMeasured(0)).toBe('<1 ms');
 expect(formatMeasured(7)).toBe('7 ms');
 expect(formatMeasured(undefined)).toBe('—');
});
it('hover picks the best unit per point; the summary and table share the unit of the largest value',()=>{
 const {container}=render(()=><Sparkline title="Latency" points={[{t:1,value:0.001},{t:2,value:0.123},{t:3,value:4.5}]} format={formatMs} formatSet={formatMsSet} formatTime={value=>String(value)}/>);
 const svg=container.querySelector('svg')!;
 vi.spyOn(svg,'getBoundingClientRect').mockReturnValue({x:0,y:0,left:0,top:0,right:100,bottom:48,width:100,height:48,toJSON:()=>({})});
 fireEvent(svg,new MouseEvent('pointermove',{clientX:50,bubbles:true}));
 expect(container.querySelector('p[aria-hidden="true"]')).toHaveTextContent(`123 ${MU}s`);
 const table=container.querySelector('table')!;
 expect(table).toHaveTextContent('0.001 ms');
 expect(table).toHaveTextContent('0.123 ms');
 expect(table).toHaveTextContent('4.5 ms');
 expect(container.textContent).not.toMatch(/\b0 ms/);
});
it('passes each point\'s resolution to the formatter',()=>{
 const {container}=render(()=><Sparkline title="Latency" points={[{t:1,value:0,resolution:1},{t:2,value:3,resolution:1}]} format={formatMs} formatSet={formatMsSet} formatTime={value=>String(value)}/>);
 expect(container.querySelector('table')).toHaveTextContent('<1 ms');
});
