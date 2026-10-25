import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
export interface AdminTotpSetup {secret:string;otpauthUri:string}
export interface AdminTotpApi {
 status():Promise<{totpEnrolled:boolean;recoveryRemaining:number}>;
 begin():Promise<AdminTotpSetup>;
 confirm(code:string):Promise<{ok:boolean;recoveryCodes:string[]}>;
 remove():Promise<void>;
 rotate():Promise<{recoveryCodes:string[]}>;
}
export const adminTotpApi:AdminTotpApi={
 async status(){const [me,keys]=await Promise.all([adminRequest<{totpEnrolled:boolean}>('/me'),adminRequest<{recoveryRemaining:number}>('/me/passkeys')]);return{totpEnrolled:me.data.totpEnrolled,recoveryRemaining:keys.data.recoveryRemaining};},
 async begin(){return(await adminRequest<AdminTotpSetup>('/me/totp',{method:'POST',body:{}})).data;},
 async confirm(code){return(await adminRequest<{ok:boolean;recoveryCodes:string[]}>('/me/totp/confirm',{method:'POST',body:{code}})).data;},
 async remove(){await adminRequest('/me/totp',{method:'DELETE',body:{}});},
 async rotate(){return(await adminRequest<{recoveryCodes:string[]}>('/me/recovery',{method:'POST',body:{}})).data;},
};
