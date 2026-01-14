<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen, type Event } from '@tauri-apps/api/event'
import { getMatches } from '@tauri-apps/plugin-cli'
import { readText } from '@tauri-apps/plugin-clipboard-manager'
import { cloneDeep, isEqual } from 'lodash-es'
import Multiselect from '@vueform/multiselect'

import { stringFormatter, useVariables } from '../composables/helper'
import { useStore } from '../store/index.ts'

import TimePicker from '../components/TimePicker.vue'
import ContextMenu from '../components/ContextMenu.vue'

const store = useStore()
const { allFade, allLufs, allTranscript, presetList } = storeToRefs(useStore())
const { filename, extension, secToMin } = stringFormatter()
const { multiSelectClasses } = useVariables()

const defaultContext = [
    { label: 'Add Files', action: 'add' },
    { label: 'Edit Template', action: 'edit' },
    { label: 'Delete', action: 'delete' },
    { label: 'Reset', action: 'reset' },
]

const showMenu = ref(false)
const menuX = ref(0)
const menuY = ref(0)
const selectedTask = ref<Task | undefined>(undefined)

const contextMenuActions = ref(cloneDeep(defaultContext))

const prop = defineProps({
    logger: {
        type: Object,
        default() {
            return {}
        },
    },
    editTemplate: {
        type: Function,
        default() {
            return ''
        },
    },
    editPublisher: {
        type: Function,
        default() {
            return ''
        },
    },
    addFiles: {
        type: Function,
        default() {
            return
        },
    },
})

onMounted(async () => {
    const matches = await getMatches()
    const files = matches.args?.files?.value

    if (Array.isArray(files)) {
        for (const path of files) {
            addFile(path)
        }
    }
})

listen('tauri://drag-drop', async (event: Event<any>) => {
    for (const path of event.payload.paths) {
        await addFile(path)
    }
})

const showContextMenu = (event: any, task: Task | undefined) => {
    selectedTask.value = task
    if (!task) {
        contextMenuActions.value = contextMenuActions.value.filter(
            (action) => action.action !== 'edit' && action.action !== 'delete'
        )
    } else {
        contextMenuActions.value = cloneDeep(defaultContext)
    }

    showMenu.value = true
    menuX.value = event.clientX
    menuY.value = event.clientY
}

const closeContextMenu = () => {
    setTimeout(() => {
        showMenu.value = false
    }, 200)
}

function handleActionClick(action: any) {
    switch (action) {
        case 'add':
            prop.addFiles()
            break
        case 'edit':
            if (selectedTask.value) {
                prop.editTemplate(selectedTask.value)
            }
            break
        case 'reset':
            window.location.reload()
            break
        case 'delete':
            if (selectedTask.value) {
                const index = store.taskList.findIndex((t: Task) => t.path === selectedTask.value.path)
                if (index !== -1) {
                    store.taskList.splice(index, 1)
                }
            }
            break
    }

    showMenu.value = false
}

async function getClipboard() {
    const content = await readText()
    const regex = /(["'])(.*?[^\\])\1|([^\s"']+(?:\\\s[^\s"']*)*)/g

    let match
    while ((match = regex.exec(content)) !== null) {
        let path = match[2] || match[3] || ''
        path = path.replace(/\\ /g, ' ')
        await addFile(path)
    }
}

async function addFile(path: string) {
    const ext = extension(path)

    if (!store.ALLOWED_EXTENSIONS.includes(ext)) {
        store.msgAlert('error', `Extension <strong>${ext}</strong> not Allowed!`, 5)
        return
    }

    if (store.taskList.some((task: Task) => task.path === path)) {
        store.msgAlert('warning', `File: <strong>${filename(path)}</strong> already in list!`, 5)
        return
    }

    const task = cloneDeep(store.defaultTask)
    task.path = path

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

async function changeBoolean(task: Task | undefined, all: { value: boolean }, field: keyof Task & ('fade' | 'lufs')) {
    if (task) {
        task[field] = !task[field]

        const count = store.taskList.reduce((count: number, item: Task) => {
            return count + (item[field] ? 1 : 0)
        }, 0)

        all.value = count === store.taskList.length
    } else {
        all.value = !all.value

        for (const task of store.taskList) {
            task[field] = all.value
        }
    }
}

async function changeTranscription(task: Task | undefined) {
    if (task) {
        const count = store.taskList.reduce((count: number, item: Task) => {
            return count + (item.transcript === task.transcript ? 1 : 0)
        }, 0)

        allTranscript.value = count === store.taskList.length ? task.transcript : 'none'
    } else {
        for (const task of store.taskList) {
            if (!task.active) {
                task.transcript = allTranscript.value
            }
        }
    }
}

function changePresets(task: Task | undefined) {
    nextTick(async () => {
        if (task) {
            const count = store.taskList.reduce((count: number, item: Task) => {
                return count + (isEqual(item.presets, task.presets) ? 1 : 0)
            }, 0)

            if (count !== store.taskList.length) {
                presetList.value.length = 0
            } else {
                presetList.value = cloneDeep(task.presets)
            }
        } else {
            for (const task of store.taskList) {
                if (!task.active) {
                    task.presets = cloneDeep(presetList.value)
                }
            }
        }
    })
}
</script>
<template>
    <div
        class="w-full h-full relative"
        @keydown.prevent.ctrl.v="getClipboard"
        @keydown.prevent.meta.v="getClipboard"
        tabindex="0"
    >
        <div v-if="store.taskList.length > 0" class="flex flex-col h-full">
            <table class="bg-base-300 border-collapse table table-pin-rows table-fixed z-10">
                <thead class="top-0 sticky z-40">
                    <tr class="bg-base-200">
                        <th class="p-0 w-auto border-r h-full border-zinc-700">
                            <div
                                class="px-2 flex items-center w-[calc(100%+1px)] h-10.25 border-b border-r border-zinc-700"
                            >
                                File
                            </div>
                        </th>
                        <th class="p-0 w-22 border-r border-zinc-700">
                            <div class="px-2 flex items-center w-22 h-10.25 border-b border-r border-zinc-700">
                                Duration
                            </div>
                        </th>
                        <th class="p-0 w-30.5 border-r border-zinc-700">
                            <div class="px-2 flex items-center w-30.5 h-10.25 border-b border-r border-zinc-700">
                                In
                            </div>
                        </th>
                        <th class="p-0 w-30.5 border-r border-zinc-700">
                            <div class="px-1 flex items-center w-30.5 h-10.25 border-b border-r border-zinc-700">
                                Out
                            </div>
                        </th>
                        <th class="p-0 w-19 border-r border-zinc-700">
                            <label
                                class="label cursor-pointer max-w-xs px-2 h-10.25 w-19 border-b border-r border-zinc-700 text-base-content/60"
                            >
                                <input
                                    type="checkbox"
                                    :checked="allFade.value"
                                    class="checkbox checkbox-xs checked:shadow-none rounded-xs"
                                    @change="changeBoolean(undefined, allFade, 'fade')"
                                />
                                <span class="pl-1 me-2">Fade</span>
                            </label>
                        </th>
                        <th class="p-0 w-19 border-r border-zinc-700">
                            <label
                                class="label cursor-pointer max-w-xs px-2 h-10.25 w-19 border-b border-r border-zinc-700 text-base-content/60"
                            >
                                <input
                                    type="checkbox"
                                    :checked="allLufs.value"
                                    class="checkbox checkbox-xs checked:shadow-none rounded-xs"
                                    @change="changeBoolean(undefined, allLufs, 'lufs')"
                                />
                                <span class="pl-1 me-2">Lufs</span>
                            </label>
                        </th>
                        <th v-if="store.showTranscript" class="p-0 w-34.5 border-r border-zinc-700">
                            <div class="px-1 py-0.75 w-34.5 h-10.25 border-b border-r border-zinc-700">
                                <select
                                    v-model="allTranscript"
                                    class="select select-sm select-bordered rounded-xs focus:border-base-content/30 focus:outline-base-content/30 w-full max-w-xs"
                                    @change="changeTranscription(undefined)"
                                >
                                    <option disabled selected>Transcript</option>
                                    <template v-for="lang in store.transcriptLanguages" :key="lang.code">
                                        <option :value="lang.code">{{ lang.name }}</option>
                                    </template>
                                </select>
                            </div>
                        </th>
                        <th class="p-0 border-r border-zinc-700">
                            <div
                                class="relative z-10 px-1 w-[calc(100%+1px)] py-0.75 h-10.25 border-b border-r border-zinc-700"
                                :style="{ minWidth: `${presetList.length * 58 + 9}px` }"
                            >
                                <Multiselect
                                    v-model="presetList"
                                    :options="store.presets"
                                    :close-on-select="true"
                                    :object="true"
                                    mode="tags"
                                    label="title"
                                    valueProp="title"
                                    :can-clear="true"
                                    :searchable="true"
                                    class="input-sm rounded-xs w-full"
                                    :classes="multiSelectClasses"
                                    placeholder="[Presets]"
                                    @select="changePresets(undefined)"
                                    @deselect="changePresets(undefined)"
                                    @clear="changePresets(undefined)"
                                >
                                </Multiselect>
                            </div>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <tr
                        v-for="task in store.taskList"
                        :key="task.path"
                        class="p-0 m-0 border-b border-zinc-700 bg-base-200 odd:bg-base-300 hover:bg-base-100"
                        :class="{ 'opacity-50': task.active || task.finished }"
                        @contextmenu.prevent="showContextMenu($event, task)"
                    >
                        <td class="p-0 border-r border-zinc-700">
                            <div class="m-0 p-2.5 truncate">
                                {{ filename(task.path) }}
                            </div>
                        </td>
                        <td class="p-0 border-r border-zinc-700">
                            <div class="p-2">
                                {{ secToMin(task.probe.format.duration) }}
                            </div>
                        </td>
                        <td class="p-0 border-r border-zinc-700">
                            <div class="p-1">
                                <TimePicker v-model="task.in" :disabled="task.active || task.finished" />
                            </div>
                        </td>
                        <td class="p-0 border-r border-zinc-700">
                            <div class="p-1">
                                <TimePicker v-model="task.out" :disabled="task.active || task.finished" />
                            </div>
                        </td>
                        <td class="p-0 border-r border-zinc-700">
                            <div class="p-2 flex items-center">
                                <input
                                    type="checkbox"
                                    :checked="task.fade"
                                    class="checkbox checkbox-xs checked:shadow-none rounded-xs"
                                    @change="changeBoolean(task, allFade, 'fade')"
                                    :disabled="task.active || task.finished"
                                />
                            </div>
                        </td>
                        <td class="p-0 border-r border-zinc-700">
                            <div class="p-2 flex items-center">
                                <input
                                    type="checkbox"
                                    :checked="task.lufs"
                                    class="checkbox checkbox-xs checked:shadow-none rounded-xs"
                                    @change="changeBoolean(task, allLufs, 'lufs')"
                                    :disabled="task.active || task.finished"
                                />
                            </div>
                        </td>
                        <th v-if="store.showTranscript" class="py-0 px-1 border-r border-zinc-700">
                            <select
                                v-model="task.transcript"
                                class="select select-sm select-bordered focus:border-base-content/30 focus:outline-base-content/30 rounded-xs w-full max-w-xs"
                                @change="changeTranscription(task)"
                                :disabled="task.active || task.finished"
                            >
                                <option disabled selected>Transcript</option>
                                <template v-for="lang in store.transcriptLanguages" :key="lang.code">
                                    <option :value="lang.code">{{ lang.name }}</option>
                                </template>
                            </select>
                        </th>
                        <th class="py-0 px-1 border-r border-zinc-700">
                            <div :style="{ minWidth: `${presetList.length * 58}px` }">
                                <Multiselect
                                    v-model="task.presets"
                                    :options="store.presets"
                                    :close-on-select="true"
                                    :object="true"
                                    mode="tags"
                                    label="title"
                                    valueProp="title"
                                    :can-clear="true"
                                    :searchable="true"
                                    class="presetDiv input-sm rounded-xs w-full"
                                    :classes="multiSelectClasses"
                                    placeholder="[Presets]"
                                    @select="changePresets(task)"
                                    @deselect="changePresets(task)"
                                    @clear="changePresets(task)"
                                    :disabled="task.active || task.finished"
                                >
                                </Multiselect>
                            </div>
                        </th>
                        <!-- <td class="p-0 border-r border-zinc-700">
                    <div class="p-1">
                        <button class="btn btn-primary btn-sm rounded-xs p-1" @click="editPublisher(task)">
                            <i class="bi-cloud-arrow-up text-xl"></i>
                        </button>
                    </div>
                </td> -->
                    </tr>
                </tbody>
            </table>
            <div
                class="w-full flex-1"
                @contextmenu.prevent="showContextMenu($event, undefined)"
                @blur="closeContextMenu"
            ></div>
        </div>
        <div
            v-else
            class="h-full w-full flex justify-center items-center"
            @contextmenu.prevent="showContextMenu($event, undefined)"
            @blur="closeContextMenu"
        >
            <i class="bi-box-arrow-in-down text-[90px]" />
        </div>

        <div class="w-full h-full fixed z-40 top-0 left-0" @click="closeContextMenu" v-if="showMenu" />

        <ContextMenu
            v-if="showMenu"
            :actions="contextMenuActions"
            @action-clicked="handleActionClick"
            :x="menuX"
            :y="menuY"
        />
    </div>
</template>
