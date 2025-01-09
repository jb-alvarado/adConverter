<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen, type Event } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { load } from '@tauri-apps/plugin-store'
import { cloneDeep, isEqual, round } from 'lodash-es'
import { onBeforeMount, ref } from 'vue'

import { stringFormatter } from './composables/helper'
import { useStore } from './store/index.ts'

import AlertMsg from './components/AlertMsg.vue'
import EditConfig from './components/EditConfig.vue'
import EditPresets from './components/EditPresets.vue'
import EditPublisher from './components/EditPublisher.vue'
import EditTemplate from './components/EditTemplate.vue'
import HeaderMenu from './components/HeaderMenu.vue'
import LogWindow from './components/LogWindow.vue'
import MediaTable from './components/MediaTable.vue'

const { folderPath, filename, removeExtension, Logger } = stringFormatter()

const store = useStore()

const defaultTemplate: Template = {
    intro: '',
    outro: '',
    lower_thirds: [],
}
const currentTask = ref<Task | null>(null)

const targetFolder = ref<string | null>(null)
const targetSubfolder = ref(false)
const noProgressValues = ref(false)
const jobInProcess = ref(false)
const showTemplateEditor = ref(false)
const showPublisherEditor = ref(false)

const log = new Logger()

onBeforeMount(async () => {
    await invoke('load_config').catch((e) => {
        store.msgAlert('error', e, 5)
        log.error(e)
    })

    const config = await load('config.json', { autoSave: false })
    store.showTranscript = (await config.get('transcript_cmd')) ? true : false
    store.transcriptLanguages = (await config.get('transcript_lang')) ?? []
    store.publishPreset = (await config.get('publish_preset')) ?? ''

    if (store.transcriptLanguages.length === 0) {
        store.transcriptLanguages = [
            { name: 'None', code: 'none' },
            { name: 'Auto', code: 'auto' },
            { name: 'Multilingual', code: 'ml' },
            { name: 'German', code: 'de' },
            { name: 'English', code: 'en' },
            { name: 'Spanish', code: 'es' },
        ]

        await config.set('transcript_lang', store.transcriptLanguages)
        await config.save()
    }

    await invoke<Preset[]>('presets_get')
        .then((prs: Preset[]) => {
            for (const preset of prs) {
                store.presets.push(preset)
            }
        })
        .catch((e) => {
            store.msgAlert('error', e, 5)
            log.error(e)
        })
})

listen<Task>('task-active', (event: Event<Task>) => {
    for (const entry of store.taskList) {
        if (entry.path === event.payload.path) {
            entry.active = true
            store.processPath = filename(entry.path)
        }
    }
})

listen<Task>('task-finish', (event: Event<Task>) => {
    for (let i = 0; i < store.taskList.length; i++) {
        if (jobInProcess.value && store.taskList[i].path === event.payload.path) {
            store.progressAll = round(((i + 1) * 100) / store.taskList.length)
            store.taskList[i].active = false
            store.taskList[i].finished = true

            if (i === store.taskList.length - 1) {
                jobInProcess.value = false
                store.jobsDone = true
            } else {
                taskSendNext()
            }

            break
        }
    }

    if (
        !store.taskList.some(
            (task: Task) => task.presets.length > 0 || (task.transcript && task.transcript != 'none') || task.publish
        )
    ) {
        jobInProcess.value = false
        store.jobsDone = true
    }
})

listen<String>('lufs-progress', async (event: Event<FFmpegProgress>) => {
    noProgressValues.value = false
    store.progressCurrent = event.payload.elapsed_pct
    store.processMsg = `<strong>Analyze (${event.payload.title} ${event.payload.speed} Speed): </strong>`
})

listen<String>('preset-start', async (event: Event<Preset>) => {
    noProgressValues.value = false
    for (const preset of currentTask.value?.presets) {
        if (preset.title === event.payload.title) {
            preset.output_path = event.payload.output_path
        }
    }
})

listen<String>('preset-progress', async (event: Event<FFmpegProgress>) => {
    store.progressCurrent = event.payload.elapsed_pct
    store.processMsg = `<strong>Encode (${event.payload.title} ${event.payload.fps} FPS): </strong>`
})

listen<string>('transcript-start', async () => {
    noProgressValues.value = true
    store.processMsg = `<strong>Transcript: </strong>`
})

listen<string>('transcript-progress', async (event: Event<string>) => {
    noProgressValues.value = false
    store.progressCurrent = parseFloat(event.payload)
    store.processMsg = `<strong>Transcript: </strong>`
})

listen<String>('preset-finish', async (event: Event<Preset>) => {
    store.progressCurrent = 100
    store.processMsg = `<strong>Done (${event.payload.title}): </strong>`

    const index = currentTask.value.presets.findIndex((item: Task) => item.name === event.payload.name)
    currentTask.value.presets.splice(index, 1)
})

listen<string>('logging', (event: Event<string>) => {
    store.logContent.push(event.payload)

    if (event.payload.includes('[ERROR]')) {
        store.msgAlert('error', event.payload.replace('[ERROR]', ''), 5)
    }

    while (store.logContent.length > 10000) {
        store.logContent.shift()
    }
})

async function getDir() {
    const path = store.taskList[store.taskList.length - 1]?.path
    let options = {
        multiple: false,
        directory: true,
    } as any

    if (path) {
        options.defaultPath = folderPath(path)
    }

    targetFolder.value = (await open(options)) as string | null
}

async function taskSendNext() {
    for (const task of store.taskList) {
        if (!task.finished) {
            if (task.presets.length === 0 && task.transcript === 'none' && !task.publish) {
                store.msgAlert('warning', 'No transcription, preset or publisher selected!', 3)
                break
            }

            jobInProcess.value = true
            task.active = true
            task.target = targetFolder.value
            task.target_subfolder = targetSubfolder.value

            if (!task.template.intro && !task.template.outro && task.template.lower_thirds.length === 0) {
                task.template = null
            }

            currentTask.value = task
            await invoke<Task>('task_send', { task }).catch((e) => {
                store.msgAlert('error', e, 5)
                log.error(e)
            })
            break
        }
    }
}

async function jobRun() {
    if (jobInProcess.value) {
        jobInProcess.value = false

        await invoke<Task>('task_cancel', { task: currentTask.value })
            .then(() => {
                currentTask.value.active = false
                currentTask.value.finished = false
            })
            .catch((e) => {
                store.msgAlert('error', e, 5)
                log.error(e)
            })
    } else {
        // start encoding job
        await invoke('task_start').catch((e) => {
            store.msgAlert('error', e, 5)
            log.error(e)
        })

        await taskSendNext()
    }
}

function editTemplate(task: Task) {
    currentTask.value = task
    showTemplateEditor.value = true

    store.currentTemplate = cloneDeep(task.template)
}

async function saveTemplate(update: boolean) {
    if (!update) {
        showTemplateEditor.value = false
    } else if (isEqual(defaultTemplate, store.currentTemplate)) {
        showTemplateEditor.value = false
    } else {
        const path = removeExtension(currentTask.value.path) + '.json'

        await invoke<Task>('template_save', { template: store.currentTemplate, path })
            .then(() => {
                store.msgAlert('success', `Save template ${filename(path)} success.`, 3)

                currentTask.value.template = cloneDeep(store.currentTemplate)
                store.currentTemplate.value = cloneDeep(defaultTemplate)
                showTemplateEditor.value = false
            })
            .catch((e) => {
                store.msgAlert('error', e, 5)
                log.error(e)
            })
    }
}

function editPublisher(task: Task) {
    currentTask.value = task
    showPublisherEditor.value = true

    // store.currentPublisher = cloneDeep(task.template)
}

function savePublisher(_save: boolean) {
    showPublisherEditor.value = false

    // store.currentPublisher = cloneDeep(task.template)
}
</script>

<template>
    <div class="flex flex-col h-screen justify-between select-none cursor-default">
        <HeaderMenu :logger="log" />
        <main class="mb-auto bg-base-300 w-full h-full overflow-x-hidden overflow-y-auto">
            <div class="relative bg-base-200 h-full">
                <MediaTable :logger="log" :editTemplate="editTemplate" :editPublisher="editPublisher" />
                <LogWindow v-if="store.openLog" />
                <EditConfig v-if="store.showConfig" :logger="log" />
                <EditPresets v-if="store.showPresets" :logger="log" />
            </div>
        </main>

        <footer class="relative z-30 h-[100px]">
            <div v-if="!store.openLog" class="absolute w-full flex justify-center -top-[12px]">
                <button
                    class="w-20 h-[12px] min-h-[12px] btn bg-base-100 rounded-none border-b-0 hover:border-zinc-600 rounded-t-box border-t border-zinc-600 hover:text-base-content/50"
                    title="Open Logging"
                    @click="store.openLog = true"
                >
                    <i class="bi-chevron-compact-up" />
                </button>
            </div>
            <div class="flex bg-base-100 border-t border-zinc-600">
                <div class="flex justify-center m-auto item-center w-2/5">
                    <div class="container px-4 flex flex-col gap-0 mb-1">
                        <div class="flex items-center gap-4">
                            <div class="font-semibold w-15">Current:</div>
                            <div class="relative grow flex items-center">
                                <progress
                                    v-if="noProgressValues"
                                    class="progress progress-accent rounded-sm [&::-webkit-progress-value]:rounded-sm h-4"
                                />
                                <template v-else>
                                    <progress
                                        class="progress progress-accent rounded-sm [&::-webkit-progress-value]:rounded-sm h-4"
                                        :value="store.progressCurrent"
                                        max="100"
                                    />
                                    <div class="absolute w-full font-semibold text-center text-xs">
                                        {{ store.progressCurrent }}%
                                    </div>
                                </template>
                            </div>
                        </div>
                        <div class="flex items-center gap-4 mt-2">
                            <div class="font-semibold w-15">Over All:</div>
                            <div class="relative grow flex items-center">
                                <progress
                                    class="progress progress-accent rounded-sm [&::-webkit-progress-value]:rounded-sm h-4"
                                    :value="store.progressAll"
                                    max="100"
                                />
                                <div class="absolute w-full font-semibold text-center text-xs">
                                    {{ store.progressAll }}%
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="flex justify-center m-auto item-center w-3/5">
                    <div class="container flex">
                        <div class="p-4 flex flex-col gap-1 w-[calc(100%-102px)]">
                            <div class="flex items-center">
                                <div
                                    class="grow font-semibold truncate pr-2 h-[25px]"
                                    v-html="store.processMsg + store.processPath"
                                />

                                <label class="label cursor-pointer pr-0 pt-0 pb-[5px]" :disabled="jobInProcess">
                                    <span class="label-text mr-2">Subfolder</span>
                                    <input
                                        type="checkbox"
                                        v-model="targetSubfolder"
                                        class="checkbox checkbox-sm rounded-sm"
                                    />
                                </label>
                            </div>
                            <div class="flex items-end">
                                <label class="cursor-pointer join w-full">
                                    <input
                                        v-model="targetFolder"
                                        type="text"
                                        class="input input-sm input-bordered rounded-sm join-item w-full"
                                        :class="{ 'disabled:input-bordered': jobInProcess }"
                                        :disabled="jobInProcess"
                                    />
                                    <button
                                        class="btn btn-sm border-[oklch(var(--bc)/0.2)] hover:border-[oklch(var(--bc)/0.15)] rounded-sm join-item"
                                        @click="getDir()"
                                        :disabled="jobInProcess"
                                    >
                                        Save As
                                    </button>
                                </label>
                            </div>
                        </div>
                        <div class="flex items-end pb-4 pr-4">
                            <button
                                class="btn btn-lg border-[oklch(var(--bc)/0.2)] hover:border-[oklch(var(--bc)/0.15)] rounded-sm w-20"
                                @click="jobRun()"
                            >
                                {{ jobInProcess ? 'Cancel' : 'Run' }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </footer>
        <AlertMsg v-if="!store.openLog" />
        <EditTemplate :show="showTemplateEditor" :currentTask="currentTask" :saveTemplate="saveTemplate" />
        <EditPublisher
            v-if="showPublisherEditor"
            :show="showPublisherEditor"
            :logger="log"
            :currentTask="currentTask"
            :savePublisher="savePublisher"
        />
    </div>
</template>
