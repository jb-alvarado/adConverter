<script setup lang="ts">
import { ref } from 'vue'
import { LazyStore } from '@tauri-apps/plugin-store'
import { cloneDeep } from 'lodash-es';

import GenericModal from '../components/GenericModal.vue'

const config = new LazyStore('config.json', { autoSave: false })

const loginDefault = {
    url: '',
    username: '',
    password: '',
}

const showLogin = ref(false)
const login = ref(cloneDeep(loginDefault))
const selected = ref({ platform: '', name: '', description: '', tags: '' })
const settings = ref([
    { platform: 'Rumble', name: '', description: '', tags: '' },
    { platform: 'Peertube', name: '', description: '', tags: '' },
    { platform: 'Youtube', name: '', description: '', tags: '' },
])

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
    savePublisher: {
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

async function switchPlatform() {
    let platform = await config.get(selected.value.name.toLowerCase())

    if (platform) {
        console.log(platform)
    } else {
        showLogin.value = true
    }
}

async function saveLogin(save: boolean) {
    if (save) {
        // TODO: don't save password, work with token/refresh token
    }

    showLogin.value = false
}
</script>
<template>
    <GenericModal :show="show" title="Edit Publisher" :modal-action="savePublisher">
        <div class="min-w-[700px]">
            <select
                v-model="selected"
                class="select select-sm select-bordered w-full rounded-sm"
                @change="switchPlatform"
            >
                <option disabled selected>Platform</option>
                <option v-for="set in settings" :key="set.platform" :value="set">{{ set.platform }}</option>
            </select>
            <div>
                <label class="form-control mt-2 max-w-full px-0">
                    <input
                        type="text"
                        v-model="selected.name"
                        class="input input-bordered input-xs w-full rounded-sm"
                        placeholder="Video name"
                    />
                </label>
                <label class="form-control mt-2 max-w-full px-0">
                    <textarea
                        v-model="selected.description"
                        class="textarea textarea-bordered textarea-xs rounded-sm h-24"
                        placeholder="Video description"
                    />
                </label>
                <label class="form-control mt-2 max-w-full px-0">
                    <input
                        type="text"
                        v-model="selected.tags"
                        class="input input-bordered input-xs w-full rounded-sm"
                        placeholder="Video Tags (comma separated list)"
                    />
                </label>
            </div>
        </div>
    </GenericModal>
    <GenericModal :show="showLogin" :title="`Login to ${selected.platform}`" :modal-action="saveLogin">
        <label class="form-control mt-2 max-w-full px-0">
            <input
                type="text"
                v-model="login.url"
                class="input input-bordered input-xs w-full rounded-sm"
                placeholder="URL"
            />
        </label>
        <label class="form-control mt-2 max-w-full px-0">
            <input
                type="text"
                v-model="login.username"
                class="input input-bordered input-xs w-full rounded-sm"
                placeholder="Username"
            />
        </label>
        <label class="form-control mt-2 max-w-full px-0">
            <input
                type="password"
                v-model="login.password"
                class="input input-bordered input-xs w-full rounded-sm"
                placeholder="Password"
            />
        </label>
    </GenericModal>
</template>
