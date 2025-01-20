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

// async function updater() {
//     if (update.value) {
//         // console.log(`found update ${update.value.version} from ${update.value.date} with notes ${update.value.body}`)

//         let downloaded = 0
//         let contentLength = 0
//         await update.value.downloadAndInstall((event) => {
//             switch (event.event) {
//                 case 'Started':
//                     contentLength = event.data.contentLength ?? 0
//                     store.msgAlert('info', `Started downloading ${formatBytes(contentLength)}`, 3)
//                     break
//                 case 'Progress':
//                     downloaded += event.data.chunkLength

//                     store.progressCurrent = round((downloaded * 100) / contentLength)
//                     store.processMsg = `Update to ${update.value?.version}`
//                     break
//                 case 'Finished':
//                     store.processMsg = 'Install update'
//                     break
//             }
//         })

//         store.processMsg = ''
//         store.progressCurrent = 0
//         store.msgAlert('success', `Update done. Restart to apply changes.`, 3)
//     }
// }

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
            <div class="flex items-stretch z-[60]">
                <div class="dropdown dropdown-start">
                    <button tabindex="0" role="button" class="btn btn-xs btn-ghost !rounded-none">File</button>
                    <ul tabindex="0" class="menu dropdown-content bg-base-100 rounded-sm w-36 mt-1 p-0 shadow">
                        <li><button class="hover:rounded-sm !rounded-sm" @click="addFiles()">Open</button></li>
                        <li><button class="hover:rounded-sm !rounded-sm" @click="closeApp()">Close</button></li>
                    </ul>
                </div>
                <div class="dropdown dropdown-start">
                    <button tabindex="0" role="button" class="btn btn-xs btn-ghost !rounded-none">Option</button>
                    <ul tabindex="0" class="menu dropdown-content bg-base-100 rounded-sm w-36 mt-1 p-0 shadow">
                        <li v-if="update">
                            <div class="hover:rounded-sm !rounded-sm" title="Download and install update">
                                Update available ({{ update.version }})
                            </div>
                        </li>
                        <li>
                            <button class="hover:rounded-sm !rounded-sm" @click="openCloseConfig($event, 'presets')">
                                Presets
                            </button>
                        </li>
                        <li>
                            <button class="hover:rounded-sm !rounded-sm" @click="openCloseConfig($event, 'config')">
                                Settings
                            </button>
                        </li>
                        <li>
                            <label class="label cursor-pointer max-w-xs hover:rounded-sm justify-normal">
                                <input
                                    type="checkbox"
                                    v-model="shutdown"
                                    class="checkbox checkbox-xs rounded-sm"
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
