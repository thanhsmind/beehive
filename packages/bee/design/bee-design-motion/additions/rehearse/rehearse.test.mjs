import test from 'node:test'
import assert from 'node:assert/strict'
import { analyse, contentChanges } from './jumps-core.mjs'
import { layoutResult } from './layout-core.mjs'

const item = (id, name, x, y, w, h, parent = -1, next = -1, content = null) => [id, name, x, y, w, h, parent, next, content]

const frames = (series) => series.map((items, i) => ({ t: i * 16, mark: null, items, cameraMoving: 0 }))

const still = frames([
  [item(1, 'card', 0, 0, 100, 40)],
  [item(1, 'card', 0, 0, 100, 40)],
  [item(1, 'card', 0, 0, 100, 40)],
  [item(1, 'card', 0, 0, 100, 40)],
])

test('a box that jumps more than 2px in one frame is reported, with its axis and size', () => {
  const snapped = frames([
    [item(1, 'card', 0, 0, 100, 40)],
    [item(1, 'card', 0, 0, 100, 40)],
    [item(1, 'card', 10, 0, 100, 40)],
    [item(1, 'card', 10, 0, 100, 40)],
  ])
  const lines = analyse(snapped)
  assert.equal(lines.length, 1, lines.join('\n'))
  assert.match(lines[0], /^jump: card x \+10\.0px in one frame/)
  assert.deepEqual(analyse(still), [])
})

test('a box that resizes more than 2px in one frame is reported on its size axis', () => {
  const resized = frames([
    [item(1, 'chip', 0, 0, 100, 40)],
    [item(1, 'chip', 0, 0, 100, 40)],
    [item(1, 'chip', 0, 0, 100, 44)],
    [item(1, 'chip', 0, 0, 100, 44)],
  ])
  const lines = analyse(resized)
  assert.equal(lines.length, 1, lines.join('\n'))
  assert.match(lines[0], /jump: chip height \+4\.0px/)
})

test('a smooth glide is not a jump, however far it travels', () => {
  const glide = frames([
    [item(1, 'card', 0, 0, 100, 40)],
    [item(1, 'card', 5, 0, 100, 40)],
    [item(1, 'card', 10, 0, 100, 40)],
    [item(1, 'card', 15, 0, 100, 40)],
    [item(1, 'card', 20, 0, 100, 40)],
  ])
  assert.deepEqual(analyse(glide), [])
})

test('an element that appears is not a jump', () => {
  const appears = frames([
    [item(1, 'card', 0, 0, 100, 40)],
    [item(1, 'card', 0, 0, 100, 40)],
    [item(1, 'card', 0, 0, 100, 40), item(2, 'toast', 300, 0, 120, 40)],
    [item(1, 'card', 0, 0, 100, 40), item(2, 'toast', 300, 0, 120, 40)],
  ])
  assert.deepEqual(analyse(appears), [])
})

test('a frame that comes late is not judged', () => {
  const stalled = frames([
    [item(1, 'card', 0, 0, 100, 40)],
    [item(1, 'card', 0, 0, 100, 40)],
    [item(1, 'card', 0, 0, 100, 40)],
    [item(1, 'card', 40, 0, 100, 40)],
    [item(1, 'card', 40, 0, 100, 40)],
  ])
  for (const [i, frame] of stalled.entries()) frame.t = i === 3 ? 900 : i * 16
  assert.deepEqual(analyse(stalled), [])
})

test('a same-box text change is listed for manual review, not as a jump', () => {
  const changing = frames([
    [item(1, 'label', 0, 0, 100, 40, -1, -1, 'send|')],
    [item(1, 'label', 0, 0, 100, 40, -1, -1, 'sent|')],
  ])
  assert.deepEqual(analyse(changing), [])
  const lines = contentChanges(changing)
  assert.equal(lines.length, 1)
  assert.match(lines[0], /same-box content change: label/)
})

const scope = ({ findings = [], controls = 1, paddedContainers = 1, deferred = [], compositionCrops = 0 } = {}) => ({
  findings,
  deferred,
  checked: { controls, paddedContainers, compositionCrops },
})

const finding = { rule: 'control-block-fit', element: 'button', message: 'layout: button is 20px high' }

test('layoutResult maps every outcome to the contract exit codes', () => {
  const clean = layoutResult([scope()])
  assert.equal(clean.status, 'no-issues-detected')
  assert.equal(clean.exitCode, 0)

  const loose = layoutResult([scope({ findings: [finding] })])
  assert.equal(loose.status, 'issues')
  assert.equal(loose.exitCode, 0, 'findings alone do not fail without --strict')

  const strict = layoutResult([scope({ findings: [finding] })], [], true)
  assert.equal(strict.status, 'issues')
  assert.equal(strict.exitCode, 1, '--strict turns findings into exit 1')

  const errored = layoutResult([scope({ findings: [finding] })], ['page: boom'], true)
  assert.equal(errored.status, 'inconclusive')
  assert.equal(errored.exitCode, 2, 'an incomplete run always outranks --strict')

  const empty = layoutResult([scope({ controls: 0, paddedContainers: 0 })])
  assert.equal(empty.status, 'inconclusive')
  assert.equal(empty.exitCode, 2)
  assert.ok(empty.errors.some((line) => /no supported visible controls/.test(line)))

  const moving = layoutResult([scope({ deferred: ['button: control geometry is moving'] })])
  assert.equal(moving.status, 'inconclusive')
  assert.equal(moving.exitCode, 2)

  assert.equal(layoutResult([]).exitCode, 2, 'no scope at all is incomplete')
  assert.equal(layoutResult([scope()]).manualReviewRequired, true)
})

test('layoutResult counts work across scopes so a fully empty scan cannot pass', () => {
  const counted = layoutResult([scope({ controls: 0, paddedContainers: 0, compositionCrops: 2 })])
  assert.equal(counted.exitCode, 0)
  const uncounted = layoutResult([scope({ controls: 0, paddedContainers: 0 })])
  assert.equal(uncounted.exitCode, 2)
})
