/** A user agent describes a client, not a verified physical device. */
export function describeSessionDevice(agent?:string|null):{browser:string;os:string;device:'desktop'|'phone'|'tablet'|'unknown'} {
  const ua=(agent??'').slice(0,4096);
  const clients:[string,RegExp][]=[['Microsoft Edge',/Edg(?:e|A|iOS)?\/([\d.]+)/],['Opera',/(?:OPR|Opera)\/([\d.]+)/],['Samsung Internet',/SamsungBrowser\/([\d.]+)/],['Firefox',/(?:Firefox|FxiOS)\/([\d.]+)/],['Thunderbird',/Thunderbird\/([\d.]+)/],['Chrome',/(?:Chrome|CriOS)\/([\d.]+)/],['Safari',/Version\/([\d.]+).*Safari\//]];
  let browser='';
  for(const [name,pattern] of clients){const match=pattern.exec(ua);if(match){browser=`${name} ${match[1]!.split('.')[0]}`;break;}}
  // Windows NT 10.0 is also reported by Windows 11: do not invent a version.
  const android=/Android(?: ([\d.]+))?/.exec(ua);
  const os=/Windows NT/.test(ua)?'Windows':android?`Android${android[1]?` ${android[1]}`:''}`:/iPhone|iPad|iPod/.test(ua)?'iOS':/CrOS/.test(ua)?'ChromeOS':/Macintosh|Mac OS X/.test(ua)?'macOS':/Linux/.test(ua)?'Linux':'';
  const device=/iPad|Tablet/.test(ua)||(/Android/.test(ua)&&!/Mobile/.test(ua))?'tablet':/Mobile|iPhone|iPod/.test(ua)?'phone':os?'desktop':'unknown';
  return {browser,os,device};
}

/** Normalize nanosecond server timestamps for all supported browsers. */
export function sessionDate(value?:string|null):Date|null {
  if(!value)return null;
  const date=new Date(value.replace(/(\.\d{3})\d+(?=Z|[+-]\d{2}:?\d{2}$)/,'$1'));
  return Number.isFinite(date.getTime())?date:null;
}
