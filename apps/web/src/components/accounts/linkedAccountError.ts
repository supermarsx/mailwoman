import {ApiError} from '../../api/client.ts';
import {t} from '../../i18n/index.ts';
export function linkedAccountError(error:unknown):string {
  const keys:Record<string,string>={linkingDisabled:'linked-accounts-disabled',linkedAccountLimit:'linked-accounts-limit',linkedAccountReauthenticationRequired:'linked-accounts-reauth',linkedAccountEnrollmentRequired:'linked-accounts-enrollment-required',linkedAccountPasswordChangeRequired:'linked-accounts-password-change-required',linkedChallengeExpired:'linked-accounts-challenge-expired',linkedBackendUnsupported:'linked-accounts-backend-unsupported',signinChallengeRequired:'linked-accounts-verification-required',signinChallengeRejected:'linked-accounts-verification-required'};
  return t(error instanceof ApiError&&error.code?(keys[error.code]??'linked-accounts-error'):'linked-accounts-error');
}
