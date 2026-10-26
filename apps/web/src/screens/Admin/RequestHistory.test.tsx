import {beforeEach,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {RequestHistory} from './RequestHistory.tsx';
import {loadRequestHistory,type RequestHistory as History} from '../../api/admin/requestHistory.ts';
vi.mock('../../api/admin/requestHistory.ts',()=>({loadRequestHistory:vi.fn()}));
const sample=():History=>({from:1000,to:4600,bucketSeconds:60,series:[{t:1000,requests:2,errors5xx:0,p50Ms:5,p95Ms:10,coverage:'complete'},{t:1060,requests:null,errors5xx:null,p50Ms:null,p95Ms:null,coverage:'unavailable'}],coverage:{observedSeconds:60,requestedSeconds:3600,gaps:[{from:1060,to:4600}]},policy:{defaultRangeSeconds:3600,maxRangeSeconds:2592000,retentionDays:30},scope:'server',timing:'response-ready'});
beforeEach(()=>{vi.mocked(loadRequestHistory).mockReset();vi.mocked(loadRequestHistory).mockResolvedValue(sample());});
it('loads the admin default and requests a fresh six-hour window from the selector',async()=>{
 render(()=><RequestHistory/>);
 expect(await screen.findByText(/History covers 1 of 60/)).toBeInTheDocument();
 expect(loadRequestHistory).toHaveBeenCalledWith(undefined,expect.any(AbortSignal));
 fireEvent.change(screen.getByLabelText('Period'),{target:{value:'21600'}});
 await waitFor(()=>expect(loadRequestHistory).toHaveBeenCalledTimes(2));
 const range=vi.mocked(loadRequestHistory).mock.calls[1]![0]!;
 expect(range.to-range.from).toBe(21600);
 expect(screen.getByRole('region',{name:'Requests per bucket'})).toHaveTextContent('Not available');
});
it('validates custom dates before requesting them and respects the returned max range',async()=>{
 vi.mocked(loadRequestHistory).mockResolvedValue({...sample(),policy:{defaultRangeSeconds:3600,maxRangeSeconds:21600,retentionDays:1}});
 render(()=><RequestHistory/>);
 await screen.findByText(/History covers/);
 expect(screen.queryByRole('option',{name:'Last 30 days'})).not.toBeInTheDocument();
 fireEvent.change(screen.getByLabelText('Period'),{target:{value:'custom'}});
 fireEvent.input(screen.getByLabelText('From (local time)'),{target:{value:'2026-01-02T12:00'}});
 fireEvent.input(screen.getByLabelText('To (local time)'),{target:{value:'2026-01-01T12:00'}});
 fireEvent.click(screen.getByRole('button',{name:'Refresh request history'}));
 expect(await screen.findByRole('alert')).toHaveTextContent('The end must be later');
 expect(loadRequestHistory).toHaveBeenCalledTimes(1);
});
it('offers retry for a failed history request rather than an unthemed error boundary',async()=>{
 vi.mocked(loadRequestHistory).mockRejectedValueOnce(new Error('unavailable'));
 render(()=><RequestHistory/>);
 expect(await screen.findByRole('alert')).toHaveTextContent('could not be loaded');
 fireEvent.click(screen.getByRole('button',{name:'Refresh request history'}));
 await screen.findByText(/History covers/);
 expect(screen.queryByRole('alert')).not.toBeInTheDocument();
});
