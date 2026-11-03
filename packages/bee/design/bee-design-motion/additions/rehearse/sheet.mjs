#!/usr/bin/env node
import { launch } from './browser.mjs'
import { mkdirSync } from 'node:fs'
import { dirname, resolve } from 'node:path'

const args = process.argv.slice(2)
const options = { every: 1000, count: 12, width: 1280, select: 'body', out: 'sheet.png', cols: null }
let url
try {
  for (let i = 0; i < args.length; i++) {
    const arg = args[i]
    if (arg === '--help') { console.log('usage: node additions/rehearse/sheet.mjs <url> [--select body] [--width 1280] [--every 1000] [--count 12] [--cols auto] [--out sheet.png]\nTakes --count screenshots of the --select element every --every ms at device scale 2, one grid PNG, each frame labelled with its time. Prints the absolute path written. Exit 0 = written, 2 = the run was incomplete (no element, no frames, page error). A sheet is a record of frames, not approval.'); process.exit(0) }
    else if (/^--(every|count|width|select|out|cols)$/.test(arg)) {
      if (!args[i + 1] || args[i + 1].startsWith('--')) throw new Error(`${arg} requires a value`)
      options[arg.slice(2)] = args[++i]
    } else if (arg.startsWith('--') || url) throw new Error(`unexpected argument: ${arg}`)
    else url = arg
  }
  if (!url) throw new Error('a URL is required; use --help')
  for (const name of ['every', 'count', 'width']) {
    options[name] = Number(options[name])
    if (!Number.isInteger(options[name]) || options[name] < 1) throw new Error(`--${name} must be a positive integer`)
  }
  if (options.cols != null && options.cols !== 'auto') {
    options.cols = Number(options.cols)
    if (!Number.isInteger(options.cols) || options.cols < 1) throw new Error('--cols must be a positive integer or auto')
  }
} catch (error) { console.error(`sheet: ${error.message}`); process.exit(2) }

const out = resolve(options.out)
const browser = await launch()
const errors = []
const shots = []
const secs = (ms) => `${(ms / 1000).toFixed(2)}s`
try {
  const page = await browser.newPage({ viewport: { width: options.width, height: 900 }, deviceScaleFactor: 2 })
  page.on('pageerror', (error) => errors.push(`page: ${error.message}`))
  await page.goto(url, { waitUntil: 'domcontentloaded' })
  const target = page.locator(options.select).first()
  await target.waitFor({ state: 'visible', timeout: 15000 })
  await page.evaluate(() => document.fonts.ready)
  const start = Date.now()
  for (let i = 0; i < options.count; i++) {
    await page.waitForTimeout(Math.max(0, start + i * options.every - Date.now()))
    const at = Date.now() - start
    const buffer = await target.screenshot()
    const box = await target.boundingBox()
    shots.push({ label: secs(at), src: `data:image/png;base64,${buffer.toString('base64')}`, w: Math.round((box?.width || options.width) * 2), h: Math.round((box?.height || 0) * 2) })
  }
  if (!shots.length) throw new Error('no frames were captured')
  const frameW = Math.round(shots[0].w / 2)
  const gap = 12
  const fit = Math.floor(1800 / (frameW + gap))
  const cols = options.cols == null || options.cols === 'auto'
    ? Math.max(1, Math.min(shots.length, fit >= 3 ? fit : Math.max(fit, Math.round(Math.sqrt(shots.length * 2)))))
    : Math.min(options.cols, shots.length)
  const sheet = await browser.newPage({ viewport: { width: cols * (frameW + gap) + gap, height: 400 }, deviceScaleFactor: 2 })
  await sheet.setContent(`<body style="margin:0;background:#e9e9e9;font:11px system-ui">
  <div style="display:grid;grid-template-columns:repeat(${cols},${frameW}px);gap:${gap}px;padding:${gap}px">
  ${shots
    .map(
      (s) => `<figure style="margin:0;position:relative">
        <img src="${s.src}" style="width:${frameW}px;display:block;background:#fff">
        <figcaption style="position:absolute;left:4px;top:4px;padding:1px 4px;background:#000c;color:#fff;border-radius:3px">${s.label}</figcaption>
      </figure>`,
    )
    .join('')}
  </div></body>`)
  await sheet.waitForLoadState('load')
  mkdirSync(dirname(out), { recursive: true })
  await sheet.screenshot({ path: out, fullPage: true })
} catch (error) { errors.push(error.message) }
await browser.close()

for (const error of errors) console.error(`incomplete: ${error}`)
if (errors.length) {
  console.error('sheet: inconclusive; nothing was written')
  process.exitCode = 2
} else {
  console.log(`${shots.length} frames every ${options.every}ms of ${options.select} at ${options.width}px -> ${out}`)
  console.log(`sheet: complete; a sheet records frames and is not approval`)
}
