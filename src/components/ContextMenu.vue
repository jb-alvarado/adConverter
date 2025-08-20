<script setup>
import { ref } from 'vue'

const { actions, x, y } = defineProps(['actions', 'x', 'y'])
const emit = defineEmits(['action-clicked'])

const emitAction = (action) => {
    emit('action-clicked', action)
}
</script>

<template>
    <div class="fixed z-50 context-menu bg-base-100 p-1 flex flex-col rounded-xs border border-base-content/30" :style="{ top: y + 'px', left: x + 'px' }">
        <button v-for="action in actions" :key="action.action" class="btn btn-ghost justify-start py-1" @click="emitAction(action.action)">
            {{ action.label }}
        </button>
    </div>
</template>
