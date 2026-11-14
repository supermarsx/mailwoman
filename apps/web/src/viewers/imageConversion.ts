import {apiFetch} from '../api/http.ts';
import {transportBase} from '../api/transport.ts';
import {filePreviews} from './filePreviewPreferences.ts';
/** Conversion uses the existing bounded, disposable render worker and WASM media jail. */
export async function convertImageData(data:string):Promise<string>{
 const raw=data.slice(data.indexOf(',')+1);if(!filePreviews().convertImages||!/^[A-Za-z0-9+/]*={0,2}$/.test(raw)||raw.length>5592408)throw new Error('Image conversion unavailable or input exceeds 4 MiB');
 const response=await apiFetch(`${transportBase()}/api/attachments/preview`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({format:'image',bytesBase64:raw})});if(!response.ok)throw new Error('Image conversion failed');
 const body=await response.json() as {preview?:{format?:unknown;attachments?:{mime?:unknown;dataBase64?:unknown;size?:unknown}[]}};const file=body.preview?.attachments?.[0];
 if(body.preview?.format!=='image'||file?.mime!=='image/png'||typeof file.dataBase64!=='string'||!file.dataBase64.startsWith('iVBORw0KGgo')||file.dataBase64.length>11184812||typeof file.size!=='number'||!Number.isInteger(file.size)||file.size<8||file.size>8388608||!/^[A-Za-z0-9+/]*={0,2}$/.test(file.dataBase64)||Math.floor(file.dataBase64.length*3/4)-(file.dataBase64.endsWith('==')?2:file.dataBase64.endsWith('=')?1:0)!==file.size)throw new Error('Invalid converted image');
 return `data:image/png;base64,${file.dataBase64}`;
}
