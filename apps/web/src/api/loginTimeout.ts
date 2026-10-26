export const LOGIN_TIMEOUT_MS = 30_000;
export class LoginTimeoutError extends Error {
  constructor() { super('Sign-in timed out'); this.name = 'LoginTimeoutError'; }
}

/** One attempt only. Cancels fetch/body consumption; never retries credentials. */
export async function withLoginTimeout<T>(run: (signal: AbortSignal) => Promise<T>): Promise<T> {
  const controller = new AbortController();
  let timer: ReturnType<typeof setTimeout> | undefined;
  const deadline = new Promise<never>((_resolve, reject) => {
    timer = setTimeout(() => { const error = new LoginTimeoutError(); controller.abort(error); reject(error); }, LOGIN_TIMEOUT_MS);
  });
  try { return await Promise.race([run(controller.signal), deadline]); }
  finally { clearTimeout(timer); }
}
