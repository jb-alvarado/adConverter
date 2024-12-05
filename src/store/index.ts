import { defineStore } from 'pinia'
import { some } from 'lodash-es'

export const useStore = defineStore('index', {
    state: () => ({
        ALLOWED_EXTENSIONS: ['avi', 'mpv', 'mp4', 'm4v', 'm4a', 'mov', 'webm', 'mkv', 'wav', 'aif', 'mp3', 'aac'],
        IMAGE_EXTENSIONS: ['exr', 'png', 'tga', 'tif', 'tiff', 'gif'],
        VIDEO_EXTENSIONS: [
            'avi',
            'mov',
            'webm',
            'mp4',
            'mpv',
            'm4v',
            'h264',
            'mkv',
            'vob',
            'wmv',
            'yuv',
            'm2v',
            'mpg',
            'mpeg',
            'mxf',
        ],
        alertList: [] as AlertObj[],
        modalShow: false,
        modalVariant: 'info' as 'info' | 'warning' | 'error',
        modalMessage: '',
        jobsDone: false,
        taskList: [] as Task[],
        defaultTask: {
            path: '',
            in: 0,
            out: 0,
            lufs: false,
            fade: false,
            transcript: 'none',
            presets: [],
            target: null,
            active: false,
            finished: false,
        } as Task,
        presets: [] as Preset[],
        publishPreset: null  as string | null,
        currentTemplate: {
            intro: '',
            intro_duration: 0.0,
            outro: '',
            outro_duration: 0.0,
            lower_thirds: [],
        } as Template,
        defaultTemplate: {
            intro: '',
            intro_duration: 0.0,
            outro: '',
            outro_duration: 0.0,
            lower_thirds: [],
        } as Template,
        logContent: [] as string[],
        openLog: false,
        showConfig: false,
        showPresets: false,
        showTranscript: false,
        transcriptLanguages: [] as TLang[],
        progressCurrent: 0,
        progressAll: 0,
        processMsg: '',
        processPath: '',
    }),

    actions: {
        msgModal(variance: 'info' | 'warning' | 'error', text: string, seconds: number = 3) {
            this.modalVariant = variance
            this.modalMessage = text
            this.modalShow = true

            setTimeout(() => {
                this.modalVariant = variance
                this.modalMessage = text
                this.modalShow = false
            }, seconds * 1000)
        },

        msgAlert(variance: string, text: string | object, seconds: number = 3) {
            const textStr = typeof text === "object" ? JSON.stringify(text) : text
            const msg = { text: textStr, variance, seconds }

            if (!some(this.alertList, msg)) {
                this.alertList.push(msg)
            }

            setTimeout(() => {
                const index = this.alertList.indexOf(msg)
                if (index >= 0) {
                    this.alertList.splice(index, 1)
                }
            }, seconds * 1000)
        },
    },
})
