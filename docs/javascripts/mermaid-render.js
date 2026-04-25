// Mermaid render workaround for mkdocs-material 9.7.x
//
// Symptom: <div class="mermaid"></div> ends up empty after page load.
// Mermaid library is loaded, source is in static HTML, mermaid.render()
// works when called directly, but the rendered DOM is missing the SVG.
//
// Diagnosis: Material's bundle.js strips the content of any element with
// class="mermaid" before mermaid runs on it. The transform was designed
// for an older fence_code_format flow and now leaves the div empty.
//
// Workaround: re-fetch the page HTML, extract the original source from
// the static markup, and call mermaid.render() ourselves.
//
// See:
// - .planning/research/v0.11.0-mkdocs-site-audit.md
// - https://github.com/squidfunk/mkdocs-material/issues (likely upstream)

(function () {
  function ready(fn) {
    if (document.readyState !== 'loading') return fn();
    document.addEventListener('DOMContentLoaded', fn);
  }

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

  ready(renderAll);
})();
