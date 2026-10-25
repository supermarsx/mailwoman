/** Password mechanisms implemented by the protocol clients; bearer tokens are separate. */
export type PasswordMechanism = 'plain' | 'login' | 'scram-sha-256' | 'scram-sha-256-plus' | 'imap-login' | 'pop3-user-pass';
export interface PasswordAuthentication {
  mode: 'fixed' | 'auto';
  mechanisms: PasswordMechanism[];
}
