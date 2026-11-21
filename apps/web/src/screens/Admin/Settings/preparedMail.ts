export interface PreparedMailConfig {enabled:boolean;budgetMiB:number;maxEntries:number;maxAccounts:number;idleSeconds:number}
export const DEFAULT_PREPARED_MAIL:Readonly<PreparedMailConfig>={enabled:false,budgetMiB:64,maxEntries:256,maxAccounts:128,idleSeconds:30};
export const PREPARED_MAIL_BOUNDS={budgetMiB:[16,16384],maxEntries:[1,100000],maxAccounts:[1,10000],idleSeconds:[1,3600]} as const;
export function validPreparedMail(value:unknown):value is PreparedMailConfig {
 if(!value||typeof value!=='object'||Array.isArray(value))return false;
 const row=value as Record<string,unknown>;
 return Object.keys(row).length===5&&typeof row.enabled==='boolean'&&Object.entries(PREPARED_MAIL_BOUNDS).every(([key,[min,max]])=>{const current=row[key];return typeof current==='number'&&Number.isSafeInteger(current)&&current>=min&&current<=max;});
}
