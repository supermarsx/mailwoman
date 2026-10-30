import {describe,expect,it} from 'vitest';
import {render,screen,within} from '@solidjs/testing-library';
import {NotFoundPathsTable} from './NotFoundPaths.tsx';
import type {NotFoundPaths} from '../../api/admin/requestHistory.ts';

const base:NotFoundPaths={from:0,to:3600,pathsFrom:0,pathsTo:3600,bucketSeconds:3600,limit:50,total:12,approximate:false,
 coverage:{observedSeconds:3600,trackedSince:0,notTracked:[]},scope:'server',paths:[]};

describe('not-found paths by source (t102)',()=>{
 it('lists the app\'s own client misses apart from scanner traffic',()=>{
  render(()=><NotFoundPathsTable formatTime={String} value={{...base,paths:[
   {method:'GET',path:'/sw.js',count:7,minCount:7,source:'client'},
   {method:'GET',path:'/wp-login.php',count:4,minCount:4,source:'external'},
   {method:'GET',path:'/assets/Dashboard-CdQvJhSt.js',count:1,minCount:1,source:'client'},
  ]}}/>);
  const client=screen.getByTestId('not-found-client');
  expect(within(client).getByText('/sw.js')).toBeInTheDocument();
  expect(within(client).getByText('/assets/Dashboard-CdQvJhSt.js')).toBeInTheDocument();
  expect(within(client).queryByText('/wp-login.php')).toBeNull();
  expect(within(screen.getByTestId('not-found-external')).getByText('/wp-login.php')).toBeInTheDocument();
 });
 it('keeps one table for a server that does not send a source',()=>{
  render(()=><NotFoundPathsTable formatTime={String} value={{...base,paths:[{method:'GET',path:'/x',count:1,minCount:1}]}}/>);
  expect(screen.getByTestId('not-found-all')).toBeInTheDocument();
  expect(screen.queryByTestId('not-found-client')).toBeNull();
 });
});
