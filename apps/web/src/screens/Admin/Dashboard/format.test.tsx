import {beforeAll,expect,it,vi} from 'vitest';
import {render,fireEvent} from '@solidjs/testing-library';
import {loadCatalog} from '../../../i18n';
import {formatMs} from './format.ts';
import {Sparkline} from '../shell/Sparkline.tsx';
beforeAll(async()=>{await loadCatalog('admin');});
it('preserves measured sub-millisecond precision without padding integer readings',()=>{
 for(const [value,expected] of [[0,'0 ms'],[0.001,'0.001 ms'],[0.123,'0.123 ms'],[0.999,'0.999 ms'],[1,'1 ms'],[1.25,'1.3 ms'],[25.7,'26 ms']] as const)expect(formatMs(value)).toBe(expected);
 expect(formatMs(0.0004)).toBe('<0.001 ms');
});
it('uses the same sub-millisecond precision in chart hover and accessible data',()=>{
 const {container}=render(()=><Sparkline title="Latency" points={[{t:1,value:0.001},{t:2,value:0.123},{t:3,value:0.999}]} format={formatMs} formatTime={value=>String(value)}/>);
 const svg=container.querySelector('svg')!;
 vi.spyOn(svg,'getBoundingClientRect').mockReturnValue({x:0,y:0,left:0,top:0,right:100,bottom:48,width:100,height:48,toJSON:()=>({})});
 fireEvent(svg,new MouseEvent('pointermove',{clientX:50,bubbles:true}));
 expect(container.querySelector('p[aria-hidden="true"]')).toHaveTextContent('0.123 ms');
 expect(container.querySelector('table')).toHaveTextContent('0.001 ms');
 expect(container.querySelector('table')).toHaveTextContent('0.999 ms');
});
