/** Mirrors mw_store::pool_policy::PoolPolicy; the server validates again. */
export interface DatabasePoolPolicy {
 minimumConnections:number;sqliteMaxConnections:number|null;postgresMaxConnections:number|null;
 sqlitePageCacheMiB:number|null;
 acquireTimeoutMs:number;idleTimeoutSeconds:number;maxLifetimeSeconds:number;
 postgresConnectionBudget:number;postgresReservedConnections:number;replicaCount:number;
}
export const DEFAULT_DATABASE_POOL:Readonly<DatabasePoolPolicy>={minimumConnections:1,sqliteMaxConnections:null,sqlitePageCacheMiB:null,postgresMaxConnections:null,acquireTimeoutMs:30000,idleTimeoutSeconds:60,maxLifetimeSeconds:1800,postgresConnectionBudget:100,postgresReservedConnections:20,replicaCount:1};
export function normalizeDatabasePool(value:unknown):DatabasePoolPolicy|null{
 if(!value||typeof value!=='object'||Array.isArray(value))return null;
 if(Object.keys(value).some(key=>!Object.hasOwn(DEFAULT_DATABASE_POOL,key)))return null;
 const candidate={...DEFAULT_DATABASE_POOL,...value} as Record<keyof DatabasePoolPolicy,unknown>;
 const integer=(key:keyof DatabasePoolPolicy,min:number,max:number)=>typeof candidate[key]==='number'&&Number.isSafeInteger(candidate[key])&&Number(candidate[key])>=min&&Number(candidate[key])<=max;
 if(!integer('minimumConnections',0,1024)||!(candidate.sqliteMaxConnections===null||integer('sqliteMaxConnections',1,32))||!(candidate.postgresMaxConnections===null||integer('postgresMaxConnections',1,1024))||!integer('acquireTimeoutMs',100,120000)||!integer('idleTimeoutSeconds',1,3600)||!integer('maxLifetimeSeconds',30,86400)||!integer('postgresConnectionBudget',1,1000000)||!integer('postgresReservedConnections',0,999999)||!integer('replicaCount',1,10000))return null;
 const pool=candidate as unknown as DatabasePoolPolicy;
 if(!(candidate.sqlitePageCacheMiB===null||integer('sqlitePageCacheMiB',16,1024)))return null;
 if(pool.postgresReservedConnections>=pool.postgresConnectionBudget)return null;
 if((pool.sqliteMaxConnections!==null&&pool.minimumConnections>pool.sqliteMaxConnections)||(pool.postgresMaxConnections!==null&&pool.minimumConnections>pool.postgresMaxConnections))return null;
 const budget=Math.floor((pool.postgresConnectionBudget-pool.postgresReservedConnections)/pool.replicaCount);
 return budget>=1&&pool.minimumConnections<=budget?pool:null;
}
