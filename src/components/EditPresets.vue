<script setup lang="ts">
import { ref, computed } from 'vue'
import { cloneDeep, isEqual, omit } from 'lodash-es'
import { invoke } from '@tauri-apps/api/core'

import { useStore } from '../store/index.ts'

const store = useStore()

const textInput = ref()
const currentPreset = ref<Preset>()
const originalPreset = ref<Preset>()
const changed = ref(false)

const defaultPreset = {
    name: '',
    title: '',
    tooltip: '',
    filter_video: {},
    filter_audio: {},
    video: {},
    audio: {},
    container_video: null,
    container_audio: null,
}

const prop = defineProps({
    logger: {
        type: Object,
        default() {
            return {}
        },
    },
})

const preset = computed({
    get(): string {
        return JSON.stringify(currentPreset.value, replacer, 4)
    },
    set(val: string) {
        currentPreset.value = JSON.parse(val)
    },
})

const replacer = (key: string, value: any) => {
    if (['output_path', 'finished'].includes(key)) return undefined

    return value
}

function editPreset(preset: Preset) {
    currentPreset.value = preset
    originalPreset.value = cloneDeep(preset)
}

function addPreset() {
    currentPreset.value = cloneDeep(defaultPreset)
    originalPreset.value = cloneDeep(defaultPreset)
}

async function reset() {
    currentPreset.value = cloneDeep(originalPreset.value)
    changed.value = false
}

function insertAtCursor(text: string) {
    text = text || ''
    if (textInput.value.selectionStart || textInput.value.selectionStart === 0) {
        const startPos = textInput.value.selectionStart
        const endPos = textInput.value.selectionEnd

        textInput.value.value =
            textInput.value.value.substring(0, startPos) +
            text +
            textInput.value.value.substring(endPos, textInput.value.value.length)
        textInput.value.selectionStart = startPos + text.length
        textInput.value.selectionEnd = startPos + text.length
    } else {
        textInput.value.value += text
    }
}

function catchKeyDown(e: any) {
    // catch Tab key and add spaces
    if (e.keyCode == 9) {
        insertAtCursor('    ')
        if (e.preventDefault) {
            e.preventDefault()
        }
        return false
    }
}

function catchKeyUp() {
    if (
        isEqual(
            omit(originalPreset.value, ['finished', 'output_path']),
            omit(currentPreset.value, ['finished', 'output_path'])
        )
    ) {
        changed.value = false
    } else {
        changed.value = true
    }
}

function close() {
    reset()
    store.showPresets = false
}

async function savePreset() {
    if (changed.value) {
        await invoke<Task>('save_preset', { preset: currentPreset.value })
            .then(() => {
                store.msgAlert('success', `Preset <strong>${currentPreset.value.title}</strong> saved.`, 3)

                for (let i = 0; i < store.presets.length; i++) {
                    if (store.presets[i].name === currentPreset.value.name) {
                        store.presets[i] = cloneDeep(currentPreset.value)
                        break
                    }
                }

                originalPreset.value = cloneDeep(currentPreset.value)
                changed.value = false
            })
            .catch((e) => {
                store.msgAlert('error', e, 5)
                prop.logger.error(e)
            })
    }
}
</script>
<template>
    <div class="absolute z-40 top-0 left-0 w-full h-full bg-base-300 p-4">
        <div class="h-full max-h-[calc(100%-42px)]">
            <div class="flex gap-2 bg-base-200 pl-3 pr-[2px] py-[2px] items-center">
                <div ref="handler" class="font-bold text-lg truncate flex-1 w-0">Presets</div>
                <button class="btn btn-sm w-8 h-8 rounded-full text-center" @click="close()">
                    <i class="bi bi-x-lg leading-3" />
                </button>
            </div>
            <div class="bg-base-100 h-full rounded-sm flex flex-col p-2 gap-2">
                <div class="grow max-h-[calc(100%-42px)] flex flex-row gap-2">
                    <div class="bg-base-100 w-32">
                        <ul class="grow overflow-auto h-full">
                            <li v-for="preset in store.presets" :key="preset.name" class="truncate">
                                <button
                                    class="btn btn-xs w-full rounded-sm justify-start"
                                    :title="preset.tooltip"
                                    @click="editPreset(preset)"
                                >
                                    {{ preset.title }}
                                </button>
                            </li>
                        </ul>
                    </div>
                    <div class="bg-base-200 w-full overflow-auto h-full">
                        <div
                            class="grid text-sm [&>textarea]:text-inherit after:text-inherit [&>textarea]:resize-none [&>textarea]:[grid-area:1/1/2/2] after:[grid-area:1/1/2/2] after:whitespace-pre-wrap after:invisible after:content-[attr(data-cloned-val)_'_'] after:border h-full font-mono"
                        >
                            <textarea
                                type="text"
                                ref="textInput"
                                v-model="preset"
                                class="textarea textarea-bordered bg-base-200 rounded-sm w-full h-full text-sm leading-5"
                                :spellcheck="false"
                                @keydown="catchKeyDown"
                                @keyup="catchKeyUp"
                            />
                        </div>
                    </div>
                </div>
                <div class="h-[32px] flex">
                    <div class="grow">
                        <button class="btn btn-sm w-28 rounded-sm" title="Add preset" @click="addPreset()">+</button>
                    </div>
                    <div class="join">
                        <button class="btn btn-sm join-item rounded-sm" @click="reset()">Reset</button>
                        <button
                            class="btn btn-sm join-item rounded-sm"
                            :class="changed ? 'btn-error' : ''"
                            @click="savePreset"
                        >
                            Save
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
