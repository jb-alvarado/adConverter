<script setup lang="ts">
import { ref, watch } from 'vue'
import { useDraggable, useElementSize, useWindowSize, type Position } from '@vueuse/core'

defineProps({
    title: {
        type: String,
        default: '',
    },
    text: {
        type: String,
        default: '',
    },
    modalAction: {
        type: Function,
        default() {
            return ''
        },
    },
    show: {
        type: Boolean,
        default: false,
    },
    hideButtons: {
        type: Boolean,
        default: false,
    },
})

const handler = ref()
const genericModal = ref()
const relativePosition = ref({ x: 0.5, y: 0.5 })

const { width, height } = useWindowSize({ initialWidth: 1800, initialHeight: 600 })
const { width: elWidth, height: elHeight } = useElementSize(genericModal, { width: 600, height: 190 })

const { style, position } = useDraggable(genericModal, {
    initialValue: { x: (width.value - elWidth.value) / 2, y: (height.value - elHeight.value) / 2 },
    onMove: onWindowMove,
    handle: handler
})

function updateRelativePosition() {
    relativePosition.value.x = (position.value.x + elWidth.value / 2) / width.value
    relativePosition.value.y = (position.value.y + elHeight.value / 2) / height.value
}

function onWindowMove(position: Position) {
    if (position.x <= 1) {
        position.x = 1
    } else if (position.x + elWidth.value >= width.value) {
        position.x = width.value - elWidth.value - 1
    }

    if (position.y <= 3) {
        position.y = 3
    } else if (position.y + elHeight.value >= height.value) {
        position.y = height.value - elHeight.value - 3
    }

    updateRelativePosition()
}

watch([width, height], () => {
    position.value.x = relativePosition.value.x * width.value - elWidth.value / 2
    position.value.y = relativePosition.value.y * height.value - elHeight.value / 2

    onWindowMove(position.value)
})
</script>
<template>
    <div
        v-if="show"
        class="z-[10013] fixed inset-0 flex justify-center bg-black/30 overflow-auto py-5"
    >
        <div
            ref="genericModal"
            class="fixed flex flex-col bg-base-100 min-w-[600px] min-h-[190px] w-auto max-w-[90%] rounded-xs shadow-xl my-auto"
            :style="style"
        >
            <div class="inline-block">
                <div class="flex gap-2 bg-base-200 pl-3 pr-[2px] py-[2px] items-center">
                    <div ref="handler" class="font-bold text-lg truncate flex-1 w-0">{{ title }}</div>
                    <button class="btn btn-sm w-8 h-8 rounded-full text-center" @click="modalAction(false)">
                        <i class="bi bi-x-lg leading-3" />
                    </button>
                </div>

                <div class="grow mt-3 px-3">
                    <slot>
                        <div v-html="text" />
                    </slot>
                </div>
            </div>

            <div v-if="!hideButtons" class="flex justify-end my-3 px-3">
                <div class="join">
                    <button
                        class="btn btn-sm rounded-xs bg-base-300 hover:bg-base-300/50 join-item"
                        @click="modalAction(false)"
                    >
                        Cancel
                    </button>
                    <button
                        class="btn btn-sm rounded-xs bg-base-300 hover:bg-base-300/50 join-item"
                        @click="modalAction(true)"
                    >
                        Ok
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>
