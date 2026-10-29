import {beforeEach,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {RequestHistory} from './RequestHistory.tsx';
import {loadRequestHistory,loadNotFoundPaths,type RequestHistory as History,type NotFoundPaths} from '../../api/admin/requestHistory.ts';
import {AdminProblemError} from './shell/adminApi.ts';
vi.mock('../../api/admin/requestHistory.ts',()=>({loadRequestHistory:vi.fn(),loadNotFoundPaths:vi.fn()}));
const sample=():History=>({from:1000,to:4600,bucketSeconds:60,series:[{t:1000,requests:2,errors5xx:0,p50Ms:5,p95Ms:10,coverage:'complete'},{t:1060,requests:null,errors5xx:null,p50Ms:null,p95Ms:null,coverage:'unavailable'}],coverage:{observedSeconds:60,requestedSeconds:3600,gaps:[{from:1060,to:4600}]},policy:{defaultRangeSeconds:3600,maxRangeSeconds:2592000,retentionDays:30},scope:'server',timing:'response-ready'});
const noPaths=():NotFoundPaths=>({from:1000,to:4600,pathsFrom:0,pathsTo:4600,bucketSeconds:3600,limit:50,total:0,approximate:false,paths:[],coverage:{observedSeconds:3600,trackedSince:0,notTracked:[]},scope:'server'});
beforeEach(()=>{vi.mocked(loadRequestHistory).mockReset();vi.mocked(loadRequestHistory).mockResolvedValue(sample());vi.mocked(loadNotFoundPaths).mockReset();vi.mocked(loadNotFoundPaths).mockResolvedValue(noPaths());});
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

it('labels legacy precision without fabricating finer measurements',async()=>{
 render(()=><RequestHistory/>);
 expect(await screen.findByText(/original measurement precision cannot be recovered/)).toBeInTheDocument();
});
it('shows fine integer-ms values and omits the legacy warning for current rows',async()=>{
 const value=sample();value.series[0]={...value.series[0]!,p50Ms:37,p95Ms:83,legacyLatencyEstimate:false,p50ResolutionMs:1,p95ResolutionMs:1};
 vi.mocked(loadRequestHistory).mockResolvedValue(value);
 render(()=><RequestHistory/>);
 await screen.findByText(/History covers/);
 expect(screen.queryByText(/original measurement precision cannot be recovered/)).not.toBeInTheDocument();
 expect(screen.getByRole('region',{name:'Response time, 95th percentile'})).toHaveTextContent('83');
});

it('renders fractional milliseconds from the server without losing the resolution metadata',async()=>{
 const value=sample();value.latencySampleUnit='microseconds';value.series[0]={...value.series[0]!,p50Ms:0.123,p95Ms:0.999,legacyLatencyEstimate:false,p50ResolutionMs:0.001,p95ResolutionMs:0.001};
 vi.mocked(loadRequestHistory).mockResolvedValue(value);
 render(()=><RequestHistory/>);await screen.findByText(/History covers/);
 expect(screen.getByTestId('spark-p95')).toHaveTextContent('999 μs');
 expect(screen.getByRole('region',{name:'Response time, median'})).toHaveTextContent('123 μs');
 expect(screen.getByTestId('spark-p95')).not.toHaveTextContent(/0 ms/);
});

const tracked=():History=>{const value=sample();value.series=[
 {t:1000,requests:40,errors5xx:0,notFound:4,notFoundCoverage:'complete',p50Ms:5,p95Ms:10,coverage:'complete'},
 {t:1060,requests:10,errors5xx:0,notFound:null,notFoundCoverage:'notTracked',p50Ms:5,p95Ms:10,coverage:'complete'},
 {t:1120,requests:60,errors5xx:0,notFound:6,notFoundCoverage:'complete',p50Ms:5,p95Ms:10,coverage:'complete'}];
 value.coverage.notFound={observedSeconds:120,trackedSince:1000,notTracked:[{from:1060,to:1120}]};return value;};
it('shows the 404 tile with its share and a 404 chart that marks untracked buckets as not available',async()=>{
 vi.mocked(loadRequestHistory).mockResolvedValue(tracked());
 render(()=><RequestHistory/>);
 const tile=await screen.findByTestId('tile-not-found');
 expect(tile).toHaveTextContent('Not found (404) in this period');
 expect(tile).toHaveTextContent('10');
 expect(tile).toHaveTextContent('10% of requests');
 const chart=screen.getByTestId('spark-not-found');
 expect(chart).toHaveTextContent('Not found (404) per bucket');
 const cells=[...chart.querySelectorAll('tbody td:nth-child(2)')].map(cell=>cell.textContent);
 expect(cells).toEqual(['4','Not available','6']);
 expect(screen.getByTestId('not-found-not-tracked')).toHaveTextContent('404s were not tracked before');
});
it('never presents a period without 404 tracking as zero',async()=>{
 const value=sample();value.series=value.series.map(p=>({...p,notFound:null,notFoundCoverage:p.coverage==='unavailable'?'unavailable':'notTracked'}));
 value.coverage.notFound={observedSeconds:0,trackedSince:null,notTracked:[{from:1000,to:1060}]};
 vi.mocked(loadRequestHistory).mockResolvedValue(value);
 render(()=><RequestHistory/>);
 const tile=await screen.findByTestId('tile-not-found');
 expect(tile).toHaveTextContent('Not available');
 expect(screen.getByTestId('not-found-not-tracked')).toHaveTextContent('404s were not tracked during this period.');
});
it('lists the redacted top not-found paths for the same window, marking bounded counts',async()=>{
 vi.mocked(loadRequestHistory).mockResolvedValue(tracked());
 vi.mocked(loadNotFoundPaths).mockResolvedValue({...noPaths(),total:10,approximate:true,paths:[
  {method:'GET',path:'/wp-login.php',count:7,minCount:7},{method:'POST',path:'/u/{email}/inbox',count:3,minCount:1}]});
 render(()=><RequestHistory/>);
 const table=await screen.findByTestId('not-found-paths');
 expect(table).toHaveTextContent('Most frequent not-found paths');
 const rows=[...table.querySelectorAll('tbody tr')].map(row=>[...row.querySelectorAll('td')].map(cell=>cell.textContent));
 expect(rows).toEqual([['GET','/wp-login.php','7'],['POST','/u/{email}/inbox','1 to 3']]);
 expect(table).toHaveTextContent('some counts are a range');
 fireEvent.change(screen.getByLabelText('Period'),{target:{value:'604800'}});
 await waitFor(()=>expect(loadNotFoundPaths).toHaveBeenCalledTimes(2));
 const [history,paths]=[vi.mocked(loadRequestHistory).mock.calls[1]![0]!,vi.mocked(loadNotFoundPaths).mock.calls[1]![0]!];
 expect(paths).toEqual(history);
 expect(paths.to-paths.from).toBe(604800);
});
it('hides the paths table for a reader without Observability access and keeps the counts',async()=>{
 vi.mocked(loadRequestHistory).mockResolvedValue(tracked());
 vi.mocked(loadNotFoundPaths).mockRejectedValue(new AdminProblemError(403,{},null));
 render(()=><RequestHistory/>);
 await screen.findByTestId('tile-not-found');
 expect(screen.queryByTestId('not-found-paths')).not.toBeInTheDocument();
 expect(screen.queryByText(/Not-found paths could not be loaded/)).not.toBeInTheDocument();
});
