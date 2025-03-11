<script setup lang="ts">
import { nextTick, watch, onBeforeMount } from 'vue'
import { useVirtualList } from '@vueuse/core'
import { storeToRefs } from 'pinia'

import { useStore } from '../store/index.ts'

const store = useStore()
const { logContent } = storeToRefs(useStore())

const { list, containerProps, wrapperProps, scrollTo } = useVirtualList(store.logContent, {
    itemHeight: 22,
})

onBeforeMount(() => {
    handleScrollTo()
})

function handleScrollTo() {
    nextTick(() => {
        scrollTo(store.logContent.length - 1)
    })
}

watch([logContent.value], () => {
    nextTick(() => {
        handleScrollTo()
    })
})
</script>
<template>
    <div>
        <div v-if="store.openLog" class="fixed top-6 z-50 w-full h-[calc(100%-128px)] bg-base-300 overflow-hidden">
            <div class="h-full">
                <div class="w-full sticky top-0 flex justify-center border-b border-base-content/30">
                    <button
                        class="h-4 leading-3 bg-base-200 cursor-pointer hover:bg-base-300 rounded-none min-w-full active:scale-100!"
                        @click="store.openLog = false"
                    >
                        <i class="bi-chevron-compact-down" />
                    </button>
                </div>

                <div v-bind="containerProps" class="h-[calc(100%-20px)]">
                    <ul
                        id="logContainer"
                        v-bind="wrapperProps"
                        class="h-full p-1 select-text text-base-content/90 font-['Roboto_Mono'] text-sm font-[300] whitespace-pre cursor-text"
                    >
                        <li v-for="line in list" :key="line.index" v-html="line.data" style="height: 22px" />
                    </ul>
                </div>
            </div>
        </div>
    </div>
</template>
