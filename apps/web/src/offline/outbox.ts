// Tab transport supplies tracking I/O; the worker uses the same pure drain core.
export * from './outboxCore.ts';
import {drainOutbox as drain,type OutboxStore,type DrainTracking,type DrainResult} from './outboxCore.ts';
import type {Client} from '../api/client.ts';
import {mintTrackingToken,bindTrackingToken} from '../api/tracking.ts';
const defaultTracking:DrainTracking={mint:mintTrackingToken,bind:bindTrackingToken};
export function drainOutbox(store:OutboxStore,client:Client,tracking:DrainTracking=defaultTracking,active:()=>boolean=()=>true,signal?:AbortSignal):Promise<DrainResult>{return drain(store,client,tracking,active,signal);}
