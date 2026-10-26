import {settings} from '../api/settings.ts';
/** Effective administrator locks and user values share the settings resolver. */
export const modifierSelectionEnabled=():boolean=>settings()['mail.modifierSelection']?.value!==false;
export const dragToFolderEnabled=():boolean=>settings()['mail.dragToFolder']?.value!==false;
