export class PushRegistrationError extends Error{
 constructor(readonly reason:'worker'|'server'|'permission'|'browser'|'bound'){super(`Push registration failed: ${reason}`);this.name='PushRegistrationError';}
}
