// import { ref, type Ref } from 'vue'
import { useStore } from '../store/index.ts'

export const stringFormatter = () => {
    function formatBytes(bytes: number, decimals = 2) {
        if (!+bytes) return '0 Bytes'

        const k = 1024
        const dm = decimals < 0 ? 0 : decimals
        const sizes = ['Bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB']

        const i = Math.floor(Math.log(bytes) / Math.log(k))

        return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`
    }

    function folderPath(path: string): string {
        const match = path.match(/^(.*[\\/])/)
        return match ? match[1] : ''
    }

    function filename(path: string): string {
        const match = path.match(/[^\\/]+$/)
        return match ? match[0] : ''
    }

    function extension(path: string): string {
        const match = path.match(/[^\.]+$/)
        return match ? match[0].toLocaleLowerCase() : ''
    }

    function removeExtension(path: string): string {
        return path.replace(/\.[^/.]+$/, '')
    }

    function secToMin(seconds: number): string {
        const minutes = Math.floor(seconds / 60)
        const remainingSeconds = (seconds % 60).toFixed(0)
        return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')} min`
    }

    function format(template: string, ...args: any[]): string {
        return template.replace(/{(\d+)}/g, (match, index) => {
            return typeof args[index] !== 'undefined' ? args[index] : match
        })
    }

    function logTime(): string {
        const now = new Date()

        const hours = String(now.getHours()).padStart(2, '0')
        const minutes = String(now.getMinutes()).padStart(2, '0')
        const seconds = String(now.getSeconds()).padStart(2, '0')
        const microseconds = String(now.getMilliseconds() * 1000).padStart(6, '0')

        return `${hours}:${minutes}:${seconds}.${microseconds}`
    }

    class Logger {
        constructor() {
            this.store = useStore()
        }

        store = {} as any

        fmt(text: string | object) {
            return typeof text === "object" ? JSON.stringify(text) : text
        }

        debug(msg: string | object) {
            let line = `<span class="text-cyan-600">${logTime()}</span> <span class="text-lime-600">[DEBUG]</span> ${this.fmt(msg)}`
            this.store.logContent.push(line)
        }

        error(msg: string | object) {
            let line = `<span class="text-gray-600">${logTime()}</span> <span class="text-red-600">[ERROR]</span> ${this.fmt(msg)}`
            this.store.logContent.push(line)
        }

        info(msg: string | object) {
            let line = `<span class="text-gray-600">${logTime()}</span> <span class="text-lime-600">[ INFO]</span> ${this.fmt(msg)}`
            this.store.logContent.push(line)
        }

        warn(msg: string | object) {
            let line = `<span class="text-yellow-600">${logTime()}</span> <span class="text-lime-600">[ WARN]</span> ${this.fmt(msg)}`
            this.store.logContent.push(line)
        }
    }

    return { formatBytes, folderPath, filename, format, extension, removeExtension, secToMin, logTime, Logger }
}

export const useVariables = () => {
    const multiSelectClasses = {
        container: 'relative input input-bordered w-full flex items-center justify-end px-0 min-h-[32px]',
        containerDisabled: 'cursor-default bg-base',
        containerOpen: 'rounded-b-none',
        containerOpenTop: 'rounded-t-none',
        containerActive: 'ring-3 ring-base-100 ring-opacity-30',
        wrapper: 'relative mx-auto w-full flex items-center justify-end box-border cursor-pointer outline-hidden',
        singleLabel:
            'flex items-center h-full max-w-full absolute left-0 top-0 pointer-events-none bg-transparent leading-snug pl-3.5 pr-16 box-border rtl:left-auto rtl:right-0 rtl:pl-0 rtl:pr-3.5',
        singleLabelText: 'text-ellipsis overflow-hidden block whitespace-nowrap max-w-full',
        multipleLabel:
            'flex items-center h-full absolute left-0 top-0 pointer-events-none bg-transparent leading-snug pl-3.5 rtl:left-auto rtl:right-0 rtl:pl-0 rtl:pr-3.5',
        search: 'w-full absolute inset-0 outline-hidden focus:ring-0 appearance-none box-border border-0 text-base font-sans bg-base-100 rounded-sm pl-3.5 rtl:pl-0 rtl:pr-3.5',
        tags: 'grow shrink flex items-center mt-0 pl-2 min-w-0 rtl:pl-0 rtl:pr-2',
        tag: 'bg-base-300 h-6 min-w-10 text-base-content text-sm font-bold py-0.5 pl-2 rounded-sm mr-1 my-0.5 flex items-center whitespace-nowrap min-w-0 rtl:pl-0 rtl:pr-2 rtl:mr-0 rtl:ml-1',
        tagDisabled: 'pr-2 opacity-50 rtl:pl-2',
        tagWrapper: 'whitespace-nowrap overflow-hidden text-ellipsis',
        tagWrapperBreak: 'whitespace-normal break-all',
        tagRemove: 'flex items-center justify-center p-1 mx-0.5 rounded-xs hover:bg-black hover:bg-opacity-10 group',
        tagRemoveIcon:
            'bg-multiselect-remove bg-center bg-no-repeat opacity-30 inline-block w-3 h-3 group-hover:opacity-60',
        tagsSearchWrapper: 'inline-block relative mx-1 grow shrink h-[24px]',
        tagsSearch:
            'absolute inset-0 border-0 outline-hidden focus:ring-0 appearance-none p-0 text-base font-sans box-border w-full h-full',
        tagsSearchCopy: 'invisible whitespace-pre-wrap inline-block h-px',
        placeholder:
            'flex items-center h-full absolute left-0 top-0 pointer-events-none bg-transparent leading-snug pl-3.5 text-gray-400 rtl:left-auto rtl:right-0 rtl:pl-0 rtl:pr-3.5',
        caret: 'multiselect-caret bg-center bg-no-repeat w-2.5 h-4 py-px box-content mr-3.5 relative z-10 shrink-0 grow-0 transition-transform transform pointer-events-none rtl:mr-0 rtl:ml-3.5',
        caretOpen: 'rotate-180 pointer-events-auto',
        clear: 'pr-3.5 relative z-10 transition duration-300 shrink-0 grow-0 flex hover:opacity-80 rtl:pr-0 rtl:pl-3.5',
        clearIcon: 'multiselect-clear-icon bg-center bg-no-repeat w-2.5 h-4 py-px box-content inline-block',
        spinner:
            'bg-multiselect-spinner bg-center bg-no-repeat w-4 h-4 z-10 mr-3.5 animate-spin shrink-0 grow-0 rtl:mr-0 rtl:ml-3.5',
        infinite: 'flex items-center justify-center w-full',
        infiniteSpinner:
            'bg-multiselect-spinner bg-center bg-no-repeat w-4 h-4 z-10 animate-spin shrink-0 grow-0 m-3.5',
        dropdown:
            'max-h-60 absolute -left-px -right-px bottom-0 transform translate-y-full border border-gray-300 -mt-px overflow-y-scroll z-50 bg-base-100 flex flex-col rounded-b-sm',
        dropdownTop: '-translate-y-full top-px bottom-auto rounded-b-none rounded-t-sm',
        dropdownHidden: 'hidden',
        options: 'flex flex-col p-0 m-0 list-none',
        optionsTop: '',
        group: 'p-0 m-0',
        groupLabel:
            'flex text-sm box-border items-center justify-start text-left py-1 px-3 font-bold bg-gray-200 cursor-default leading-normal',
        groupLabelPointable: 'cursor-pointer',
        groupLabelPointed: 'bg-gray-300 text-gray-700',
        groupLabelSelected: 'bg-accent text-my-text',
        groupLabelDisabled: 'bg-gray-100 text-gray-300 cursor-not-allowed',
        groupLabelSelectedPointed: 'bg-accent text-my-text opacity-90',
        groupLabelSelectedDisabled: 'text-green-100 bg-green-600 bg-opacity-50 cursor-not-allowed',
        groupOptions: 'p-0 m-0',
        option: 'flex items-center justify-start box-border text-left cursor-pointer text-base leading-snug py-2 px-3',
        optionPointed: 'text-gray-800 bg-secondary',
        optionSelected: 'text-my-text bg-accent',
        optionDisabled: 'text-gray-300 cursor-not-allowed',
        optionSelectedPointed: 'text-my-text bg-link-hover',
        optionSelectedDisabled: 'text-green-100 bg-green-500 bg-opacity-50 cursor-not-allowed',
        noOptions: 'py-2 px-3 text-gray-600 bg-base-100 text-left rtl:text-right',
        noResults: 'py-2 px-3 text-gray-600 bg-base-100 text-left rtl:text-right',
        fakeInput:
            'bg-transparent absolute left-0 right-0 -bottom-px w-full h-px border-0 p-0 appearance-none outline-hidden text-transparent',
        assist: 'absolute -m-px w-px h-px overflow-hidden',
        spacer: 'h-6 py-px box-content',
    }

    function alertIcon(variance: string) {
        switch (variance) {
            case 'error':
                return `<svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="stroke-current shrink-0 h-6 w-6"
                    fill="none"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                </svg>`
            case 'success':
                return ` <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="stroke-current shrink-0 h-6 w-6"
                    fill="none"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                </svg>`
            case 'warning':
                return `<svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="stroke-current shrink-0 h-6 w-6"
                    fill="none"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                    />
                </svg>`
            default:
                return ` <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    class="stroke-current shrink-0 w-6 h-6"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                </svg>`
        }
    }

    return {
        multiSelectClasses,
        alertIcon,
    }
}
