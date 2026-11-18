import {afterEach,expect,it,vi} from 'vitest';
import {createDraftAutosave} from './accountDraftAutosave.ts';
import {DraftSaveError,type DraftEditorState,type AccountDraftDocument} from '../../api/accountDrafts.ts';
it('treats reordered server JSON and absent optional MIME types as the same saved state',async()=>{
 vi.useFakeTimers();const status=vi.fn();
 const value={...state(),attachments:[{blobId:'owned.1',name:'file',size:2}]} as unknown as DraftEditorState;
 const returned=Object.fromEntries(Object.entries({...value,attachments:[{blobId:'owned.1',name:'file',size:2,contentType:null}]}).reverse()) as unknown as DraftEditorState;
 const save=vi.fn().mockResolvedValue(saved(returned));
 const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status,failure:vi.fn(),saved:vi.fn()});
 controller.update(value);await vi.advanceTimersByTimeAsync(1000);expect(status).toHaveBeenLastCalledWith('saved');
 controller.update(value);await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);controller.dispose();
});
const state=(subject='draft'):DraftEditorState=>({version:1,identityId:'',to:'unfinished',cc:'',bcc:'',subject,bodyFormat:'plain',bodyHtml:'',bodyText:'unsent',insertedSignatures:[],inReplyTo:[],references:[],attachments:[],inline:[],requestReceipt:false,requestDeliveryReceipt:false,openTracking:false,protectedContent:false});
const saved=(value:DraftEditorState,revision='r1'):AccountDraftDocument=>({id:'draft',accountId:'a',storageAuthority:'authority',revision,emailId:'email',savedAt:1,status:'saved',cleanupPending:false,summary:{subject:value.subject},state:value});
afterEach(()=>vi.useRealTimers());
it('rebases one explicit revision conflict while keeping the client draft contents',async()=>{
 const save=vi.fn().mockRejectedValueOnce(new DraftSaveError('draftConflict')).mockImplementationOnce(async(_id,_operation,_revision,value)=>saved(value,'r3'));
 const load=vi.fn(async()=>saved(state('Other device text'),'r2'));
 const controller=createDraftAutosave({id:'draft',transport:{save,load},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});
 controller.seedRevision('r1');controller.update(state('My offline edits'));expect(await controller.flush()).toBe(true);
 expect(load).toHaveBeenCalledWith('draft');expect(save.mock.calls[1]?.[2]).toBe('r2');expect(save.mock.calls[1]?.[3].subject).toBe('My offline edits');controller.dispose();
});
it('debounces edits and serializes a newer edit behind confirmed revision',async()=>{
 vi.useFakeTimers();let resolve!:(value:AccountDraftDocument)=>void;
 const save=vi.fn().mockImplementationOnce(()=>new Promise<AccountDraftDocument>(done=>resolve=done)).mockImplementation(async(_id,_op,_rev,value)=>saved(value,'r2'));
 const status=vi.fn();const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status,failure:vi.fn(),saved:vi.fn(),operation:()=> 'operation'});
 controller.update(state('one'));await vi.advanceTimersByTimeAsync(700);controller.update(state('two'));await vi.advanceTimersByTimeAsync(1000);expect(save).toHaveBeenCalledTimes(1);
 controller.update(state('three'));resolve(saved(state('two')));await vi.advanceTimersByTimeAsync(0);await vi.advanceTimersByTimeAsync(1000);
 expect(save).toHaveBeenCalledTimes(2);expect(save.mock.calls[1]?.[2]).toBe('r1');expect(save.mock.calls[1]?.[3].subject).toBe('three');expect(status).toHaveBeenLastCalledWith('saved');controller.dispose();
});
it('never retries a network-uncertain APPEND even after later edits or manual save',async()=>{
 vi.useFakeTimers();const save=vi.fn().mockRejectedValue(new TypeError('connection lost'));const failure=vi.fn();
 const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status:vi.fn(),failure,saved:vi.fn()});controller.update(state());await vi.advanceTimersByTimeAsync(1000);controller.update(state('new'));await vi.advanceTimersByTimeAsync(60000);await controller.flush();
 expect(save).toHaveBeenCalledTimes(1);expect(failure).toHaveBeenCalledWith('draftSaveUncertain');controller.dispose();
});
it('periodic saving does not slide with every letter; manual mode never schedules',async()=>{
 vi.useFakeTimers();const save=vi.fn().mockImplementation(async(_id,_op,_rev,value)=>saved(value));const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});
 controller.configure({mode:'periodic',intervalSecs:5,debounceMs:500});controller.update(state('a'));await vi.advanceTimersByTimeAsync(4000);controller.update(state('ab'));await vi.advanceTimersByTimeAsync(1000);expect(save).toHaveBeenCalledTimes(1);
 controller.configure({mode:'manual',intervalSecs:5,debounceMs:500});controller.update(state('abc'));await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);await controller.flush();expect(save).toHaveBeenCalledTimes(2);controller.dispose();
});
it('drops late acknowledgement and scheduled writes after account authority changes',async()=>{
 vi.useFakeTimers();let current=true;let resolve!:(value:AccountDraftDocument)=>void;const callback=vi.fn();const save=vi.fn(()=>new Promise<AccountDraftDocument>(done=>resolve=done));
 const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>current,status:vi.fn(),failure:vi.fn(),saved:callback});controller.update(state());await vi.advanceTimersByTimeAsync(1000);current=false;resolve(saved(state()));await vi.advanceTimersByTimeAsync(0);expect(callback).not.toHaveBeenCalled();controller.update(state('other'));await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);controller.dispose();
});
it('preparation refusal reports failure without automatically looping',async()=>{
 vi.useFakeTimers();const save=vi.fn().mockRejectedValue(new DraftSaveError('draftPreparationRefused'));const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});controller.update(state());await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);controller.dispose();
});

it('suspends pending plaintext saves and waits for an already-started save before sending',async()=>{
 vi.useFakeTimers();let resolve!:(value:AccountDraftDocument)=>void;const save=vi.fn(()=>new Promise<AccountDraftDocument>(done=>resolve=done));
 const controller=createDraftAutosave({id:'draft',transport:{save},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});
 controller.update(state());controller.suspend(true);await vi.advanceTimersByTimeAsync(2000);expect(save).not.toHaveBeenCalled();
 controller.suspend(false);await vi.advanceTimersByTimeAsync(1000);expect(save).toHaveBeenCalledTimes(1);
 controller.update(state('edited while saving'));controller.suspend(true);let settled=false;void controller.settled().then(()=>settled=true);await vi.advanceTimersByTimeAsync(0);expect(settled).toBe(false);
 resolve(saved(state()));await vi.advanceTimersByTimeAsync(0);expect(settled).toBe(true);await vi.advanceTimersByTimeAsync(60000);expect(save).toHaveBeenCalledTimes(1);controller.dispose();
});
it('checks the uncertain save before uploading the latest exact editor state',async()=>{
 vi.useFakeTimers();const original=state('one'),latest={...state('two'),bodyText:'Every letter 🙂  \nNext line\n',to:'still typing@',cc:'x@',bodyHtml:'<p>Every letter 🙂  </p>'};
 const save=vi.fn().mockRejectedValueOnce(new TypeError('timeout')).mockImplementation(async(_id,_op,_revision,value)=>saved(value,'r2'));
 const load=vi.fn().mockResolvedValue(saved(original)),status=vi.fn();
 const controller=createDraftAutosave({id:'draft',transport:{save,load},current:()=>true,status,failure:vi.fn(),saved:vi.fn()});
 controller.update(original);await vi.advanceTimersByTimeAsync(1000);controller.update(latest);await vi.advanceTimersByTimeAsync(30000);
 expect(load).toHaveBeenCalledTimes(1);expect(save).toHaveBeenCalledTimes(2);expect(save.mock.calls[1]?.[2]).toBe('r1');expect(save.mock.calls[1]?.[3]).toEqual(latest);expect(status).toHaveBeenLastCalledWith('saved');controller.dispose();
});
it('keeps checking with backoff without duplicate APPENDs or repeated warning toasts',async()=>{
 vi.useFakeTimers();const save=vi.fn().mockRejectedValue(new TypeError('timeout')),load=vi.fn().mockResolvedValue({...saved(state()),status:'uncertain',revision:null}),failure=vi.fn();
 const controller=createDraftAutosave({id:'draft',transport:{save,load},current:()=>true,status:vi.fn(),failure,saved:vi.fn()});controller.update(state());await vi.advanceTimersByTimeAsync(1000);
 await vi.advanceTimersByTimeAsync(30000);expect(load).toHaveBeenCalledTimes(1);controller.update(state('newest'));await vi.advanceTimersByTimeAsync(59000);expect(load).toHaveBeenCalledTimes(1);await vi.advanceTimersByTimeAsync(1000);expect(load).toHaveBeenCalledTimes(2);expect(save).toHaveBeenCalledTimes(1);expect(failure).toHaveBeenCalledTimes(1);controller.dispose();
});
it('resumes a browser recovery copy by reconciling its server revision first',async()=>{
 vi.useFakeTimers();const save=vi.fn().mockImplementation(async(_id,_op,_rev,value)=>saved(value,'r3')),load=vi.fn().mockResolvedValue(saved(state('old'),'r2'));
 const controller=createDraftAutosave({id:'draft',transport:{save,load},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});controller.seedRevision('r1');controller.seedUncertain();controller.update(state('latest browser copy'));await vi.advanceTimersByTimeAsync(30000);
 expect(load).toHaveBeenCalledOnce();expect(save.mock.calls[0]?.[2]).toBe('r2');expect(save.mock.calls[0]?.[3].subject).toBe('latest browser copy');controller.dispose();
});
it('retries an initial request that never claimed a server draft',async()=>{
 vi.useFakeTimers();const save=vi.fn().mockRejectedValueOnce(new TypeError('offline')).mockImplementation(async(_id,_op,_rev,value)=>saved(value));const load=vi.fn().mockRejectedValue(new DraftSaveError('draftNotFound'));
 const controller=createDraftAutosave({id:'draft',transport:{save,load},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});controller.update(state());await vi.advanceTimersByTimeAsync(31000);expect(save).toHaveBeenCalledTimes(2);expect(save.mock.calls[1]?.[2]).toBeNull();controller.dispose();
});
it('cancels recovery polling while offline, disposed or outside its account authority',async()=>{
 vi.useFakeTimers();let current=true;const load=vi.fn().mockResolvedValue(saved(state())),save=vi.fn().mockRejectedValue(new TypeError('offline'));
 const controller=createDraftAutosave({id:'draft',transport:{save,load},current:()=>current,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});controller.update(state());await vi.advanceTimersByTimeAsync(1000);controller.suspend(true);await vi.advanceTimersByTimeAsync(300000);expect(load).not.toHaveBeenCalled();controller.suspend(false);current=false;await vi.advanceTimersByTimeAsync(300000);expect(load).not.toHaveBeenCalled();controller.dispose();
});

it('keeps letters typed while server confirmation is in flight',async()=>{
 vi.useFakeTimers();let confirm!:(document:AccountDraftDocument)=>void;const load=vi.fn(()=>new Promise<AccountDraftDocument>(resolve=>{confirm=resolve;}));
 const save=vi.fn().mockRejectedValueOnce(new TypeError('lost confirmation')).mockImplementation(async(_id,_op,_rev,value)=>saved(value,'r2'));
 const controller=createDraftAutosave({id:'draft',transport:{save,load},current:()=>true,status:vi.fn(),failure:vi.fn(),saved:vi.fn()});controller.update(state('old'));await vi.advanceTimersByTimeAsync(31000);
 const latest={...state('new'),bodyText:'Typed during the check 🙂  '};controller.update(latest);confirm(saved(state('old')));await vi.advanceTimersByTimeAsync(0);
 expect(save.mock.calls[1]?.[3]).toEqual(latest);controller.dispose();
});
