<script setup lang="ts">
import { ref, onBeforeMount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { LazyStore } from '@tauri-apps/plugin-store'

import { useStore } from '../store/index.ts'

const config = new LazyStore('config.json', { autoSave: false })
const store = useStore()

const prop = defineProps({
    logger: {
        type: Object,
        default() {
            return {}
        },
    },
})

const copyright = ref<string | undefined>('')
const lufs = ref<LufsConfig>({
    i: 0,
    lra: 0,
    tp: 0,
})
const transcript_cmd = ref('')

onBeforeMount(async () => {
    copyright.value = await config.get('copyright')
    lufs.value = await config.get('lufs')
    transcript_cmd.value = (await config.get('transcript_cmd')) ?? ''
})

function addLang() {
    store.transcriptLanguages.push({ name: '', code: '' })
}

function deleteLang(lang: TLang) {
    store.transcriptLanguages = store.transcriptLanguages.filter((l) => l.code !== lang.code)
}

async function saveConfig() {
    await config.set('copyright', copyright.value)
    await config.set('lufs', lufs.value)
    await config.set('transcript_cmd', transcript_cmd.value)
    await config.set('transcript_lang', store.transcriptLanguages)
    await config.set('publish_preset', store.publishPreset)
    await config.save()

    store.showConfig = false
    store.showTranscript = transcript_cmd.value ? true : false

    await invoke('save_config')
        .then(() => {
            store.msgAlert('success', 'Save config succeeded', 5)
        })
        .catch((e) => {
            store.msgAlert('error', e, 5)
            prop.logger.error(e)
        })
}

async function cancel() {
    await config.reload()

    store.showConfig = false
}
</script>
<template>
    <div class="absolute z-40 top-0 left-0 w-full h-full bg-base-300 p-4">
        <div class="bg-base-100 h-full rounded-sm flex flex-col p-2">
            <div class="grow flex flex-col gap-2 max-h-[calc(100%-36px)] overflow-auto">
                <div class="min-h-36 flex gap-2">
                    <div class="bg-base-200 p-2">
                        <strong>Audio</strong>
                        <div>
                            LUFS:
                            <label class="label max-w-xs justify-normal p-0">
                                <input
                                    type="number"
                                    v-model="lufs.i"
                                    min="-70.0"
                                    max="-5.0"
                                    step="0.1"
                                    class="input input-xs w-20 rounded-sm"
                                />
                                <span class="pl-3 me-2">Integrated loudness</span>
                            </label>
                            <label class="label max-w-xs justify-normal px-0 py-1">
                                <input
                                    type="number"
                                    v-model="lufs.lra"
                                    min="1.0"
                                    max="50.0"
                                    step="0.1"
                                    class="input input-xs w-20 rounded-sm"
                                />
                                <span class="pl-3 me-2">Loudness range</span>
                            </label>
                            <label class="label max-w-xs justify-normal p-0">
                                <input
                                    type="number"
                                    v-model="lufs.tp"
                                    min="-9.0"
                                    max="0.0"
                                    step="0.1"
                                    class="input input-xs w-20 rounded-sm"
                                />
                                <span class="pl-3 me-2">True peak</span>
                            </label>
                        </div>
                    </div>
                    <div class="bg-base-200 p-2">
                        Copyright
                        <label class="form-control mt-2 max-w-64 px-0">
                            <input
                                v-model="copyright"
                                class="input input-xs input-bordered rounded-sm w-full"
                                placeholder="Copyright string"
                            />
                        </label>
                    </div>
                    <div class="bg-base-200 p-2 grow">
                        Publish
                        <label class="form-control mt-2 max-w-32 px-0">
                            <select
                                v-model="store.publishPreset"
                                class="select select-xs select-bordered rounded-sm w-full"
                            >
                                <option disabled selected>Preset to publish</option>
                                <option v-for="preset in store.presets" :key="preset.name" :value="preset.name">
                                    {{ preset.title }}
                                </option>
                            </select>
                        </label>
                    </div>
                </div>
                <div class="grow bg-base-200 p-2">
                    <strong>Transcript</strong>
                    <div class="flex gap-2">
                        <div class="w-48">
                            Languages
                            <table
                                class="bg-base-300 border-collapse table table-zebra table-fixed border border-zinc-700 rounded-sm mt-2"
                            >
                                <thead>
                                    <tr class="bg-base-200 border border-zinc-700">
                                        <th class="p-1 border border-zinc-700">Name</th>
                                        <th class="p-1 border border-zinc-700 w-14">Code</th>
                                        <th class="p-0 border border-zinc-700 w-6">
                                            <button
                                                class="btn btn-ghost btn-xs rounded-none p-[5px]"
                                                title="Add language"
                                                @click="addLang()"
                                            >
                                                <i class="bi bi-plus-lg leading-3 text-center" />
                                            </button>
                                        </th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr
                                        v-for="lang in store.transcriptLanguages"
                                        :key="lang.code"
                                        class="p-1 border border-zinc-700"
                                    >
                                        <td class="p-0 border border-zinc-700">
                                            <input
                                                type="text"
                                                v-model="lang.name"
                                                class="input input-ghost input-xs w-full rounded-none"
                                            />
                                        </td>
                                        <td class="p-0 border border-zinc-700">
                                            <input
                                                type="text"
                                                v-model="lang.code"
                                                class="input input-ghost input-xs w-full rounded-none"
                                            />
                                        </td>
                                        <td class="p-0 border border-zinc-700">
                                            <button
                                                class="btn btn-ghost btn-xs rounded-none p-[5px]"
                                                title="Delete language"
                                                @click="deleteLang(lang)"
                                            >
                                                <i class="bi bi-x-lg leading-3 text-center" />
                                            </button>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                        <div>
                            Command line argument
                            <label class="form-control mt-2 max-w-full px-0">
                                <input
                                    type="text"
                                    v-model="transcript_cmd"
                                    class="input input-xs w-full rounded-sm"
                                    placeholder="/usr/local/bin/transcript.py -c int8 -l %lang% -f %file%"
                                />
                            </label>
                            <label class="label">
                                <span class="text-sm select-text text-base-content/80"
                                    >%lang% and %file% are mandatory values, %mount% represents the parent folder of the
                                    file and is required in container environments.</span
                                >
                            </label>
                        </div>
                    </div>
                </div>
            </div>
            <div class="flex justify-end mt-2">
                <div class="join">
                    <button class="btn btn-sm join-item rounded-sm" @click="cancel">Cancel</button>
                    <button class="btn btn-sm join-item rounded-sm" @click="saveConfig">Save</button>
                </div>
            </div>
        </div>
    </div>
</template>
