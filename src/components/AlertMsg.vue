<template>
    <div>
        <div
            ref="progressContainer"
            class="toast toast-end fixed top-8 z-50 !h-auto max-h-[calc(100%-134px)] overflow-y-auto gap-1 p-1"
            :style="`height: ${store.alertList.length * 80}px`"
        >
            <div
                v-for="alert in store.alertList"
                :key="alert.text"
                class="alert w-auto max-w-[800px] justify-start py-1 rounded-sm"
                :class="`alert-${alert.variance}`"
            >
                <div v-html="alertIcon(alert.variance)" />
                <span class="truncate w-full" v-html="alert.text" />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { storeToRefs } from 'pinia'

import { useStore } from '../store/index.ts'
import { useVariables } from '../composables/helper'

const store = useStore()
const { alertList } = storeToRefs(useStore())
const { alertIcon } = useVariables()

const progressContainer = ref()

watch([alertList.value], () => {
    nextTick(() => {
        if (progressContainer.value) {
            progressContainer.value.scrollTop = progressContainer.value.scrollHeight + 50
        }
    })
})
</script>
