import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';

export type MailImportFormat = 'eml' | 'mbox' | 'maildir';
export type MailExportFormat = 'eml' | 'mbox' | 'txt' | 'md' | 'msg' | 'docx';
export interface MailImportFormats {engineMode:boolean;available:boolean;maxBytes:number;maxMessages:number}
export interface MailImportResult {imported:number;failed:number;total:number;mailboxId:string}
export class MailTransferError extends Error {
  constructor(readonly status:number) {super(`mail transfer failed (${status})`);}
}
const contentTypes:Record<MailImportFormat,string> = {eml:'message/rfc822',mbox:'application/mbox',maildir:'application/x-tar'};
export interface MailTransferApi {
  formats(signal:AbortSignal):Promise<MailImportFormats>;
  import(format:MailImportFormat,file:File,mailboxId:string,signal:AbortSignal):Promise<MailImportResult>;
  export(emailId:string,format:MailExportFormat,signal:AbortSignal):Promise<Blob>;
}
async function jsonResponse<T>(response:Response):Promise<T> {
  if(!response.ok)throw new MailTransferError(response.status);
  return response.json() as Promise<T>;
}
export const mailTransferApi:MailTransferApi = {
  async formats(signal) {
    return jsonResponse(await apiFetch(withBase('/api/import/formats'),{credentials:'same-origin',signal}));
  },
  async import(format,file,mailboxId,signal) {
    if(!mailboxId)throw new Error('a destination mailbox is required');
    return jsonResponse(await apiFetch(withBase(`/api/import/${format}?mailboxId=${encodeURIComponent(mailboxId)}`),{
      method:'POST',credentials:'same-origin',headers:{'content-type':contentTypes[format]},body:file,signal,
    }));
  },
  async export(emailId,format,signal) {
    if(!emailId)throw new Error('a message is required');
    const response=await apiFetch(withBase(`/api/export/${encodeURIComponent(emailId)}?format=${format}`),{credentials:'same-origin',signal});
    if(!response.ok)throw new MailTransferError(response.status);
    return response.blob();
  },
};
