<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'

import { stringFormatter } from '../composables/helper'
import { useStore } from '../store/index.ts'

import GenericModal from '../components/GenericModal.vue'
import TimePicker from './TimePicker.vue'

const { folderPath } = stringFormatter()
const store = useStore()

const prop = defineProps({
    logger: {
        type: Object,
        default() {
            return {}
        },
    },
    currentTask: {
        type: Object,
        default() {
            return {}
        },
    },
    saveTemplate: {
        type: Function,
        default() {
            return ''
        },
    },
    show: {
        type: Boolean,
        default() {
            return false
        },
    },
})

async function getIntro() {
    const path = prop.currentTask.path
    const folder = folderPath(path)
    let options = {
        multiple: false,
        directory: false,
        filters: [{
            name: 'Image',
            extensions: store.IMAGE_EXTENSIONS
        },
        {
            name: 'Video',
            extensions: store.VIDEO_EXTENSIONS
        }]
    } as any

    if (path) {
        options.defaultPath = folder
    }

    let intro = (await open(options)) as string | null

    store.currentTemplate.intro = intro?.replace(folder, '')
}

async function getOutro() {
    const path = prop.currentTask.path
    const folder = folderPath(path)
    let options = {
        multiple: false,
        directory: false,
        filters: [{
            name: 'Image',
            extensions: store.IMAGE_EXTENSIONS
        },
        {
            name: 'Video',
            extensions: store.VIDEO_EXTENSIONS
        }]
    } as any

    if (path) {
        options.defaultPath = folder
    }

    let outro = (await open(options)) as string | null

    store.currentTemplate.outro = outro?.replace(folder, '')
}

async function addLowerThird() {
    const path = prop.currentTask.path
    const folder = folderPath(path)
    let options = {
        multiple: false,
        directory: false,
    } as any

    if (path) {
        options.defaultPath = folder
    }

    let lowerThird = (await open(options)) as string | null

    store.currentTemplate.lower_thirds.push({
        path: lowerThird?.replace(folder, ''),
        duration: 0,
        position: ['00:00:10.000'],
    })
}

function removeLowerThirdPosition(li: number, i: number) {
    if (store.currentTemplate.lower_thirds[li].position.length === 1) {
        store.currentTemplate.lower_thirds.splice(li, 1)
    } else {
        store.currentTemplate.lower_thirds[li].position.splice(i, 1)
    }
}
</script>
<template>
    <GenericModal :show="show" title="Edit Template" :modal-action="saveTemplate">
        <div class="min-w-[700px]">
            <label class="cursor-pointer join w-full">
                <div class="label w-14">
                    <span class="label-text">Intro: </span>
                </div>
                <input
                    v-model="store.currentTemplate.intro"
                    type="text"
                    class="input input-sm input-bordered focus-within:border-base-content/30 focus-within:outline-base-content/30 rounded-xs join-item w-full"
                />
                <button
                    class="btn btn-sm border-base-content/30 hover:border-base-content/40 rounded-xs join-item"
                    @click="getIntro()"
                >
                    ...
                </button>
                <input
                    v-model="store.currentTemplate.intro_duration"
                    type="number"
                    class="max-w-14 input bg-base-200 input-bordered focus-within:border-base-content/30 focus-within:outline-base-content/30 input-sm join-item rounded-xs"
                    min="0"
                    max="99"
                    step="0.1"
                />
            </label>
            <label class="cursor-pointer join w-full">
                <div class="label w-14">
                    <span class="label-text">Outro: </span>
                </div>
                <input
                    v-model="store.currentTemplate.outro"
                    type="text"
                    class="input input-sm input-bordered focus-within:border-base-content/30 focus-within:outline-base-content/30 rounded-xs join-item w-full"
                />
                <button
                    class="btn btn-sm border-base-content/30 hover:border-base-content/40 rounded-xs join-item"
                    @click="getOutro()"
                >
                    ...
                </button>
                <input
                    v-model="store.currentTemplate.outro_duration"
                    type="number"
                    class="max-w-14 input bg-base-200 input-bordered focus-within:border-base-content/30 focus-within:outline-base-content/30 input-sm join-item rounded-xs"
                    min="0"
                    max="99"
                    step="0.1"
                />
            </label>
            <div class="flex gap-2 mt-1">
                <div class="flex flex-col gap-1 w-10">
                    <button
                        class="btn btn-xs rounded-xs border-base-content/30 hover:border-base-content/40"
                        title="Add lower third"
                        @click="addLowerThird()"
                    >
                        <i class="bi bi-plus leading-3 text-lg" />
                    </button>
                </div>
                <div class="grow bg-base-200 min-h-20 rounded-xs">
                    <div class="overflow-x-auto max-h-[300px]">
                        <table class="table table-zebra rounded-xs border-collapse">
                            <thead class="top-0 sticky z-50 bg-base-200">
                                <tr>
                                    <th class="min-w-[200px] p-0">
                                        <div class="w-full h-full border border-base-content/30 p-1">Path</div>
                                    </th>
                                    <th class="w-16 p-0">
                                        <div class="w-full h-full border border-base-content/30 p-1">
                                            Duration
                                        </div>
                                    </th>
                                    <th class="min-w-[220px] max-w-[400px] p-0">
                                        <div class="w-full h-full border border-base-content/30 p-1">
                                            Position
                                        </div>
                                    </th>
                                    <th class="w-[28px] p-0">
                                        <div class="w-full h-full border border-base-content/30 p-1">&nbsp;</div>
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr v-for="(lower, li) in store.currentTemplate.lower_thirds" :key="lower.path">
                                    <th class="border border-base-content/30 truncate max-w-96">{{ lower.path }}</th>
                                    <td class="border border-base-content/30 p-1">
                                        <input
                                            type="number"
                                            step="0.1"
                                            min="0.0"
                                            v-model.number="lower.duration"
                                            class="input input-sm input-bordered focus-within:border-base-content/30 focus-within:outline-base-content/30 rounded-xs max-w-14"
                                        />
                                    </td>
                                    <td class="border border-base-content/30 p-1">
                                        <div class="flex">
                                            <div class="grow flex flex-wrap max-w-[430px] pr-1">
                                                <div
                                                    v-for="(pos, i) in lower.position"
                                                    :key="pos"
                                                    class="w-[106px] relative"
                                                >
                                                    <TimePicker v-model="lower.position[i]" :isNumber="false" />
                                                    <button
                                                        class="absolute top-0 right-0 btn btn-sm w-2 px-2 btn-ghost rounded-xs"
                                                        title="Delete position"
                                                        @click="removeLowerThirdPosition(li, i)"
                                                    >
                                                        <i class="bi bi-x leading-3" />
                                                    </button>
                                                </div>
                                            </div>

                                            <div class="w-5 flex justify-center">
                                                <button
                                                    class="btn btn-sm btn-ghost border border-base-content/30 w-5 rounded-xs text-center"
                                                    title="Add position"
                                                    @click="lower.position.push('00:00:00.000')"
                                                >
                                                    <i class="bi bi-plus" />
                                                </button>
                                            </div>
                                        </div>
                                    </td>
                                    <td class="border border-base-content/30 p-1">
                                        <button
                                            class="btn btn-sm btn-ghost border border-base-content/30 w-5 rounded-xs text-center"
                                            title="Delete lower third"
                                            @click="store.currentTemplate.lower_thirds.splice(li, 1)"
                                        >
                                            <i class="bi bi-x-lg leading-3 text-center" />
                                        </button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    </GenericModal>
</template>
