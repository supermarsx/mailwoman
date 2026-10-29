// Pre-paint boot script (t52). Loaded as a CLASSIC script at the top of <head>
// (vite.config.ts `bootScript()` emits it as a hashed asset, so `script-src 'self'`
// covers it and the service worker caches it like the rest of the build). It runs
// before the first paint, reads only what `src/shell/bootSplash.ts` stored on the
// previous visit, and writes only attributes and CSSOM properties on <html> (not
// governed by `style-src`). See bootSplash.ts for the whole hand-over.
//
// Plain JavaScript with no imports: it is not bundled.
/* global document, window, location */
(function () {
  var root = document.documentElement;
  var ls = null;
  try { ls = window.localStorage; } catch { ls = null; }
  function read(key) {
    if (ls === null) return null;
    try { var raw = ls.getItem(key); return raw === null ? null : JSON.parse(raw); } catch { return null; }
  }
  // The admin console keeps its own appearance and its own loading states.
  var consoleMeta = document.querySelector('meta[name="mw-console"]');
  if (consoleMeta !== null && consoleMeta.getAttribute('content') === 'admin') return;

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
  try { if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) root.setAttribute('data-reduced-motion', ''); } catch { /* no matchMedia */ }
  root.setAttribute('data-mw-boot', session && !consent ? 'mail' : 'plain');

  // The previous visit's shell geometry, if it was measured for the same layout
  // (phone or wide). Values are clamped: they come from storage.
  var geo = read('mw.boot.geometry');
  if (geo !== null && typeof geo === 'object' && geo.v === 1 && geo.wide === (window.innerWidth >= 768)) {
    var names = ['bar', 'side', 'tabs', 'top', 'list', 'lead', 'row'];
    for (var i = 0; i < names.length; i++) {
      var n = Number(geo[names[i]]);
      if (isFinite(n) && n >= 0 && n <= 4000) root.style.setProperty('--mwb-' + names[i], Math.round(n) + 'px');
    }
  }

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
})();
