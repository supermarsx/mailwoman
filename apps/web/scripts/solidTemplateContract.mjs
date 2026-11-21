import {createHash} from 'node:crypto';
import {parseFragment} from 'parse5';

const hash = value => createHash('sha256').update(value).digest('hex');
const fail = message => { throw Error(`Solid template proof: ${message}`); };
const MAX_CODE = 8 * 1024 * 1024;
const MAX_NODES = 1000000;
const MAX_HTML = 256 * 1024;
const MAX_TEMPLATES = 4096;
const canonicalSolidWeb = source => typeof source === 'string' && /^solid-js\/web(?:\/|$)/.test(source);

// Visit actual ESTree nodes, never the contents of string literals. Bounds also
// apply to vendor/synthetic modules; a rendered unknown is not an exemption.
function walk(root, visit) {
  const pending = [[root, null, '']];
  let count = 0;
  while (pending.length) {
    const [node, parent, key] = pending.pop();
    if (!node || typeof node.type !== 'string') fail('invalid AST');
    if (++count > MAX_NODES) fail('AST node bound');
    visit(node, parent, key);
    for (const [name, value] of Object.entries(node)) {
      if (name === 'parent') fail('cyclic AST');
      if (Array.isArray(value)) {
        for (const child of value) if (child && typeof child.type === 'string') pending.push([child, node, name]);
      } else if (value && typeof value.type === 'string') pending.push([value, node, name]);
    }
  }
}

export function solidModuleSources(ast) {
  const sources = new Set();
  walk(ast, node => {
    const source = ['ImportDeclaration','ExportNamedDeclaration','ExportAllDeclaration','ImportExpression'].includes(node.type) ? node.source?.value :
      node.type === 'CallExpression' && node.callee?.name === 'require' ? node.arguments[0]?.value : null;
    if (typeof source === 'string') sources.add(source);
    if (sources.size > 4096) fail('module import bound');
  });
  return [...sources];
}

export function inspectSolidTemplates(code, ast, resolvedSolidSources = []) {
  if (typeof code !== 'string' || Buffer.byteLength(code) > MAX_CODE || ast?.type !== 'Program') fail('missing/bounded module code or AST');
  const solidWeb = source => canonicalSolidWeb(source) || resolvedSolidSources.includes(source);
  const aliases = new Set();
  const namespaces = new Set();
  for (const node of ast.body) {
    if (node.type !== 'ImportDeclaration' || !solidWeb(node.source?.value)) continue;
    for (const spec of node.specifiers) {
      if (spec.type === 'ImportSpecifier' && (spec.imported.name ?? spec.imported.value) === 'template') aliases.add(spec.local.name);
      if (spec.type === 'ImportNamespaceSpecifier' || spec.type === 'ImportDefaultSpecifier') namespaces.add(spec.local.name);
    }
  }
  const templates = [];
  walk(ast, (node, parent, key) => {
    if ((node.type === 'ImportExpression' && solidWeb(node.source?.value)) ||
        (node.type === 'CallExpression' && node.callee?.name === 'require' && solidWeb(node.arguments[0]?.value))) fail('dynamic Solid import');
    if (solidWeb(node.source?.value) && (node.type === 'ExportAllDeclaration' ||
        (node.type === 'ExportNamedDeclaration' && node.specifiers.some(spec => (spec.local?.name ?? spec.local?.value) === 'template')))) fail('unattested Solid re-export');
    if (node.type === 'Identifier' && namespaces.has(node.name)) {
      if (parent?.type !== 'ImportNamespaceSpecifier' && parent?.type !== 'ImportDefaultSpecifier') fail('Solid namespace/default use');
    }
    if (node.type !== 'Identifier' || !aliases.has(node.name)) return;
    if (parent?.type === 'ImportSpecifier') return;
    // Noncomputed object/property names are not variable references.
    if ((parent?.type === 'MemberExpression' && key === 'property' && !parent.computed) ||
        (parent?.type === 'Property' && key === 'key' && !parent.computed && !parent.shorthand)) return;
    if (parent?.type !== 'CallExpression' || key !== 'callee' || parent.optional) fail('escaped or shadowed template binding');
    const args = parent.arguments;
    const literal = args[0]?.type === 'Literal' && typeof args[0].value === 'string' ? args[0].value :
      args[0]?.type === 'TemplateLiteral' && args[0].expressions.length === 0 && args[0].quasis.length === 1 ? args[0].quasis[0].value.cooked : null;
    if (args.length < 1 || args.length > 4 || typeof literal !== 'string' ||
        args.slice(1).some(arg => arg.type !== 'Literal' || typeof arg.value !== 'boolean')) fail('unexpected template call');
    const html = literal;
    if (Buffer.byteLength(html) > MAX_HTML || templates.length >= MAX_TEMPLATES) fail('template bound');
    let styles = 0;
    const errors = [];
    const fragment = parseFragment(html, {onParseError: error => errors.push(error.code)});
    if (errors.length) fail('malformed template HTML');
    const pending = [fragment];
    while (pending.length) {
      const element = pending.pop();
      styles += (element.attrs ?? []).filter(attr => attr.name.toLowerCase() === 'style').length;
      pending.push(...(element.childNodes ?? []));
      if (element.content) pending.push(element.content);
    }
    templates.push({sha256: hash(html), styleAttributes: styles});
  });
  return {status: 'rendered', codeSha256: hash(code), templates};
}

export function inspectStandaloneAsset(code, ast) {
  // A nested worker/esbuild output has lost original factory import bindings.
  // Certify only self-contained scripts with no HTML parser sinks; otherwise
  // require real source-module provenance instead of assuming a clean worker.
  const result = inspectSolidTemplates(code, ast);
  walk(ast, node => {
    if (['ImportDeclaration','ImportExpression','ExportAllDeclaration'].includes(node.type) ||
        (node.type === 'ExportNamedDeclaration' && node.source) ||
        (node.type === 'CallExpression' && node.callee?.name === 'require')) fail('asset requires source-module import provenance');
    if (node.type === 'MemberExpression') {
      const property = node.computed ? node.property?.value : node.property?.name;
      if (['innerHTML','outerHTML','insertAdjacentHTML','createContextualFragment','parseFromString'].includes(property)) fail('asset requires DOM parser source provenance');
    }
  });
  return result;
}

function receiptStyles(result) {
  if (result?.status !== 'rendered' || !/^[a-f0-9]{64}$/.test(result.codeSha256) || !Array.isArray(result.templates) || result.templates.length > MAX_TEMPLATES) fail('rendered proof');
  let styles = 0;
  for (const item of result.templates) {
    if (!/^[a-f0-9]{64}$/.test(item.sha256) || !Number.isSafeInteger(item.styleAttributes) || item.styleAttributes < 0) fail('template proof');
    styles += item.styleAttributes;
  }
  return styles;
}

export function solidTemplateGraphProof(graph) {
  if (graph?.schema !== 1 || graph.solidTemplates?.schema !== 1 || !graph.chunks) fail('missing versioned graph attestation');
  const proof = graph.solidTemplates.chunks;
  const closure = graph.solidTemplates.sourceClosure;
  if (!proof || Object.keys(proof).length !== Object.keys(graph.chunks).length || !closure || Object.keys(closure).length > 20000) fail('chunk/source coverage');
  const violations = [];
  const roots = new Set();
  let modules = 0, templates = 0;
  for (const [name, chunk] of Object.entries(graph.chunks)) {
    const entry = proof[name];
    if (!entry || entry.sha256 !== chunk.sha256 || entry.byteLength !== chunk.byteLength || !entry.modules ||
        Object.keys(entry.modules).length !== Object.keys(chunk.modules).length) fail('chunk/hash/module coverage');
    for (const [id, length] of Object.entries(chunk.modules)) {
      if (++modules > 20000 || !Number.isSafeInteger(length) || length < 0) fail('module bound');
      const result = entry.modules[id];
      if (!result || result.renderedLength !== length || !closure[result.sourceId]) fail('module coverage');
      roots.add(result.sourceId);
      if (length === 0) {
        if (result.status !== 'zero-rendered') fail('zero-rendered proof');
      } else {
        if (result.status !== 'rendered' || !/^[a-f0-9]{64}$/.test(result.codeSha256) || closure[result.sourceId].result.codeSha256 !== result.codeSha256) fail('module/source binding');
      }
    }
  }
  // Forwarding modules may have zero emitted bytes or no chunk.modules entry at
  // all. Traverse the actual Rollup dependency closure, not only retained code.
  const pending = [...roots], reached = new Set();
  while (pending.length) {
    const id = pending.pop();
    if (reached.has(id)) continue;
    reached.add(id);
    const source = closure[id];
    if (!source || !Array.isArray(source.dependencies) || source.dependencies.length > 4096) fail('source dependency coverage');
    pending.push(...source.dependencies);
    const styles = receiptStyles(source.result);
    templates += source.result.templates.length;
    if (templates > 100000) fail('template count bound');
    if (styles) violations.push({file:'source-closure',module:id,styleAttributes:styles});
  }
  if (reached.size !== Object.keys(closure).length) fail('unreachable source proof');
  return {modules, templates, violations};
}

// Emitted assets (boot scripts and separately bundled workers) have no owning
// chunk.modules. They require their own parsed-output attestation, not an exempt
// filename. Every current JS file must occur in exactly one evidence category.
export function solidTemplateAssetCoverage(graph, actualFiles) {
  const assets = graph?.solidTemplates?.assets;
  if (!assets || !Array.isArray(actualFiles) || new Set(actualFiles).size !== actualFiles.length) fail('missing emitted JS coverage');
  const chunks = Object.keys(graph.chunks).filter(name => /\.(?:m?js)$/.test(name));
  const files = Object.keys(assets);
  const expected = [...chunks, ...files];
  if (new Set(expected).size !== expected.length || expected.length !== actualFiles.length || expected.some(name => !actualFiles.includes(name))) fail('emitted JS coverage');
  const violations = [];
  for (const [name,value] of Object.entries(assets)) {
    if (!value || !Number.isSafeInteger(value.byteLength) || value.byteLength < 0 || !/^[a-f0-9]{64}$/.test(value.sha256)) fail('asset bytes');
    const styles = receiptStyles(value);
    if (styles) violations.push({file:name,module:'parsed-output-asset',styleAttributes:styles});
  }
  return violations;
}
