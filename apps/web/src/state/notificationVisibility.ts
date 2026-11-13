import {settings} from '../api/settings.ts';

/** View visibility does not disable security notices, mail toasts or browser alerts. */
export const notificationViewEnabled=():boolean=>settings()['notifications.view.enabled']?.value!==false;
export const notificationViewInSidebar=():boolean=>notificationViewEnabled()&&settings()['notifications.view.showInSidebar']?.value!==false;
