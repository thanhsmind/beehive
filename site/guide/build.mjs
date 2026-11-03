import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const worktreeRoot = path.resolve(__dirname, '../..');
const distDir = path.join(worktreeRoot, 'dist', 'site');
const guideDir = __dirname;
const viDir = path.join(guideDir, 'vi');
const manifestPath = path.join(guideDir, 'manifest.json');
const cssPath = path.join(guideDir, 'guide.css');

function escapeHtml(str) {
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function rewriteLinks(html, filename) {
  let rewritten = html.replace(/href="\/guide\/([a-zA-Z0-9_-]+)"/g, 'href="./$1.html"');
  rewritten = rewritten.replace(/href="\/guide\/?(?=")/g, 'href="./index.html"');

  if (rewritten.includes('href="/guide')) {
    throw new Error(`Unrewritten /guide link found in ${filename}`);
  }
  return rewritten;
}

function renderRail(chapters, currentSlug) {
  const items = [
    `<a class="guide-rail__item guide-rail__item--home" href="./index.html">Tổng quan</a>`
  ];
  for (const c of chapters) {
    const onClass = c.slug === currentSlug ? ' guide-rail__item--on' : '';
    items.push(
      `<a class="guide-rail__item${onClass}" href="./${c.slug}.html"><span class="guide-rail__num">${c.number}</span><span>${escapeHtml(c.title)}</span></a>`
    );
  }
  return `<nav class="guide-rail" aria-label="Chương">
  <div class="guide-rail__head">Hướng dẫn bee</div>
  ${items.join('\n  ')}
</nav>`;
}

function renderPage({ title, description, rail, article }) {
  return `<!doctype html>
<html lang="vi">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${escapeHtml(title)}</title>
  <meta name="description" content="${escapeHtml(description)}">
  <link rel="stylesheet" href="./guide.css">
</head>
<body>
  <div class="layout layout--guide"><aside id="sidebar" class="sidebar">${rail}</aside><div class="sidebar-backdrop"></div><main class="content"><article class="fg-prose guide-prose">${article}</article></main></div>
</body>
</html>
`;
}

function build() {
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const chapters = manifest.chapters;

  // Fresh dist/site directory
  fs.rmSync(distDir, { recursive: true, force: true });
  fs.mkdirSync(distDir, { recursive: true });

  // Copy guide.css
  fs.copyFileSync(cssPath, path.join(distDir, 'guide.css'));

  // 1. Build index page
  const indexFragmentRaw = fs.readFileSync(path.join(viDir, '_index.html'), 'utf8');
  const indexFragment = rewriteLinks(indexFragmentRaw, '_index.html');
  const indexRail = renderRail(chapters, null);

  const cardsHtml = chapters.map(c => `
  <a class="guide-card" href="./${c.slug}.html"><span class="guide-card__num">Chương ${c.number}</span><span class="guide-card__title">${escapeHtml(c.title)}</span><span class="guide-card__blurb">${escapeHtml(c.blurb)}</span></a>`).join('');

  const indexArticle = `${indexFragment}
<div class="guide-cards">${cardsHtml}
</div>`;

  const indexHtml = renderPage({
    title: manifest.title,
    description: manifest.title,
    rail: indexRail,
    article: indexArticle,
  });

  fs.writeFileSync(path.join(distDir, 'index.html'), indexHtml, 'utf8');

  // 2. Build chapter pages
  for (let i = 0; i < chapters.length; i++) {
    const chapter = chapters[i];
    const fragmentRaw = fs.readFileSync(path.join(viDir, `${chapter.slug}.html`), 'utf8');
    const chapterFragment = rewriteLinks(fragmentRaw, `${chapter.slug}.html`);
    const rail = renderRail(chapters, chapter.slug);

    const prev = i > 0 ? chapters[i - 1] : null;
    const next = i < chapters.length - 1 ? chapters[i + 1] : null;

    let stepsHtml = '';
    const stepItems = [];
    if (prev) {
      stepItems.push(`<a class="guide-step" href="./${prev.slug}.html"><span class="guide-step__rel">Chương trước</span><span class="guide-step__title">${escapeHtml(prev.title)}</span></a>`);
    }
    if (next) {
      stepItems.push(`<a class="guide-step guide-step--next" href="./${next.slug}.html"><span class="guide-step__rel">Chương sau</span><span class="guide-step__title">${escapeHtml(next.title)}</span></a>`);
    }
    if (stepItems.length > 0) {
      stepsHtml = `\n<nav class="guide-steps" aria-label="Chương trước và sau">${stepItems.join('')}</nav>`;
    }

    const chapterArticle = `<header class="guide-head"><div class="guide-head__eyebrow">Chương ${chapter.number} / ${chapters.length}</div><h1 class="guide-head__title">${escapeHtml(chapter.title)}</h1><p class="guide-head__blurb">${escapeHtml(chapter.blurb)}</p></header>${chapterFragment}${stepsHtml}`;

    const chapterHtml = renderPage({
      title: `${chapter.title} — ${manifest.title}`,
      description: chapter.blurb,
      rail,
      article: chapterArticle,
    });

    fs.writeFileSync(path.join(distDir, `${chapter.slug}.html`), chapterHtml, 'utf8');
  }

  console.log(`Successfully built ${chapters.length + 1} pages to dist/site/`);
}

build();
