import {ApiError} from '../../api/client.ts';
import {t} from '../../i18n/index.ts';
import {failureText,isTransientFailure} from '../recoverableAction.ts';
import {errorReason} from '../../api/recovery.ts';
export function linkedAccountError(error:unknown):string {
  const keys:Record<string,string>={linkingDisabled:'linked-accounts-disabled',linkedAccountLimit:'linked-accounts-limit',linkedAccountReauthenticationRequired:'linked-accounts-reauth',linkedAccountMismatch:'linked-accounts-renew-mismatch',linkedPasswordRejected:'linked-accounts-renew-password-rejected',linkedServerUnreachable:'linked-accounts-renew-unreachable',linkedRenewLimit:'linked-accounts-renew-limit',linkedUsernameRequired:'linked-accounts-fields-required',linkedAccountEnrollmentRequired:'linked-accounts-enrollment-required',linkedAccountPasswordChangeRequired:'linked-accounts-password-change-required',linkedChallengeExpired:'linked-accounts-challenge-expired',linkedBackendUnsupported:'linked-accounts-backend-unsupported',signinChallengeRequired:'linked-accounts-verification-required',signinChallengeRejected:'linked-accounts-verification-required'};
  const known=error instanceof ApiError&&error.code?keys[error.code]:undefined;
  if(known)return t(known);
  // t91: a failure in transit says why (the form's button sends the same request, with the same key).
  if(isTransientFailure(error))return t('linked-accounts-failed-reason',{reason:failureText(errorReason(error))});
  return t('linked-accounts-error');
}
