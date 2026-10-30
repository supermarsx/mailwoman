/* tslint:disable */
/* eslint-disable */

/**
 * `sanitizeEmailHtml(html)` → sanitized HTML. Applies the SAME allowlist policy as
 * the server-side [`crate::sanitize_email_html`] (scripts/styles/event-handlers/
 * remote images stripped), run CLIENT-SIDE on decrypted E2EE HTML (plan §1.3).
 */
export function sanitizeEmailHtml(html: string): string;

/**
 * `sanitizeEmailHtmlReader(html, blockTrackers, cleanLinks)` → sanitized HTML in
 * the reader's remote-content mode (t32 e31a): remote images become `mw-rc:<n>`
 * placeholders with their URLs in hidden markers, trackers are stripped when
 * `blockTrackers`, and tracking parameters are removed from links when
 * `cleanLinks`. For decrypted mail, whose URLs the reader then asks the server
 * to sign (`POST /api/rc/sign`).
 */
export function sanitizeEmailHtmlReader(html: string, block_trackers: boolean, clean_links: boolean): string;

/**
 * Reader quarantine with all remote resources stripped, including decrypted
 * messages whose caller has not supplied a remote-content reader policy.
 */
export function sanitizeEmailHtmlReaderStrip(html: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly sanitizeEmailHtml: (a: number, b: number, c: number) => void;
    readonly sanitizeEmailHtmlReader: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly sanitizeEmailHtmlReaderStrip: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
