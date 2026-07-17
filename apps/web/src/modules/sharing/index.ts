// Public barrel for the web mailbox-sharing (ACL) module (t13 e8). E9 mounts the
// editor and injects a production `AclClient`; consumers import from here.

export { AclEditor, default } from './AclEditor.tsx';
export type { AclEditorProps } from './AclEditor.tsx';
