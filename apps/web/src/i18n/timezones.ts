/** Time-zone calculations never infer a person's location from an email address. */
export function validTimeZone(value: string): boolean {
  if (!value || value.length > 64 || !/^[A-Za-z_+-]+(?:\/[A-Za-z0-9_+-]+)*$/.test(value)) return false;
  try { new Intl.DateTimeFormat('en', { timeZone: value }).format(0); return true; } catch { return false; }
}
export function localTimeZone(): string { return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'; }
export function displayTimeZone(value: unknown): string {
  return typeof value === 'string' && value !== 'local' && validTimeZone(value) ? value : localTimeZone();
}

/** Resolve a wall-clock value without guessing through a DST gap or overlap.
 * Zero results means nonexistent/invalid; two means the repeated fall-back hour. */
export function wallTimeCandidates(value: string, zone: string): Date[] {
  if (!validTimeZone(zone)) return [];
  const match=/^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})(?::(\d{2}))?$/.exec(value);
  if (!match) return [];
  const wanted=match.slice(1).map(part=>Number(part??0));
  const [year,month,day,hour,minute,second]=wanted as [number,number,number,number,number,number];
  const seed=new Date(0);seed.setUTCFullYear(year,month-1,day);seed.setUTCHours(hour,minute,second,0);
  if (seed.getUTCFullYear()!==year||seed.getUTCMonth()!==month-1||seed.getUTCDate()!==day||seed.getUTCHours()!==hour||seed.getUTCMinutes()!==minute||seed.getUTCSeconds()!==second) return [];
  const formatter=new Intl.DateTimeFormat('en-US-u-ca-gregory-nu-latn',{timeZone:zone,year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',second:'2-digit',hourCycle:'h23'});
  function parts(instant: number): number[] {
    const found=Object.fromEntries(formatter.formatToParts(instant).map(part=>[part.type,part.value]));
    return ['year','month','day','hour','minute','second'].map(key=>Number(found[key]));
  }
  const offsets=new Set<number>();
  for (const hours of [-48,-24,0,24,48]) {
    const instant=seed.getTime()+hours*3600000;
    const [y,m,d,h,mi,s]=parts(instant) as [number,number,number,number,number,number];
    const local=new Date(0);local.setUTCFullYear(y,m-1,d);local.setUTCHours(h,mi,s,0);
    offsets.add(local.getTime()-instant);
  }
  return [...offsets].map(offset=>seed.getTime()-offset)
    .filter(instant=>parts(instant).every((part,index)=>part===wanted[index]))
    .sort((a,b)=>a-b).map(instant=>new Date(instant));
}
