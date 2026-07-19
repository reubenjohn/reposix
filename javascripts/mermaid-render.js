// Mermaid render workaround for mkdocs-material 9.7.x
//
// Symptom 1 (original): <div class="mermaid"></div> ends up empty after
// page load. Material's bundle strips the content of any element with
// class="mermaid" before mermaid runs on it; the transform was designed
// for an older fence_code_format flow and now leaves the div empty.
// Workaround: re-fetch the page HTML, extract the original source from
// the static markup, and call mermaid.render() ourselves.
//
// Symptom 2 (HANDOVER §0.1.b — surfaced 2026-04-27 PM): mermaid only
// renders on the SECOND page reload. Diagnosis: navigation.instant is
// enabled in mkdocs.yml; clicking an in-site link does an XHR DOM swap
// without firing DOMContentLoaded. The previous IIFE only hooked
// DOMContentLoaded, so subsequent in-site navs never re-ran renderAll
// and the new page's <div class="mermaid"> blocks stayed empty. A real
// reload triggers a full page load → DOMContentLoaded fires → script
// runs → diagrams appear (the "works on second reload" symptom).
//
// Fix: subscribe to mkdocs-material's `document$` observable (re-fires
// on every instant nav), with a DOMContentLoaded fallback for builds
// without the Material theme. Both code paths funnel through renderAll.
//
// See:
// - .planning/research/v0.11.0-mkdocs-site-audit.md
// - HANDOVER.md §0.1 + §0.1.b
// - https://squidfunk.github.io/mkdocs-material/customization/#additional-javascript

(function () {
  async function waitForMermaid(maxMs = 5000) {
    const start = Date.now();
    while (!window.mermaid && Date.now() - start < maxMs) {
      await new Promise((r) => setTimeout(r, 50));
    }
    return !!window.mermaid;
  }

  function decodeEntities(s) {
    const t = document.createElement('textarea');
    t.innerHTML = s;
    return t.value;
  }

  async function renderAll() {
    const divs = document.querySelectorAll('div.mermaid');
    if (!divs.length) return;

    if (!(await waitForMermaid())) {
      console.warn('[mermaid-render.js] mermaid library never loaded');
      return;
    }

    // Pull the original source out of the static HTML so we don't depend
    // on Material's transform leaving anything behind.
    const res = await fetch(location.pathname, { cache: 'no-store' });
    const html = await res.text();
    const sources = [];
    const re = /<div class="mermaid">([\s\S]*?)<\/div>/g;
    let m;
    while ((m = re.exec(html)) !== null) {
      sources.push(decodeEntities(m[1]).trim());
    }

    try {
      window.mermaid.initialize({
        startOnLoad: false,
        securityLevel: 'loose',
        theme: 'default',
      });
    } catch (e) {
      // Already initialized; ignore.
    }

    for (let i = 0; i < divs.length; i++) {
      const div = divs[i];
      if (div.querySelector('svg')) continue; // already rendered
      const source = sources[i];
      if (!source) continue;
      try {
        const id = `mermaid-${i}-${Math.random().toString(36).slice(2)}`;
        const { svg } = await window.mermaid.render(id, source);
        div.innerHTML = svg;
      } catch (e) {
        console.error('[mermaid-render.js] render failed for diagram', i, e);
      }
    }
  }

  // mkdocs-material exposes `document$` as an RxJS Observable that emits
  // on the initial page load AND on every instant navigation. Subscribing
  // to it is the supported way to re-run setup code per page.
  if (typeof document$ !== 'undefined' && document$ && typeof document$.subscribe === 'function') {
    document$.subscribe(() => { renderAll(); });
  } else {
    // Fallback for non-Material themes or strict CSP that blocks Material's
    // script bundle. Hooks DOMContentLoaded only — does NOT survive instant
    // nav, but is correct for a vanilla mkdocs build.
    if (document.readyState !== 'loading') {
      renderAll();
    } else {
      document.addEventListener('DOMContentLoaded', renderAll);
    }
  }
})();
