<script setup lang="ts">
import { ref, watch, onBeforeMount } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { storeToRefs } from 'pinia'
import { cloneDeep } from 'lodash-es'

import { stringFormatter } from '../composables/helper'
import { useStore } from '../store/index.ts'

const { folderPath, filename } = stringFormatter()

const store = useStore()
const { jobsDone } = storeToRefs(useStore())
const shutdown = ref(false)
const update = ref<Update | null>(null)

const prop = defineProps({
    logger: {
        type: Object,
        default() {
            return {}
        },
    },
})

onBeforeMount(async () => {
    update.value = await check()
})

watch([jobsDone], () => {
    if (store.jobsDone && shutdown.value) {
        shutdown_system()
    }
})

async function addFiles() {
    const path = store.taskList[store.taskList.length - 1]?.path
    let options = {
        multiple: true,
        directory: false,
        filters: [
            {
                name: 'File Types',
                extensions: store.ALLOWED_EXTENSIONS,
            },
        ],
    } as any

    if (path) {
        options.defaultPath = folderPath(path)
    }

    let files = (await open(options)) || []

    for (const file of files) {
        const task = cloneDeep(store.defaultTask)

        if (store.taskList.some((task: Task) => task.path === file)) {
            store.msgAlert('warning', `File: <strong>${filename(file)}</strong> already in list!`, 5)
            continue
        }

        task.path = file

        await invoke<Task>('file_drop', { task })
            .then((task: Task) => {
                if (!task.template) {
                    task.template = cloneDeep(store.defaultTemplate)
                }
                store.taskList.push(task)
            })
            .catch((e) => {
                store.msgAlert('error', e, 5)
                prop.logger.error(e)
            })
    }
}

async function resetApp($event: any,) {
    store.taskList.length = 0
    store.processMsg = ''
    store.processPath = ''
    store.progressAll = 0
    store.progressCurrent = 0
    store.allFade = { value: false }
    store.allLufs = { value: false }
    store.allTranscript = 'none'
    store.presetList = []
    store.jobInProcess = false

    setTimeout(() => {
        $event.target.blur()
    }, 60)
}

async function closeApp() {
    await getCurrentWindow().close()
}

function openCloseConfig($event: any, link: string) {
    if (link === 'config') {
        store.showPresets = false
        store.showConfig = !store.showConfig
    }
    if (link === 'presets') {
        store.showConfig = false
        store.showPresets = !store.showPresets
    }

    setTimeout(() => {
        $event.target.blur()
    }, 60)
}

async function shutdown_system() {
    await invoke('shutdown_system').catch((e) => {
        store.msgAlert('error', e, 5)
        prop.logger.error(e)
    })
}
</script>
<template>
    <header class="bg-base-100 max-h-[42px] w-full">
        <div class="flex flex-1 justify-start">
            <div class="flex items-stretch z-60">
                <div class="dropdown dropdown-start">
                    <button tabindex="0" role="button" class="btn btn-xs btn-ghost rounded-none!">File</button>
                    <ul tabindex="0" class="menu dropdown-content bg-base-100 rounded-xs w-36 mt-1 p-0 shadow-sm">
                        <li><button class="hover:rounded-xs rounded-xs!" @click="addFiles()">Open</button></li>
                        <li><button class="hover:rounded-xs rounded-xs!" @click="resetApp">Reset</button></li>
                        <li><button class="hover:rounded-xs rounded-xs!" @click="closeApp()">Close</button></li>
                    </ul>
                </div>
                <div class="dropdown dropdown-start">
                    <button tabindex="0" role="button" class="btn btn-xs btn-ghost rounded-none!">Option</button>
                    <ul tabindex="0" class="menu dropdown-content bg-base-100 rounded-xs w-36 mt-1 p-0 shadow-sm">
                        <li v-if="update">
                            <div class="hover:rounded-xs rounded-xs!" title="Download and install update">
                                Update available ({{ update.version }})
                            </div>
                        </li>
                        <li>
                            <button class="hover:rounded-xs rounded-xs!" @click="openCloseConfig($event, 'presets')">
                                Presets
                            </button>
                        </li>
                        <li>
                            <button class="hover:rounded-xs rounded-xs!" @click="openCloseConfig($event, 'config')">
                                Settings
                            </button>
                        </li>
                        <li>
                            <label class="label cursor-pointer max-w-xs hover:rounded-xs justify-normal">
                                <input
                                    type="checkbox"
                                    v-model="shutdown"
                                    class="checkbox checkbox-xs checked:shadow-none rounded-xs"
                                    title="Shutdown after all jobs are done"
                                />
                                <span class="pl-3 me-2">Shutdown</span>
                            </label>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    </header>
</template>
