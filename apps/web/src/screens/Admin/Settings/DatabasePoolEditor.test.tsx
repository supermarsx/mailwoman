import {expect,it} from 'vitest';
import {createSignal} from 'solid-js';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {ValueInput} from './ValueInput.tsx';
import {parseDraft,type Draft} from './value.ts';
import {DEFAULT_DATABASE_POOL,normalizeDatabasePool} from './databasePool.ts';
import {expectNoAxeViolations,expectNoInlineStyle} from '../shell/uxTestkit.ts';

it('edits connection limits and timings in guided groups without exposing JSON',async()=>{
 const [draft,setDraft]=createSignal<Draft>('{}');
 const {container}=render(()=><ValueInput kind={{type:'databasePool',restartRequired:true}} draft={draft()} onDraft={setDraft} wiring={{id:'pool',describedBy:undefined,invalid:false}}/>);
 expect(screen.getByRole('group',{name:'Connections'})).toBeVisible();expect(screen.getByRole('group',{name:'Timeouts'})).toBeVisible();expect(screen.getByRole('group',{name:'PostgreSQL budget'})).toBeVisible();
 expect(container.querySelector('textarea')).toBeNull();expect(screen.getByText(/Changing ceilings or timings requires a full server restart/)).toBeVisible();
 fireEvent.change(screen.getByRole('combobox',{name:'SQLite maximum connections'}),{target:{value:'custom'}});
 fireEvent.input(screen.getByRole('spinbutton',{name:'SQLite maximum connections'}),{target:{value:'24'}});
 fireEvent.input(screen.getByRole('spinbutton',{name:'Minimum connections'}),{target:{value:'4'}});
 fireEvent.input(screen.getByRole('spinbutton',{name:'Wait for a connection (milliseconds)'}),{target:{value:'500'}});
 const parsed=parseDraft({type:'databasePool'},draft(),false);expect(parsed).toMatchObject({ok:true,value:{...DEFAULT_DATABASE_POOL,sqliteMaxConnections:24,minimumConnections:4,acquireTimeoutMs:500}});
 fireEvent.change(screen.getByRole('combobox',{name:'SQLite maximum connections'}),{target:{value:'auto'}});
 expect(JSON.parse(String(draft())).sqliteMaxConnections).toBeNull();expect(screen.queryByRole('spinbutton',{name:'SQLite maximum connections'})).toBeNull();
 await expectNoAxeViolations(container);expectNoInlineStyle(container);
});
it('keeps incomplete numeric edits visible and invalid instead of silently selecting Auto',()=>{
 const [draft,setDraft]=createSignal<Draft>(JSON.stringify({...DEFAULT_DATABASE_POOL,sqliteMaxConnections:8}));
 render(()=><ValueInput kind={{type:'databasePool'}} draft={draft()} onDraft={setDraft} wiring={{id:'pool',describedBy:undefined,invalid:false}}/>);
 fireEvent.input(screen.getByRole('spinbutton',{name:'SQLite maximum connections'}),{target:{value:''}});
 expect(JSON.parse(String(draft())).sqliteMaxConnections).toBe('');expect(screen.getByRole('combobox',{name:'SQLite maximum connections'})).toHaveValue('custom');expect(parseDraft({type:'databasePool'},draft(),false).ok).toBe(false);
});
it('honours disabled policy controls',()=>{
 render(()=><ValueInput kind={{type:'databasePool'}} draft="{}" onDraft={()=>{throw new Error('read only');}} disabled wiring={{id:'pool',describedBy:undefined,invalid:false}}/>);
 for(const control of [...screen.getAllByRole('spinbutton'),...screen.getAllByRole('combobox')])expect(control).toBeDisabled();
});
it('fills missing fields from Rust-compatible defaults and rejects unknown fields or nonobjects',()=>{
 expect(normalizeDatabasePool({})).toEqual(DEFAULT_DATABASE_POOL);expect(normalizeDatabasePool({minimumConnections:0})).toEqual({...DEFAULT_DATABASE_POOL,minimumConnections:0});
 for(const value of [null,[],true,{unknown:1},{minimumConnections:null},{sqliteMaxConnections:''},{minimumConnections:1.1},{replicaCount:'2'}])expect(normalizeDatabasePool(value),JSON.stringify(value)).toBeNull();
});
it.each([
 ['minimumConnections',-1],['minimumConnections',1025],['sqliteMaxConnections',0],['sqliteMaxConnections',33],['postgresMaxConnections',1025],['acquireTimeoutMs',99],['acquireTimeoutMs',120001],['idleTimeoutSeconds',0],['idleTimeoutSeconds',3601],['maxLifetimeSeconds',29],['maxLifetimeSeconds',86401],['postgresConnectionBudget',0],['postgresConnectionBudget',1000001],['postgresReservedConnections',-1],['replicaCount',0],['replicaCount',10001],
])('validates the server bound for %s=%s',(key,value)=>{expect(normalizeDatabasePool({...DEFAULT_DATABASE_POOL,[key]:value})).toBeNull();});
it('enforces explicit ceilings and the shared PostgreSQL replica budget',()=>{
 for(const value of [{minimumConnections:8,sqliteMaxConnections:4},{minimumConnections:8,postgresMaxConnections:4},{postgresConnectionBudget:20,postgresReservedConnections:20},{postgresConnectionBudget:2,postgresReservedConnections:1,replicaCount:2},{minimumConnections:2,postgresConnectionBudget:2,postgresReservedConnections:1}])expect(normalizeDatabasePool(value),JSON.stringify(value)).toBeNull();
 expect(normalizeDatabasePool({...DEFAULT_DATABASE_POOL,minimumConnections:0,sqliteMaxConnections:32,postgresMaxConnections:1024,acquireTimeoutMs:120000,idleTimeoutSeconds:3600,maxLifetimeSeconds:86400,postgresConnectionBudget:1000000,postgresReservedConnections:0,replicaCount:10000})).not.toBeNull();
});

it('keeps the cache opt-in, validates the aggregate bound and restores unchanged driver behavior',()=>{
 const [draft,setDraft]=createSignal<Draft>('{}');
 render(()=><ValueInput kind={{type:'databasePool'}} draft={draft()} onDraft={setDraft} wiring={{id:'pool',describedBy:undefined,invalid:false}}/>);
 const select=screen.getByRole('combobox',{name:'SQLite foreground cache target (MiB)'});
 expect(select).toHaveValue('driver');expect(screen.queryByRole('spinbutton',{name:'SQLite foreground cache target (MiB)'})).toBeNull();
 fireEvent.change(select,{target:{value:'custom'}});
 expect(JSON.parse(String(draft())).sqlitePageCacheMiB).toBe(256);
 const input=screen.getByRole('spinbutton',{name:'SQLite foreground cache target (MiB)'});
 fireEvent.input(input,{target:{value:''}});expect(parseDraft({type:'databasePool'},draft(),false).ok).toBe(false);
 fireEvent.input(input,{target:{value:'16'}});expect(parseDraft({type:'databasePool'},draft(),false)).toMatchObject({ok:true,value:{sqlitePageCacheMiB:16}});
 fireEvent.change(select,{target:{value:'driver'}});expect(JSON.parse(String(draft())).sqlitePageCacheMiB).toBeNull();
 for(const value of [0,15,1025,16.5,'256'])expect(normalizeDatabasePool({sqlitePageCacheMiB:value})).toBeNull();
 for(const value of [null,16,256,1024])expect(normalizeDatabasePool({sqlitePageCacheMiB:value})).not.toBeNull();
 expect(screen.getByText(/not a process memory limit/)).toBeVisible();
});
