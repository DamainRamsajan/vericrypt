/* Applies a saved theme choice before first paint. Classic script, loaded in <head>. */
(function () {
  try {
    var t = localStorage.getItem('iw-theme');
    if (t === 'light' || t === 'dark') document.documentElement.setAttribute('data-theme', t);
  } catch (e) { /* storage unavailable: follow the system setting */ }
})();
