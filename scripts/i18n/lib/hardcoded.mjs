// AST analysis behind no-hardcoded-strings.mjs (t32-admin §2.9).
//
// Parses one source file with the TypeScript compiler API and reports every
// user-visible string that does not go through t():
//
//   jsx-text          text between JSX tags, including text that spans several
//                     lines, and a string literal placed as a JSX child ({'Hi'})
//   attr:<name>       a string value for a user-visible JSX attribute
//                     (aria-label, title, placeholder, alt, label, …), also the
//                     literal branches of `cond ? 'A' : 'B'` and `x ?? 'A'`
//   prop:<name>       a string value for a `label:` / `description:` / `title:` /
//                     `placeholder:` / `tooltip:` / `ariaLabel:` property in an
//                     object literal (tables of options rendered as UI)
//   call:<name>       a string passed to alert() / confirm() / prompt() or to
//                     `new Notification(…)`
//   locale-format     `toLocaleString()` / `toLocaleDateString()` /
//                     `toLocaleTimeString()`, or `Intl.*Format` built without a
//                     locale (`undefined`, `[]` or no argument): both format in
//                     the browser locale instead of the app locale
//
// A site can be accepted in place with a comment on the same line or the line
// above: `// i18n-ignore: <reason>` (the reason is required).

/** At least one run of two letters: a word, not a glyph, number, symbol or HTML entity. */
export const hasWord = (s) => /\p{L}{2,}/u.test(s.replace(/&[a-zA-Z]+;|&#x?[0-9a-fA-F]+;/g, ' '));

/** A Fluent message id (`mail-outbox-sent`): the value is passed to t() later, it is not copy. */
const MESSAGE_ID = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/;

/** A CSS length or number with a unit (`0.06em`, `12px`): a style value, not copy. */
const CSS_VALUE = /^-?\d*\.?\d+[a-z%]*$/;

/** Text that would be shown as is: has a word and is neither a message id nor a CSS value. */
export const isCopy = (s) => hasWord(s) && !MESSAGE_ID.test(s.trim()) && !CSS_VALUE.test(s.trim());

/** JSX attributes whose string value is shown to people (or read by screen readers). */
export const UI_ATTRIBUTES = new Set([
  'aria-label',
  'aria-description',
  'aria-roledescription',
  'aria-placeholder',
  'aria-valuetext',
  'title',
  'placeholder',
  'alt',
  'label',
]);

/** Object-literal property names whose string value is rendered as UI copy. */
export const UI_PROPERTIES = new Set([
  'label',
  'description',
  'title',
  'placeholder',
  'tooltip',
  'ariaLabel',
  'aria-label',
  'emptyText',
  'helpText',
]);

const DIALOG_CALLS = new Set(['alert', 'confirm', 'prompt']);
const LOCALE_METHODS = new Set(['toLocaleString', 'toLocaleDateString', 'toLocaleTimeString']);
const INTL_FORMATTERS = new Set([
  'DateTimeFormat',
  'NumberFormat',
  'RelativeTimeFormat',
  'ListFormat',
  'PluralRules',
  'DisplayNames',
]);

const IGNORE_RE = /\/\/\s*i18n-ignore:\s*\S/;

/** Collapse JSX whitespace the way the compiler does for rendered text. */
function jsxTextValue(raw) {
  return raw.replace(/\s+/g, ' ').trim();
}

/**
 * Analyse one file. `ts` is the typescript module; `fileName` decides the
 * script kind (.tsx parses JSX). Returns findings `{ kind, text, line }` with a
 * 1-based line number; `multiline` marks JSX text that spans lines.
 */
export function analyzeSource(ts, fileName, source) {
  const kind = fileName.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS;
  const sf = ts.createSourceFile(fileName, source, ts.ScriptTarget.Latest, true, kind);
  const lines = source.split(/\r?\n/);
  const findings = [];

  const lineOf = (node) => sf.getLineAndCharacterOfPosition(node.getStart(sf)).line + 1;
  const ignored = (line) => IGNORE_RE.test(lines[line - 1] ?? '') || IGNORE_RE.test(lines[line - 2] ?? '');

  const add = (node, k, text, extra = {}) => {
    const line = lineOf(node);
    if (ignored(line)) return;
    findings.push({ kind: k, text, line, ...extra });
  };

  /** String literals an expression can evaluate to, through ?:, ??, || and parens. */
  const literalBranches = (expr, out = []) => {
    if (expr === undefined) return out;
    if (ts.isStringLiteral(expr) || ts.isNoSubstitutionTemplateLiteral(expr)) out.push(expr);
    else if (ts.isParenthesizedExpression(expr)) literalBranches(expr.expression, out);
    else if (ts.isConditionalExpression(expr)) {
      literalBranches(expr.whenTrue, out);
      literalBranches(expr.whenFalse, out);
    } else if (
      ts.isBinaryExpression(expr) &&
      (expr.operatorToken.kind === ts.SyntaxKind.QuestionQuestionToken ||
        expr.operatorToken.kind === ts.SyntaxKind.BarBarToken)
    ) {
      literalBranches(expr.right, out);
    } else if (ts.isTemplateExpression(expr)) {
      // `Hello ${name}`: the literal parts are copy.
      const text = jsxTextValue([expr.head.text, ...expr.templateSpans.map((s) => s.literal.text)].join(' '));
      if (hasWord(text)) out.push({ text, node: expr });
    }
    return out;
  };
  // literalBranches yields either a literal node or `{ text, node }` for a template.
  const literalText = (n) => n.text;
  const literalNode = (n) => n.node ?? n;

  const propName = (name) => {
    if (ts.isIdentifier(name) || ts.isStringLiteral(name)) return name.text;
    return null;
  };

  const visit = (node) => {
    if (ts.isJsxText(node)) {
      const text = jsxTextValue(node.text);
      if (hasWord(text)) {
        // Report the line of the first visible character; the text is
        // multi-line when the visible part itself contains a line break.
        const line = sf.getLineAndCharacterOfPosition(node.pos + node.text.search(/\S/)).line + 1;
        if (!ignored(line)) {
          findings.push({ kind: 'jsx-text', text, line, multiline: node.text.trim().includes('\n') });
        }
      }
    } else if (ts.isJsxExpression(node) && node.expression && ts.isJsxElement(node.parent)) {
      for (const lit of literalBranches(node.expression)) {
        if (isCopy(literalText(lit))) add(literalNode(lit), 'jsx-text', literalText(lit).trim());
      }
    } else if (ts.isJsxAttribute(node)) {
      const name = node.name.getText(sf);
      if (UI_ATTRIBUTES.has(name) && node.initializer) {
        const init = node.initializer;
        const lits = ts.isStringLiteral(init) ? [init] : ts.isJsxExpression(init) ? literalBranches(init.expression) : [];
        for (const lit of lits) {
          if (isCopy(literalText(lit))) add(literalNode(lit), `attr:${name}`, literalText(lit).trim());
        }
      }
    } else if (ts.isPropertyAssignment(node) && ts.isObjectLiteralExpression(node.parent)) {
      const name = propName(node.name);
      if (name !== null && UI_PROPERTIES.has(name)) {
        for (const lit of literalBranches(node.initializer)) {
          if (isCopy(literalText(lit))) add(literalNode(lit), `prop:${name}`, literalText(lit).trim());
        }
      }
    } else if (ts.isCallExpression(node) || ts.isNewExpression(node)) {
      const callee = node.expression;
      const args = node.arguments ?? [];
      // alert('…') / window.confirm('…') / new Notification('…')
      const calleeName = ts.isIdentifier(callee)
        ? callee.text
        : ts.isPropertyAccessExpression(callee)
          ? callee.name.text
          : null;
      if (
        calleeName !== null &&
        ((ts.isCallExpression(node) && DIALOG_CALLS.has(calleeName)) ||
          (ts.isNewExpression(node) && calleeName === 'Notification'))
      ) {
        for (const lit of literalBranches(args[0])) {
          if (isCopy(literalText(lit))) add(literalNode(lit), `call:${calleeName}`, literalText(lit).trim());
        }
      }
      // x.toLocaleString(…)
      if (ts.isCallExpression(node) && ts.isPropertyAccessExpression(callee) && LOCALE_METHODS.has(callee.name.text)) {
        add(node, 'locale-format', `${callee.name.text}()`);
      }
      // Intl.DateTimeFormat(undefined | [] | nothing, …), except a bare
      // Intl.DateTimeFormat().resolvedOptions() (a time-zone lookup, not formatting).
      if (
        ts.isPropertyAccessExpression(callee) &&
        ts.isIdentifier(callee.expression) &&
        callee.expression.text === 'Intl' &&
        INTL_FORMATTERS.has(callee.name.text)
      ) {
        const first = args[0];
        const noLocale =
          first === undefined ||
          (ts.isIdentifier(first) && first.text === 'undefined') ||
          (ts.isArrayLiteralExpression(first) && first.elements.length === 0);
        const resolvedOnly =
          ts.isPropertyAccessExpression(node.parent) && node.parent.name.text === 'resolvedOptions';
        if (noLocale && !resolvedOnly) add(node, 'locale-format', `Intl.${callee.name.text}(${first ? first.getText(sf) : ''})`);
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(sf);
  return findings;
}
