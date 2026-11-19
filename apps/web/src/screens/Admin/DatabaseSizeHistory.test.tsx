import {expect,it} from 'vitest';
import {fireEvent,render,screen,within} from '@solidjs/testing-library';
import {DatabaseSizeHistory} from './DatabaseSizeHistory.tsx';
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
