import type {FailureReason} from './recovery.ts';
/** What a refused answer says about itself, besides its status. */
export interface ApiErrorDetails {
  /** The server's machine code (`accountDisabled`, `signinNotAllowed`, …). */
  code?: string;
  /** The answer's `X-Request-Id`, shown as "Reference: …" for a support request. */
  requestId?: string;
  retryAfterSecs?: number;
}

export class ApiError extends Error {
  readonly status: number;
  readonly code: string | undefined;
  readonly requestId: string | undefined;
  readonly retryAfterSecs: number | undefined;
  constructor(status: number, message: string, details: ApiErrorDetails = {}) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = details.code;
    this.requestId = details.requestId;
    this.retryAfterSecs = details.retryAfterSecs;
  }
}

/** The application response is known, but browser cookie delivery failed. The
 * original action must never be replayed as transport recovery. */
export class ActionCompletionError extends ApiError {
  readonly actionResponse:Response;
  constructor(response:Response){
    super(409,'The action completed, but sign-in delivery failed. Sign in again before continuing.',{code:'actionCompletedTransportFailed'});
    this.name='ActionCompletionError';this.actionResponse=response;
  }
}
/** Codes survive the worker boundary as ordinary ApiError instances. */
export function terminalActionError(error:unknown):boolean{
  return error instanceof ApiError&&(error.code==='outcomeUnknown'||error.code==='actionCompletedTransportFailed'||error.code==='actionNotDispatched'||error.code==='actionNotDispatchedBusy');
}

/** Raised when the request never reached the server (offline / dropped). */
export class NetworkError extends Error {
  /** t91: why the recovery layer gave up, when it was the one that did. */
  readonly reason: FailureReason | undefined;
  constructor(message: string, reason?: FailureReason) {
    super(message);
    this.name = 'NetworkError';
    this.reason = reason;
  }
}

