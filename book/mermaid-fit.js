// Shows wide mermaid diagrams at a readable scale with horizontal scrolling,
// instead of letting them shrink to the width of the text column.
(() => {
    const MIN_SCALE = 0.75;

    const fit = (svg) => {
        const viewBox = svg.viewBox && svg.viewBox.baseVal;
        const pre = svg.closest('pre.mermaid');
        if (!viewBox || !viewBox.width || !pre) return;
        // Narrow enough to stay readable: keep mermaid's own sizing.
        if (viewBox.width * MIN_SCALE <= pre.clientWidth) return;
        svg.style.maxWidth = 'none';
        svg.style.width = `${Math.round(viewBox.width * MIN_SCALE)}px`;
        svg.style.height = 'auto';
        pre.classList.add('mermaid-wide');
    };

    const fitAll = () => document.querySelectorAll('pre.mermaid svg').forEach(fit);

    // Mermaid replaces each <pre class="mermaid"> body with an <svg> once rendered.
    const observer = new MutationObserver(fitAll);
    document.querySelectorAll('pre.mermaid').forEach((pre) => observer.observe(pre, { childList: true }));
    window.addEventListener('load', fitAll);
})();
