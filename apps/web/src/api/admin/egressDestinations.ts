import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
export interface InternalDestination {id:string;trafficClass:string;destination:string;note:string;createdBy:string;createdAt:number}
export interface DestinationDraft {trafficClass:string;destination:string;note:string}
export interface EgressDestinationsApi {
  list():Promise<{destinations:InternalDestination[];eligibleClasses:string[]}>;
  create(draft:DestinationDraft):Promise<void>;
  remove(id:string):Promise<void>;
}
export const egressDestinationsApi:EgressDestinationsApi={
  async list(){return (await adminRequest<{destinations:InternalDestination[];eligibleClasses:string[]}>('/egress/destinations')).data;},
  async create(body){await adminRequest('/egress/destinations',{method:'POST',body});},
  async remove(id){await adminRequest(`/egress/destinations/${encodeURIComponent(id)}`,{method:'DELETE'});},
};
