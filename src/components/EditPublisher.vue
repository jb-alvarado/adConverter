<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { LazyStore } from '@tauri-apps/plugin-store'
import { open } from '@tauri-apps/plugin-dialog'
import { fetch } from '@tauri-apps/plugin-http'
import { cloneDeep } from 'lodash-es'

import { stringFormatter } from '../composables/helper'
import { useStore } from '../store/index.ts'

import GenericModal from '../components/GenericModal.vue'

const { folderPath } = stringFormatter()
const store = useStore()

const config = new LazyStore('config.json', { autoSave: false })

const loginDefault = {
    url: '',
    username: '',
    password: '',
}

const showLogin = ref(false)
const login = ref(cloneDeep(loginDefault))
const publish = ref({ name: '', thumbnail: '', description: '', tags: '' })

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

onMounted(async () => {
    await loadPlatforms()
})

async function refresh_peertube_token(data: any) {
    let payload = {
        client_id: data.client_id,
        client_secret: data.client_secret,
        grant_type: 'refresh_token',
        refresh_token: data.refresh_token,
    }

    fetch(`${login.value.url}/api/v1/users/token`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams(payload),
    })
        .then((response) => response.json())
        .then(async (data) => {
            await config.set('publisher', { peertube: data })
            await config.save()
        })
        .catch((e) => {
            prop.logger.error(e)
        })
}

async function loadPlatforms() {
    let platform: any = await config.get('publisher')

    if (platform?.peertube) {
        if (platform.peertube.expires_in < 7200 && platform.peertube.refresh_token_expires_in > 0) {
            await refresh_peertube_token(platform.peertube)

            await invoke('load_config').catch((e) => {
                store.msgAlert('error', e, 5)
                prop.logger.error(e)
            })
        }
    } else {
        showLogin.value = true
    }
}

async function saveLogin(save: boolean) {
    if (save) {
        fetch(`${login.value.url}/api/v1/oauth-clients/local`)
            .then((response) => response.json())
            .then((data) => {
                let payload = {
                    client_id: data.client_id,
                    client_secret: data.client_secret,
                    grant_type: 'password',
                    response_type: 'code',
                    username: login.value.username,
                    password: login.value.password,
                }

                fetch(`${login.value.url}/api/v1/users/token`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                    body: new URLSearchParams(payload),
                })
                    .then((response) => response.json())
                    .then(async (data) => {
                        await config.set('publisher', { peertube: data })
                        await config.save()

                        await invoke('load_config').catch((e) => {
                            store.msgAlert('error', e, 5)
                            prop.logger.error(e)
                        })
                    })
                    .catch((e) => {
                        prop.logger.error(e)
                    })
            })
            .catch((e) => {
                prop.logger.error(e)
            })
    }

    showLogin.value = false
}

async function getThumbnail() {
    const path = prop.currentTask.path
    const folder = folderPath(path)
    let options = {
        multiple: false,
        directory: false,
        filters: [
            {
                name: 'Image',
                extensions: store.IMAGE_EXTENSIONS,
            },
        ],
    } as any

    if (path) {
        options.defaultPath = folder
    }

    let thumbnail = (await open(options)) as string | null

    publish.value.thumbnail = thumbnail ?? ''
}
</script>
<template>
    <GenericModal :show="show" title="Edit Publisher" :modal-action="savePublisher">
        <div class="min-w-[700px]">
            <div>
                <label class="form-control mt-2 max-w-full px-0">
                    <input
                        type="text"
                        v-model="publish.name"
                        class="input input-bordered input-xs w-full rounded-sm"
                        placeholder="Video name"
                    />
                </label>
                <label class="join mt-2 w-full">
                    <input
                        v-model="publish.thumbnail"
                        type="text"
                        class="input input-xs input-bordered rounded-sm join-item w-full"
                        placeholder="Thumbnail"
                    />
                    <button
                        class="btn btn-xs border-[oklch(var(--bc)/0.2)] hover:border-[oklch(var(--bc)/0.15)] rounded-sm join-item"
                        @click="getThumbnail()"
                    >
                        ...
                    </button>
                </label>
                <label class="form-control mt-2 max-w-full px-0">
                    <textarea
                        v-model="publish.description"
                        class="textarea textarea-bordered textarea-xs rounded-sm h-24"
                        placeholder="Video description"
                    />
                </label>
                <label class="form-control mt-2 max-w-full px-0">
                    <input
                        type="text"
                        v-model="publish.tags"
                        class="input input-bordered input-xs w-full rounded-sm"
                        placeholder="Video Tags (comma separated list)"
                    />
                </label>
            </div>
        </div>
    </GenericModal>
    <GenericModal :show="showLogin" title="Login to Peertube" :modal-action="saveLogin">
        <label class="form-control mt-2 max-w-full px-0">
            <input
                type="text"
                v-model="login.url"
                name="url"
                class="input input-bordered input-xs w-full rounded-sm"
                placeholder="URL"
            />
        </label>
        <label class="form-control mt-2 max-w-full px-0">
            <input
                type="text"
                v-model="login.username"
                name="username"
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
