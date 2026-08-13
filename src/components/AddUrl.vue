<script setup lang="ts">
import { ref, watch } from 'vue'

import GenericModal from './GenericModal.vue'

const props = defineProps({
    show: { type: Boolean, default: false },
    initialUrl: { type: String, default: '' },
    addUrl: { type: Function, required: true },
})

const url = ref('')

watch(
    () => [props.show, props.initialUrl],
    () => {
        if (props.show) url.value = props.initialUrl
    }
)

function submit(save: boolean) {
    if (save && url.value.trim()) props.addUrl(url.value.trim())
    else props.addUrl('')
}
</script>

<template>
    <GenericModal :show="show" title="Add URL" :modal-action="submit">
        <label class="form-control w-full">
            <span class="label-text mb-2">Video or audio URL</span>
            <input
                v-model="url"
                type="url"
                class="input input-bordered input-sm rounded-xs w-full"
                placeholder="https://…"
                @keyup.enter="submit(true)"
            />
        </label>
    </GenericModal>
</template>
