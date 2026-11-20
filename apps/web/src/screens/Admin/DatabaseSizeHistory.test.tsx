import {expect,it} from 'vitest';
import {fireEvent,render,screen,within} from '@solidjs/testing-library';
import {DatabaseSizeHistory} from './DatabaseSizeHistory.tsx';
import {formatBytes} from './Dashboard/format.ts';
const GiB=1024**3;
const samples=[{at:1000,dbBytes:115*GiB,walBytes:1024},{at:2000,dbBytes:115.1*GiB,walBytes:2048},{at:5000,dbBytes:115.2*GiB,walBytes:4096}];
it('shows themed charts immediately, filters time ranges and retains a table alternative',()=>{
 render(()=><DatabaseSizeHistory samples={samples}/>);
 const card=screen.getByTestId('database-size-history');
 expect(card.querySelector('svg')).not.toBeNull();expect(screen.getByTestId('database-wal-history').querySelector('svg')).not.toBeNull();
 const rows=()=>within(card).getAllByRole('row');expect(rows()).toHaveLength(3);
 const range=screen.getByRole('group',{name:'History period'});const buttons=within(range).getAllByRole('button');
 fireEvent.click(buttons[0]!);expect(rows()).toHaveLength(2);expect(buttons[0]).toHaveAttribute('aria-pressed','true');
 fireEvent.click(screen.getByRole('button',{name:'All available'}));expect(rows()).toHaveLength(4);
 fireEvent.click(within(card).getByRole('button',{name:'View as table'}));expect(card.querySelector('[data-chart-table]')).toHaveAttribute('data-chart-table','shown');
});
it('spaces irregular samples by time and makes small size differences visible',()=>{
 render(()=><DatabaseSizeHistory samples={samples}/>);fireEvent.click(screen.getByRole('button',{name:'All available'}));
 const points=screen.getByTestId('database-size-history').querySelector('polyline')!.getAttribute('points')!.split(' ').map(p=>p.split(',').map(Number));
 const wal=screen.getByTestId('database-wal-history').querySelector('polyline')!.getAttribute('points')!.split(' ').map(p=>p.split(',').map(Number));expect(wal[1]![0]).toBe(75);
 expect(points[1]![0]).toBe(75);expect(Math.abs(points[0]![1]!-points[2]![1]!)).toBeGreaterThan(20);
});
it('does not invent measurements when no history exists',()=>{
 render(()=><DatabaseSizeHistory samples={[]}/>);
 expect(screen.getByTestId('database-size-history').querySelector('svg')).toBeNull();
 expect(screen.getByTestId('database-wal-history').querySelector('svg')).toBeNull();
});

it('retains generation boundaries as separated graph segments and exposes reusable space',()=>{
 render(()=><DatabaseSizeHistory samples={[
  {at:1000,dbBytes:10000,walBytes:100,occupiedBytes:9000,freeBytes:1000,segment:1},
  {at:2000,dbBytes:10000,walBytes:200,occupiedBytes:7000,freeBytes:3000,segment:1},
  {at:3000,dbBytes:4000,walBytes:0,occupiedBytes:3000,freeBytes:1000,segment:2,breakReason:'generation'},
  {at:4000,dbBytes:4000,walBytes:100,occupiedBytes:2000,freeBytes:2000,segment:2},
 ]}/>);
 fireEvent.click(screen.getByRole('button',{name:'All available'}));
 const card=screen.getByTestId('database-size-history');
 expect(within(card).getByRole('columnheader',{name:'Reusable space'})).toBeInTheDocument();
 expect(within(card).getByRole('columnheader',{name:'Segment'})).toBeInTheDocument();
 expect(within(card).getAllByRole('row')).toHaveLength(5);
 const lines=card.querySelectorAll('polyline');expect(lines).toHaveLength(4);
 for(const line of lines)expect(line.getAttribute('points')!.split(' ')).toHaveLength(2);
 const svg=card.querySelector('svg')!;
 svg.getBoundingClientRect=()=>({x:0,y:0,left:0,top:0,right:300,bottom:48,width:300,height:48,toJSON:()=>({})});
 fireEvent(svg,new MouseEvent('pointermove',{clientX:200,bubbles:true}));
 const tip=card.querySelector('[data-chart-tip]')!;
 expect([...tip.querySelectorAll('strong')].map(node=>node.textContent)).toEqual([formatBytes(4000),formatBytes(3000)]);
 expect(screen.getByTestId('database-free-history').querySelectorAll('polyline')).toHaveLength(2);
 expect(screen.getByText(/Growth is not calculated across these boundaries/)).toBeVisible();
});

it('uses the latest observation for its default range after a backwards clock correction',()=>{
 render(()=><DatabaseSizeHistory samples={[
  {at:100000,dbBytes:10000,walBytes:0,segment:1},
  {at:1000,dbBytes:4000,walBytes:0,segment:2,breakReason:'clock'},
  {at:2000,dbBytes:4000,walBytes:0,segment:2},
 ]}/>);
 const card=screen.getByTestId('database-size-history');
 expect(within(card).getAllByRole('row')).toHaveLength(3);
 fireEvent.click(screen.getByRole('button',{name:'All available'}));
 expect(within(card).getAllByRole('row')).toHaveLength(4);
});
