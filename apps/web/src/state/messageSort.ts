/** Server comparators, shared by every page of the active list/search. */
export function messageSort(order:unknown,group:unknown,timeZone='UTC'):{property:string;isAscending:boolean;timeZone?:string}[]{
  const received={property:'receivedAt',isAscending:false};
  const orders:Record<string,{property:string;isAscending:boolean}[]>={
    newest:[received],oldest:[{property:'receivedAt',isAscending:true}],
    sender:[{property:'from',isAscending:true},received],'sender-desc':[{property:'from',isAscending:false},received],
    subject:[{property:'subject',isAscending:true},received],'subject-desc':[{property:'subject',isAscending:false},received],
    'sent-newest':[{property:'sentAt',isAscending:false},received],'sent-oldest':[{property:'sentAt',isAscending:true},received],
    importance:[{property:'importance',isAscending:false},received],'importance-asc':[{property:'importance',isAscending:true},received],
    size:[{property:'size',isAscending:false},received],'size-asc':[{property:'size',isAscending:true},received],
  };
  const groups:Record<string,{property:string;isAscending:boolean;timeZone?:string}>={received:{property:'mailwomanGroupReceived',isAscending:false,timeZone},sent:{property:'mailwomanGroupSent',isAscending:false,timeZone},sender:{property:'mailwomanGroupFrom',isAscending:true},subject:{property:'mailwomanGroupSubject',isAscending:true},importance:{property:'importance',isAscending:false},size:{property:'mailwomanGroupSize',isAscending:false}};
  const comparators=typeof order==='string'?orders[order]??orders['newest']!:orders['newest']!;
  const primary=typeof group==='string'?groups[group]:undefined;
  return primary?[primary,...comparators]:comparators;
}
