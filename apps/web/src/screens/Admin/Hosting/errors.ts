import { AdminProblemError } from '../shell/adminApi.ts';
import { describeAdminError } from '../shell/Feedback.tsx';
import { t } from '../../../i18n';
export function describeHostingError(error:unknown):string {
 if(error instanceof AdminProblemError && error.resultUncertain)return t('admin-hosting-result-uncertain');
 if(error instanceof AdminProblemError && ['unsupported-route','ssh-unavailable'].includes(error.code??''))return t('admin-hosting-route-unavailable');
 return describeAdminError(error);
}
