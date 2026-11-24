// Pre-paint boot script (t52). Loaded as a CLASSIC script at the top of <head>
// (vite.config.ts `bootScript()` emits it as a hashed asset, so `script-src 'self'`
// covers it and the service worker caches it like the rest of the build). It runs
// before the first paint, reads only what `src/shell/bootSplash.ts` stored on the
// previous visit, and writes only attributes and CSSOM properties on <html> (not
// governed by `style-src`). See bootSplash.ts for the whole hand-over. Since
// t52w it also starts the first screen's downloads (below).
//
// Plain JavaScript with no imports: it is not bundled.
/* global document, window, location, navigator, fetch, URL */
(function () {
  var root = document.documentElement;
  var ls = null;
  try { ls = window.localStorage; } catch { ls = null; }
  function read(key) {
    if (ls === null) return null;
    try { var raw = ls.getItem(key); return raw === null ? null : JSON.parse(raw); } catch { return null; }
  }
  try { if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) root.setAttribute('data-reduced-motion', ''); } catch { /* no matchMedia */ }

  // Pulse in step with the document timeline, as the in-app placeholders do
  // (bootSplash.ts `alignPulse`), so a hand-over does not restart the pulse.
  document.addEventListener('DOMContentLoaded', function () {
    var splash = document.getElementById('mw-boot');
    if (splash === null || typeof splash.getAnimations !== 'function') return;
    try {
      var list = splash.getAnimations({ subtree: true });
      for (var j = 0; j < list.length; j++) {
        if (list[j].effect && list[j].effect.getTiming().iterations === Infinity) list[j].startTime = 0;
      }
    } catch { /* keep the engine's own phase */ }
  });
  // The admin console (t72) keeps its own appearance (applied after sign-in from
  // the admin's account preferences), so none of the webmail's is replayed here.
  // It gets its own placeholder: the console frame when an admin session was
  // open last time, else the sign-in card, sized as it was measured on the
  // previous visit for the same layout class (the console's phone layout is
  // below 761px, screens/Admin/shell/shell.css.ts).
  var consoleMeta = document.querySelector('meta[name="mw-console"]');
  if (consoleMeta !== null && consoleMeta.getAttribute('content') === 'admin') {
    var adminSession = false;
    try { adminSession = ls !== null && ls.getItem('mw.boot.admin') === '1'; } catch { adminSession = false; }
    root.setAttribute('data-mw-boot', adminSession ? 'admin-console' : 'admin');
    // The sign-in form offers a passkey button only where the browser has WebAuthn.
    if (typeof window.PublicKeyCredential === 'undefined' || !('credentials' in navigator)) root.setAttribute('data-mw-boot-nopasskey', '');
    var card = read('mw.boot.admin.card');
    if (!adminSession && card !== null && typeof card === 'object' && card.v === 1 && card.wide === (window.innerWidth >= 761)) {
      var h = Number(card.height);
      if (isFinite(h) && h >= 120 && h <= 2000) root.style.setProperty('--mwb-admin-card', Math.round(h * 10) / 10 + 'px');
      if (card.logo === true) root.setAttribute('data-mw-boot-logo', '');
    }
    return;
  }

  function clock(value) {
    var m = /^([01]\d|2[0-3]):([0-5]\d)$/.exec(String(value || ''));
    return m === null ? null : Number(m[1]) * 60 + Number(m[2]);
  }
  function darkNow(snap) {
    if (snap.mode === 'system') {
      try { return window.matchMedia('(prefers-color-scheme: dark)').matches; } catch { return snap.appearance === 'dark'; }
    }
    var s = snap.schedule || {};
    var start = clock(s.darkStart), end = clock(s.darkEnd);
    if (start === null || end === null || start === end) return false;
    var now = new Date(), minutes = now.getHours() * 60 + now.getMinutes();
    return start < end ? minutes >= start && minutes < end : minutes >= start || minutes < end;
  }

  var snap = read('mw.boot.theme');
  if (snap !== null && typeof snap === 'object' && snap.v === 1 && typeof snap.theme === 'string') {
    var theme = snap.theme, appearance = snap.appearance === 'light' ? 'light' : 'dark';
    if (snap.mode === 'system' || snap.mode === 'schedule') {
      var dark = darkNow(snap);
      var picked = dark ? snap.dark : snap.light;
      if (typeof picked === 'string' && picked !== '') { theme = picked; appearance = dark ? 'dark' : 'light'; }
    }
    root.setAttribute('data-theme', theme);
    root.setAttribute('data-appearance', appearance);
    root.style.setProperty('color-scheme', appearance);
    if (typeof snap.density === 'string' && snap.density !== '') root.setAttribute('data-density', snap.density);
    if (typeof snap.accent === 'string' && snap.accent !== '') root.setAttribute('data-accent', snap.accent);
    if (typeof snap.badge === 'string' && snap.badge !== '' && snap.badge !== 'default') root.setAttribute('data-badge', snap.badge);
    if (typeof snap.font === 'string' && snap.font !== '') root.style.setProperty('--mw-ui-font', snap.font);
    // A custom accent is shaded per pack: replay it only for the pack it was made for.
    if (theme === snap.theme && snap.props !== null && typeof snap.props === 'object') {
      for (var key in snap.props) {
        if (Object.prototype.hasOwnProperty.call(snap.props, key) && key.indexOf('--') === 0) root.style.setProperty(key, String(snap.props[key]));
      }
    }
  }

  // Which placeholder to draw. The mailbox shell only for the mail client's own
  // page, and only when a session was open last time; everything else (sign-in,
  // OAuth / OIDC consent) gets the themed page and nothing on it.
  var path = location.pathname.replace(/\/+$/, '');
  var consent = /\/(oauth\/authorize|oidc\/consent)$/.test(path);
  var session = false;
  try { session = ls !== null && ls.getItem('mw.boot.session') === '1'; } catch { session = false; }
  root.setAttribute('data-mw-boot', session && !consent ? 'mail' : 'plain');

  // t52w: the load-time reads in one request, started here so its round trip
  // overlaps the entry script's download. src/api/http.ts `startBootstrap`
  // adopts this promise (and sends the request itself when it is absent); the
  // same plain cookie GET, never in a native shell (bearer tokens there). The
  // URL is the deploy prefix's /api/bootstrap: this script is {prefix}/assets/.
  try {
    var bootEl = document.currentScript;
    if (typeof window.__TAURI_INTERNALS__ === 'undefined' && bootEl !== null && bootEl.src) {
      var started = fetch(new URL('../api/bootstrap', bootEl.src).href, { credentials: 'same-origin', headers: { accept: 'application/json' } });
      started.catch(function () { /* a failed answer is "no seed" (api/bootstrap.ts) */ });
      window.__mwBootstrap = started;
    }
  } catch { /* the app sends its own request */ }

  // t52w: start what the first screen will ask for while the entry script is
  // still downloading, from the build manifest the shell carries ahead of this
  // script (src/boot/bootManifest.ts): the mailbox's chunks when a session was
  // open last time, else the sign-in screen's; the one UI font file in use; the
  // active locale's translation pack. Same-origin requests under the shell's
  // own CSP; setting `href` on a link is not a Trusted Types sink.
  //
  // The chunks are fetched, not preloaded with links: a module fetch that fails
  // (a 503 under a burst) stays failed for the life of the page in every
  // engine, a modulepreload included, while a fetch can be repeated. The
  // promise is parked in `__mwPreloaded` by URL; the build's preload helper
  // (src/boot/chunkRetry.ts) waits for it when the import asks for the file
  // and retries it if it did not complete. The font and the pack are not module
  // fetches, so plain preload links serve them.
  try {
    var data = document.getElementById('mw-boot-manifest');
    var manifest = data === null ? null : JSON.parse(data.textContent || 'null');
    if (manifest !== null && typeof manifest === 'object' && manifest.v === 1) {
      var ok = function (href) { return typeof href === 'string' && href.indexOf('./assets/') === 0; };
      var add = function (href, as) {
        if (!ok(href)) return;
        var link = document.createElement('link');
        link.rel = 'preload';
        link.setAttribute('as', as);
        if (as === 'font') link.setAttribute('type', 'font/woff2');
        // The requests these stand in for are CORS-mode, same-origin credentials.
        link.setAttribute('crossorigin', '');
        link.href = href;
        document.head.appendChild(link);
      };
      var early = window.__mwPreloaded = window.__mwPreloaded || {};
      var files = consent ? [] : session ? manifest.mail : manifest.login;
      for (var f = 0; files && f < files.length; f++) {
        if (!ok(files[f])) continue;
        var url = new URL(files[f], document.baseURI).href;
        var init = { credentials: 'same-origin' };
        var sri = manifest.sri && manifest.sri[files[f]];
        if (typeof sri === 'string') init.integrity = sri;
        if (!early[url]) early[url] = fetch(url, init).then(function (r) { return r.ok ? r.arrayBuffer().then(function () { return true; }) : false; }, function () { return false; });
      }
      // The UI face: the stored font override's first family, else the theme's (Jost).
      var stack = snap !== null && typeof snap === 'object' && typeof snap.font === 'string' && snap.font !== '' ? snap.font : 'Jost';
      var family = stack.split(',')[0].trim().replace(/^["']|["']$/g, '').toLowerCase().replace(/ variable$/, '').replace(/\s+/g, '-');
      if (manifest.fonts && Object.prototype.hasOwnProperty.call(manifest.fonts, family)) add(manifest.fonts[family], 'font');
      // The active locale, as screens/Settings/language.ts and i18n/locales.ts
      // pick it: the stored choice, else the browser's languages against the
      // shipped ones (Portuguese regions other than BR read as pt-PT), else en.
      var packs = manifest.packs || {};
      var locale = null;
      try { locale = ls === null ? null : ls.getItem('mw.locale'); } catch { locale = null; }
      if (locale === null || locale === 'browser' || !Object.prototype.hasOwnProperty.call(packs, locale)) {
        locale = null;
        var wanted = navigator.languages && navigator.languages.length ? navigator.languages : [navigator.language || 'en'];
        var shipped = Object.keys(packs);
        for (var w = 0; locale === null && w < wanted.length; w++) {
          var parts = String(wanted[w]).replace(/_/g, '-').split('-');
          var lang = parts[0].toLowerCase();
          var tag = parts.join('-').toLowerCase();
          // Fluent's tag grammar has no numeric region (es-419): such a tag matches nothing there.
          if (lang !== 'pt' && /-\d{3}(?:-|$)/.test(tag)) continue;
          if (lang === 'pt' && parts.length > 1) {
            var region = null;
            for (var p = 1; p < parts.length; p++) if (/^([a-z]{2}|\d{3})$/i.test(parts[p])) { region = parts[p].toUpperCase(); break; }
            if (region !== null) tag = region === 'BR' ? 'pt-br' : 'pt-pt';
          }
          for (var s = 0; locale === null && s < shipped.length; s++) if (shipped[s].toLowerCase() === tag) locale = shipped[s];
          if (locale === null && lang === 'pt' && Object.prototype.hasOwnProperty.call(packs, 'pt-BR')) locale = 'pt-BR';
          for (var b = 0; locale === null && b < shipped.length; b++) if (shipped[b].toLowerCase() === lang) locale = shipped[b];
        }
        if (locale === null) locale = 'en';
      }
      add(packs[locale], 'fetch');
    }
  } catch { /* no preloads: every file is still requested where it is used */ }

  // The previous visit's shell geometry, if it was measured for the same layout
  // (phone or wide). Values are clamped: they come from storage.
  var geo = read('mw.boot.geometry');
  if (geo !== null && typeof geo === 'object' && geo.v === 1 && geo.wide === (window.innerWidth >= 761)) {
    var names = ['bar', 'side', 'tabs', 'top', 'list', 'lead', 'row'];
    for (var i = 0; i < names.length; i++) {
      var n = Number(geo[names[i]]);
      if (isFinite(n) && n >= 0 && n <= 4000) root.style.setProperty('--mwb-' + names[i], Math.round(n) + 'px');
    }
  }

})();
