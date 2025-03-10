<template>
    <div
        id="timeField"
        class="input input-bordered rounded-xs flex pl-[2px] pr-0 py-0 focus-within:border-base-content/30 focus-within:outline-base-content/30"
        :class="`input-${size}`"
    >
        <div class="grow flex items-center">
            <input
                ref="timeInput"
                :value="secToTime(props.modelValue)"
                type="text"
                pattern="([01]?[0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]((\.|:)[0-9]{1,3})?"
                class="w-full px-1 py-0 h-[20px] text-[0.8455rem]"
                @click="setCursorPos"
                @change="$emit('update:modelValue', timeToSec($event))"
                :disabled="disabled"
            />
        </div>

        <div v-if="props.isNumber" class="w-auto">
            <div class="flex flex-col text-xs h-[30px]">
                <button
                    class="bg-base-300 hover:bg-base-300/50 px-1 leading-0 text-[9px] h-[15px] rounded-t-sm"
                    tabindex="0"
                    @click="countUp"
                >
                    <i class="bi-chevron-up" />
                </button>
                <button
                    class="bg-base-300 hover:bg-base-300/50 px-1 leading-3 pb-[2px] text-[9px] h-[15px] rounded-b-sm"
                    @click="countDown"
                >
                    <i class="bi-chevron-down" />
                </button>
            </div>
        </div>
    </div>
</template>
<script setup lang="ts">
import { ref, nextTick } from 'vue'

const emit = defineEmits(['update:modelValue'])

const props = defineProps({
    modelValue: {
        type: [Number, String],
        required: true,
    },
    disabled: {
        type: Boolean,
        default: false,
    },
    size: {
        type: String,
        default: 'sm',
    },
    isNumber: {
        type: Boolean,
        default: true,
    }
})

const timeInput = ref()
const cursorPos = ref(8)

function secToTime(sec: number | string) {
    if (typeof sec === 'string') {
        return sec
    }

    const hours = Math.floor(sec / 3600)
    sec %= 3600
    const minutes = Math.floor(sec / 60)
    const seconds = Math.floor(sec % 60)
    const ms = Math.round((sec - Math.floor(sec)) * 1000) / 1000
    const secFmt = (seconds + ms).toFixed(3)

    const m = String(minutes).padStart(2, '0')
    const h = String(hours).padStart(2, '0')
    const s = secFmt.padStart(6, '0')

    return `${h}:${m}:${s}`
}

function timeToSec(event: any) {
    if (!props.isNumber) {
        return event.target?.value
    }

    const time = event.target?.value ?? 0

    const [h, m, s] = time.split(':').map((val: string) => Number(val) || 0)

    return h * 3600 + m * 60 + s
}

function setCursorPos() {
    cursorPos.value = timeInput.value?.selectionStart
}

function countUp() {
    if (typeof props.modelValue === 'string') {
        return
    }

    let count = 0

    if (cursorPos.value && cursorPos.value >= 6) {
        count = 1
    } else if (cursorPos.value && cursorPos.value >= 3) {
        count = 60
    } else {
        count = 3600
    }

    emit('update:modelValue', props.modelValue + count)

    nextTick(() => {
        timeInput.value?.focus()
        timeInput.value?.setSelectionRange(cursorPos.value, cursorPos.value)
    })
}

function countDown() {
    if (typeof props.modelValue === 'string') {
        return
    }

    let sec = props.modelValue
    let count = 0

    if (cursorPos.value && cursorPos.value >= 6) {
        count = 1
    } else if (cursorPos.value && cursorPos.value >= 3) {
        count = 60
    } else {
        count = 3600
    }

    sec -= count

    if (sec < 0) {
        emit('update:modelValue', 0)
    } else {
        emit('update:modelValue', sec)
    }

    nextTick(() => {
        timeInput.value?.focus()
        timeInput.value?.setSelectionRange(cursorPos.value, cursorPos.value)
    })
}
</script>
<style scoped>
#timeField:has(> div > input:invalid) {
    border: red solid 1px;
}
</style>
