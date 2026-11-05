// Minimal ZIP writer (method 0, "stored"): attachments are usually already compressed,
// and this keeps the client free of a compression dependency.
const TABLE=(()=>{const table=new Uint32Array(256);for(let n=0;n<256;n++){let c=n;for(let k=0;k<8;k++)c=c&1?0xedb88320^(c>>>1):c>>>1;table[n]=c>>>0;}return table;})();
export function crc32(data:Uint8Array):number {let c=0xffffffff;for(let i=0;i<data.length;i++)c=TABLE[(c^data[i]!)&0xff]!^(c>>>8);return (c^0xffffffff)>>>0;}
export interface ZipEntry {name:string;data:Uint8Array}
/** Build a ZIP archive; entry names are UTF-8 and may contain `/` for folders. */
export function buildZip(entries:ZipEntry[]):Blob {
 const enc=new TextEncoder(),parts:BlobPart[]=[],central:Uint8Array[]=[];let offset=0;
 for(const entry of entries){
  const name=enc.encode(entry.name),crc=crc32(entry.data);
  const local=new DataView(new ArrayBuffer(30));
  local.setUint32(0,0x04034b50,true);local.setUint16(4,20,true);local.setUint16(6,0x0800,true);local.setUint16(8,0,true);local.setUint16(10,0,true);local.setUint16(12,0x21,true);
  local.setUint32(14,crc,true);local.setUint32(18,entry.data.length,true);local.setUint32(22,entry.data.length,true);local.setUint16(26,name.length,true);local.setUint16(28,0,true);
  parts.push(local.buffer,name as BlobPart,entry.data as BlobPart);
  const head=new DataView(new ArrayBuffer(46));
  head.setUint32(0,0x02014b50,true);head.setUint16(4,20,true);head.setUint16(6,20,true);head.setUint16(8,0x0800,true);head.setUint16(12,0,true);head.setUint16(14,0x21,true);
  head.setUint32(16,crc,true);head.setUint32(20,entry.data.length,true);head.setUint32(24,entry.data.length,true);head.setUint16(28,name.length,true);head.setUint32(42,offset,true);
  central.push(new Uint8Array(head.buffer),name);
  offset+=30+name.length+entry.data.length;
 }
 let size=0;for(const part of central){parts.push(part as BlobPart);size+=part.length;}
 const end=new DataView(new ArrayBuffer(22));
 end.setUint32(0,0x06054b50,true);end.setUint16(8,entries.length,true);end.setUint16(10,entries.length,true);end.setUint32(12,size,true);end.setUint32(16,offset,true);
 parts.push(end.buffer);
 return new Blob(parts,{type:'application/zip'});
}
