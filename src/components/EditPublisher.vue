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
    channelID: 0,
}

enum User {
    NotConfigured = 0,
    NeedsLogin = 1,
    IsLogin = 2,
}

const userPeertube = ref<User>(User.NotConfigured)
const showLogin = ref(false)
const login = ref(cloneDeep(loginDefault))
const doPublish = ref(false)
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
        .then((response: any) => response.json())
        .then(async (data: any) => {
            data.timestamp = new Date().getTime() / 1000
            data.username = login.value.username
            data.url = login.value.url
            data.channel_id = login.value.channelID
            await config.set('publisher', { peertube: data })
            await config.save()
            userPeertube.value = User.IsLogin
        })
        .catch((e: any) => {
            prop.logger.error(e)
        })
}

async function loadPlatforms() {
    let platform: any = await config.get('publisher')
    const timeNow = new Date().getTime() / 1000

    login.value.channelID = platform?.peertube?.channel_id ?? 0
    login.value.username = platform?.peertube?.username ?? ''
    login.value.url = platform?.peertube?.url ?? ''

    if (platform?.peertube) {
        const lastTime = platform.peertube.timestamp ?? 0

        if (lastTime && lastTime + platform.peertube.expires_in - 3600 > timeNow) {
            userPeertube.value = User.IsLogin
        } else if (lastTime && lastTime + platform.peertube.refresh_token_expires_in - 20 > timeNow) {
            await refresh_peertube_token(platform.peertube)

            await invoke('load_config').catch((e) => {
                store.msgAlert('error', e, 5)
                prop.logger.error(e)
            })
        } else {
            userPeertube.value = User.NeedsLogin
        }
    } else {
        userPeertube.value = User.NotConfigured
    }
}

async function saveLogin(save: boolean) {
    if (save) {
        fetch(`${login.value.url}/api/v1/oauth-clients/local`)
            .then((response: any) => response.json())
            .then((data: any) => {
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
                    .then((response: any) => response.json())
                    .then(async (data: any) => {
                        data.timestamp = new Date().getTime() / 1000
                        data.username = login.value.username
                        data.url = login.value.url
                        data.channel_id = login.value.channelID
                        await config.set('publisher', { peertube: data })
                        await config.save()

                        userPeertube.value = User.IsLogin

                        await invoke('load_config').catch((e) => {
                            store.msgAlert('error', e, 5)
                            prop.logger.error(e)
                        })
                    })
                    .catch((e: any) => {
                        prop.logger.error(e)
                    })
            })
            .catch((e: any) => {
                prop.logger.error(e)
            })
    }

    login.value.password = ''

    showLogin.value = false
}

function handlePeertube() {
    if (userPeertube.value !== User.IsLogin) {
        showLogin.value = true
    }
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

function addPublisher($event: any) {
    if (doPublish.value) {
        publish.value.tags = publish.value.tags.trim().replace(/\s*,\s*/g, ',')
        prop.currentTask.publish = cloneDeep(publish.value)
    } else {
        prop.currentTask.publish = null
    }

    prop.savePublisher($event)
}
</script>
<template>
    <GenericModal :show="show" title="Edit Publisher" :modal-action="addPublisher">
        <div class="min-w-[700px]">
            <div>
                <div class="h-8">
                    <button class="btn btn-sm rounded-xs" @click="handlePeertube()">
                        Peertube
                        <div
                            class="badge badge-sm"
                            :class="{
                                'badge-secondary': userPeertube === User.NotConfigured,
                                'badge-success': userPeertube === User.IsLogin,
                                'badge-warning': userPeertube === User.NeedsLogin,
                            }"
                            :title="
                                userPeertube === User.NotConfigured
                                    ? 'Not Configured'
                                    : userPeertube === User.IsLogin
                                    ? 'Logged In'
                                    : 'Needs Login'
                            "
                        ></div>
                    </button>
                </div>
                <div class="flex gap-3">
                    <div class="form-control w-20">
                        <label class="label cursor-pointer mt-[2px] p-0 pt-2">
                            <span class="label-text">Publish</span>
                            <input v-model="doPublish" type="checkbox" class="checkbox checkbox-sm" />
                        </label>
                    </div>
                    <label class="form-control grow mt-2 max-w-full px-0">
                        <input
                            type="text"
                            v-model="publish.name"
                            class="input input-bordered input-xs w-full rounded-xs"
                            placeholder="Video name"
                        />
                    </label>
                </div>
                <label class="join mt-2 w-full">
                    <input
                        v-model="publish.thumbnail"
                        type="text"
                        class="input input-xs input-bordered rounded-xs join-item w-full"
                        placeholder="Thumbnail"
                    />
                    <button
                        class="btn btn-xs border-base-content/30 hover:border-base-content/40 rounded-xs join-item"
                        @click="getThumbnail()"
                    >
                        ...
                    </button>
                </label>
                <label class="form-control mt-2 max-w-full px-0">
                    <textarea
                        v-model="publish.description"
                        class="textarea textarea-bordered textarea-xs rounded-xs h-24"
                        placeholder="Video description"
                    />
                </label>
                <label class="form-control mt-2 max-w-full px-0">
                    <input
                        type="text"
                        v-model="publish.tags"
                        class="input input-bordered input-xs w-full rounded-xs"
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
                class="input input-bordered input-xs w-full rounded-xs"
                placeholder="URL"
            />
        </label>
        <label class="form-control mt-2 max-w-full px-0">
            <input
                type="number"
                v-model="login.channelID"
                name="channel"
                class="input input-bordered input-xs w-full rounded-xs"
                min="0"
                step="1"
            />
        </label>
        <label class="form-control mt-2 max-w-full px-0">
            <input
                type="text"
                v-model="login.username"
                name="username"
                class="input input-bordered input-xs w-full rounded-xs"
                placeholder="Username"
            />
        </label>
        <label class="form-control mt-2 max-w-full px-0">
            <input
                type="password"
                v-model="login.password"
                class="input input-bordered input-xs w-full rounded-xs"
                placeholder="Password"
            />
        </label>
    </GenericModal>
</template>
