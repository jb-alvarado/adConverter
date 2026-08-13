import assert from 'node:assert/strict'
import test from 'node:test'

import { findTaskById, finishedProgress, hasPendingTasks, nextPendingTask, taskHasWork } from '../src/composables/taskQueue.ts'

function task(overrides: Record<string, unknown> = {}) {
    return {
        id: crypto.randomUUID(),
        url: null,
        presets: [],
        transcript: 'none',
        publish: null,
        active: false,
        finished: false,
        ...overrides,
    } as Task
}

test('finds a task by stable id when paths are identical', () => {
    const first = task({ id: 'first', path: '/tmp/video.mp4', finished: true })
    const second = task({ id: 'second', path: '/tmp/video.mp4', active: true })
    assert.equal(findTaskById([first, second], 'second'), second)
})

test('selects only pending and inactive queue entries', () => {
    const active = task({ active: true })
    const done = task({ finished: true })
    const pending = task()
    assert.equal(nextPendingTask([active, done, pending]), pending)
    assert.equal(hasPendingTasks([done, pending]), true)
    assert.equal(hasPendingTasks([done]), false)
})

test('recognizes downloads and configured processing as work', () => {
    assert.equal(taskHasWork(task({ url: 'https://example.test/video' })), true)
    assert.equal(taskHasWork(task({ presets: [{}] })), true)
    assert.equal(taskHasWork(task({ transcript: 'de' })), true)
    assert.equal(taskHasWork(task()), false)
})

test('calculates progress from completed entries instead of list position', () => {
    assert.equal(finishedProgress([task({ finished: true }), task(), task({ finished: true })]), 67)
    assert.equal(finishedProgress([]), 0)
})
