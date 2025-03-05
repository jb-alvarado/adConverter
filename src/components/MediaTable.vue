<script setup lang="ts">
import { nextTick, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen, type Event } from '@tauri-apps/api/event'
import { getMatches } from '@tauri-apps/plugin-cli'
import { cloneDeep, isEqual } from 'lodash-es'
import Multiselect from '@vueform/multiselect'

import { stringFormatter, useVariables } from '../composables/helper'
import { useStore } from '../store/index.ts'

import TimePicker from '../components/TimePicker.vue'

const store = useStore()
const { allFade, allLufs, allTranscript, presetList } = storeToRefs(useStore())
const { filename, extension, secToMin } = stringFormatter()
const { multiSelectClasses } = useVariables()

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
    <table v-if="store.taskList.length > 0" class="bg-base-300 border-collapse table table-zebra table-fixed">
        <thead class="top-0 sticky z-40">
            <tr class="bg-base-200">
                <th class="p-0 w-auto border-r h-full border-zinc-700">
                    <div class="px-2 flex items-center w-[calc(100%+1px)] h-[41px] border-b border-r border-zinc-700">
                        File
                    </div>
                </th>
                <th class="p-0 w-[88px] border-r border-zinc-700">
                    <div class="px-2 flex items-center w-[88px] h-[41px] border-b border-r border-zinc-700">
                        Duration
                    </div>
                </th>
                <th class="p-0 w-[122px] border-r border-zinc-700">
                    <div class="px-2 flex items-center w-[122px] h-[41px] border-b border-r border-zinc-700">In</div>
                </th>
                <th class="p-0 w-[122px] border-r border-zinc-700">
                    <div class="px-1 flex items-center w-[122px] h-[41px] border-b border-r border-zinc-700">Out</div>
                </th>
                <th class="p-0 w-[76px] border-r border-zinc-700">
                    <label
                        class="label cursor-pointer max-w-xs px-2 h-[41px] w-[76px] border-b border-r border-zinc-700"
                    >
                        <input
                            type="checkbox"
                            :checked="allFade.value"
                            class="checkbox checkbox-xs rounded-sm"
                            @change="changeBoolean(undefined, allFade, 'fade')"
                        />
                        <span class="pl-3 me-2">Fade</span>
                    </label>
                </th>
                <th class="p-0 w-[76px] border-r border-zinc-700">
                    <label
                        class="label cursor-pointer max-w-xs px-2 h-[41px] w-[76px] border-b border-r border-zinc-700"
                    >
                        <input
                            type="checkbox"
                            :checked="allLufs.value"
                            class="checkbox checkbox-xs rounded-sm"
                            @change="changeBoolean(undefined, allLufs, 'lufs')"
                        />
                        <span class="pl-3 me-2">Lufs</span>
                    </label>
                </th>
                <th class="p-0 w-[41px] border-r border-zinc-700">
                    <div class="px-1 flex items-center w-[41px] h-[41px] border-b border-r border-zinc-700"></div>
                </th>
                <th v-if="store.showTranscript" class="p-0 w-[138px] border-r border-zinc-700">
                    <div class="px-1 py-[3px] w-[138px] h-[41px] border-b border-r border-zinc-700">
                        <select
                            v-model="allTranscript"
                            class="select select-sm select-bordered rounded-sm w-full max-w-xs"
                            @change="changeTranscription(undefined)"
                        >
                            <option disabled selected>Transcript</option>
                            <template v-for="lang in store.transcriptLanguages" :key="lang.value">
                                <option :value="lang.code">{{ lang.name }}</option>
                            </template>
                        </select>
                    </div>
                </th>
                <th class="p-0 border-r border-zinc-700">
                    <div
                        class="relative z-10 px-1 w-[calc(100%+1px)] py-[3px] h-[41px] border-b border-r border-zinc-700"
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
                            class="input-sm rounded-sm w-full"
                            :classes="multiSelectClasses"
                            placeholder="[Presets]"
                            @select="changePresets(undefined)"
                            @deselect="changePresets(undefined)"
                            @clear="changePresets(undefined)"
                        >
                        </Multiselect>
                    </div>
                </th>
                <!-- <th class="p-0 w-[41px] border-r border-zinc-700">
                    <div class="px-1 flex items-center w-[41px] h-[41px] border-b border-r border-zinc-700"></div>
                </th> -->
            </tr>
        </thead>
        <tbody>
            <tr
                v-for="task in store.taskList"
                :key="task.path"
                class="p-0 m-0 border-b border-zinc-700"
                :class="{ 'opacity-50': task.active || task.finished }"
            >
                <td class="p-0 border-r border-zinc-700">
                    <div class="m-0 p-[10px] truncate">
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
                            class="checkbox checkbox-xs rounded-sm"
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
                            class="checkbox checkbox-xs rounded-sm"
                            @change="changeBoolean(task, allLufs, 'lufs')"
                            :disabled="task.active || task.finished"
                        />
                    </div>
                </td>
                <td class="p-0 border-r border-zinc-700">
                    <div class="p-1">
                        <button class="btn btn-primary btn-sm rounded-sm p-1" @click="editTemplate(task)">
                            <i class="bi-collection text-xl"></i>
                        </button>
                    </div>
                </td>
                <th v-if="store.showTranscript" class="py-0 px-1 border-r border-zinc-700">
                    <select
                        v-model="task.transcript"
                        class="select select-sm select-bordered rounded-sm w-full max-w-xs"
                        @change="changeTranscription(task)"
                        :disabled="task.active || task.finished"
                    >
                        <option disabled selected>Transcript</option>
                        <template v-for="lang in store.transcriptLanguages" :key="lang.value">
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
                            class="input-sm rounded-sm w-full"
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
                        <button class="btn btn-primary btn-sm rounded-sm p-1" @click="editPublisher(task)">
                            <i class="bi-cloud-arrow-up text-xl"></i>
                        </button>
                    </div>
                </td> -->
            </tr>
        </tbody>
    </table>
    <div v-else class="h-full w-full flex justify-center items-center">
        <i class="bi-box-arrow-in-down text-[90px]" />
    </div>
</template>
