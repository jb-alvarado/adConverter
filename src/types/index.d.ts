import type { FFmpegProgress, Preset, Task, Template } from './backend'

export {}

declare global {
    type Task = Task
    type Preset = Preset
    type FFmpegProgress = FFmpegProgress
    type Template = Template
    type Config = Config
    type LufsConfig = LufsConfig

    type AlertObj = {
        text: string
        variance: string
        seconds: number
    }

    type TLang = {
        name: string
        code: string
    }
}
