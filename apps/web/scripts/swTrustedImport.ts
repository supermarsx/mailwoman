/**
 * t59: the line the build writes into dist/sw.js to load the offline-background
 * bundle.
 *
 * mw-server sends every response, /sw.js included, with the shell CSP, and that
 * CSP carries `require-trusted-types-for 'script'`. In a worker,
 * `importScripts(url)` is a TrustedScriptURL sink. The page's default policy
 * (src/security/trustedTypes.ts) does not exist in the worker global, so a plain
 * string is refused: "This document requires 'TrustedScriptURL' assignment". The
 * worker script then throws during evaluation and never installs.
 *
 * The emitted line creates a policy in the worker. Its createScriptURL accepts
 * exactly one value, the URL fixed at build time, and throws for anything else.
 * A browser without `self.trustedTypes` gets the plain string, which it accepts.
 * The CSP has no `trusted-types` directive, so any policy name is allowed.
 */
export const SW_IMPORT_POLICY = 'mw-sw-import';

/** Only same-directory content-addressed files under assets/ are accepted. */
const ASSET = /^assets\/[\w.-]+\.js$/;

export function swTrustedImport(file: string): string {
  if (!ASSET.test(file)) throw new Error(`swTrustedImport: ${JSON.stringify(file)} is not an assets/<name>.js path`);
  const url = JSON.stringify(`./${file}`);
  return `(function(u){var t=self.trustedTypes;importScripts(t&&t.createPolicy?t.createPolicy(${JSON.stringify(SW_IMPORT_POLICY)},{createScriptURL:function(v){if(v!==u)throw new TypeError(${JSON.stringify(`${SW_IMPORT_POLICY}: refused `)}+v);return v;}}).createScriptURL(u):u);})(${url});`;
}
