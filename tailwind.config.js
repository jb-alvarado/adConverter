/** @type {import('tailwindcss').Config} */
import daisyui from 'daisyui'
import svgToDataUri from 'mini-svg-data-uri'

export default {
    content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
    theme: {
        extend: {
            backgroundImage: (theme) => ({
                'multiselect-caret': `url("${svgToDataUri(
                    `<svg viewBox="0 0 320 512" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M31.3 192h257.3c17.8 0 26.7 21.5 14.1 34.1L174.1 354.8c-7.8 7.8-20.5 7.8-28.3 0L17.2 226.1C4.6 213.5 13.5 192 31.3 192z"></path></svg>`
                )}")`,
                'multiselect-spinner': `url("${svgToDataUri(
                    `<svg viewBox="0 0 512 512" fill="${theme(
                        'colors.green.500'
                    )}" xmlns="http://www.w3.org/2000/svg"><path d="M456.433 371.72l-27.79-16.045c-7.192-4.152-10.052-13.136-6.487-20.636 25.82-54.328 23.566-118.602-6.768-171.03-30.265-52.529-84.802-86.621-144.76-91.424C262.35 71.922 256 64.953 256 56.649V24.56c0-9.31 7.916-16.609 17.204-15.96 81.795 5.717 156.412 51.902 197.611 123.408 41.301 71.385 43.99 159.096 8.042 232.792-4.082 8.369-14.361 11.575-22.424 6.92z"></path></svg>`
                )}")`,
                'multiselect-remove': `url("${svgToDataUri(
                    `<svg viewBox="0 0 320 512" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M207.6 256l107.72-107.72c6.23-6.23 6.23-16.34 0-22.58l-25.03-25.03c-6.23-6.23-16.34-6.23-22.58 0L160 208.4 52.28 100.68c-6.23-6.23-16.34-6.23-22.58 0L4.68 125.7c-6.23 6.23-6.23 16.34 0 22.58L112.4 256 4.68 363.72c-6.23 6.23-6.23 16.34 0 22.58l25.03 25.03c6.23 6.23 16.34 6.23 22.58 0L160 303.6l107.72 107.72c6.23 6.23 16.34 6.23 22.58 0l25.03-25.03c6.23-6.23 6.23-16.34 0-22.58L207.6 256z"></path></svg>`
                )}")`,
            }),
            maskImage: (theme) => ({
                'multiselect-caret': `url("${svgToDataUri(
                    `<svg viewBox="0 0 320 512" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M31.3 192h257.3c17.8 0 26.7 21.5 14.1 34.1L174.1 354.8c-7.8 7.8-20.5 7.8-28.3 0L17.2 226.1C4.6 213.5 13.5 192 31.3 192z"></path></svg>`
                )}")`,
                'multiselect-spinner': `url("${svgToDataUri(
                    `<svg viewBox="0 0 512 512" fill="${theme(
                        'colors.green.500'
                    )}" xmlns="http://www.w3.org/2000/svg"><path d="M456.433 371.72l-27.79-16.045c-7.192-4.152-10.052-13.136-6.487-20.636 25.82-54.328 23.566-118.602-6.768-171.03-30.265-52.529-84.802-86.621-144.76-91.424C262.35 71.922 256 64.953 256 56.649V24.56c0-9.31 7.916-16.609 17.204-15.96 81.795 5.717 156.412 51.902 197.611 123.408 41.301 71.385 43.99 159.096 8.042 232.792-4.082 8.369-14.361 11.575-22.424 6.92z"></path></svg>`
                )}")`,
                'multiselect-clear-icon': `url("${svgToDataUri(
                    `<svg viewBox="0 0 320 512" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M207.6 256l107.72-107.72c6.23-6.23 6.23-16.34 0-22.58l-25.03-25.03c-6.23-6.23-16.34-6.23-22.58 0L160 208.4 52.28 100.68c-6.23-6.23-16.34-6.23-22.58 0L4.68 125.7c-6.23 6.23-6.23 16.34 0 22.58L112.4 256 4.68 363.72c-6.23 6.23-6.23 16.34 0 22.58l25.03 25.03c6.23 6.23 16.34 6.23 22.58 0L160 303.6l107.72 107.72c6.23 6.23 16.34 6.23 22.58 0l25.03-25.03c6.23-6.23 6.23-16.34 0-22.58L207.6 256z"></path></svg>`
                )}")`,
                'multiselect-tag-remove-icon': `url("${svgToDataUri(
                    `<svg viewBox="0 0 320 512" fill="currentColor" xmlns="http://www.w3.org/2000/svg"><path d="M207.6 256l107.72-107.72c6.23-6.23 6.23-16.34 0-22.58l-25.03-25.03c-6.23-6.23-16.34-6.23-22.58 0L160 208.4 52.28 100.68c-6.23-6.23-16.34-6.23-22.58 0L4.68 125.7c-6.23 6.23-6.23 16.34 0 22.58L112.4 256 4.68 363.72c-6.23 6.23-6.23 16.34 0 22.58l25.03 25.03c6.23 6.23 16.34 6.23 22.58 0L160 303.6l107.72 107.72c6.23 6.23 16.34 6.23 22.58 0l25.03-25.03c6.23-6.23 6.23-16.34 0-22.58L207.6 256z"></path></svg>`
                )}")`,
            }),
            boxShadow: {
                'all': '0 0 15px rgba(0, 0, 0, 0.3)',
                'b': '0 10px 10px -10px rgba(0, 0, 0, 0.3)',
            },
            fontFamily: {
                sans: ['Source Sans Pro', 'Segoe UI', 'Helvetica Neue', 'Arial', 'sans-serif'],
                mono: ['Roboto Mono']
            },
        },
    },
    safelist: [
        'text-lime-600',
        'text-cyan-600',
        'text-red-600',
        'text-yellow-600',
        'text-base-content/60',
        'alert-success',
        'alert-warning',
        'alert-info',
        'alert-error',
    ],
    daisyui: {
        themes: [
            {
                light: {
                    'color-scheme': 'light',
                    primary: '#e0e0e0',
                    'base-content': '#222222',
                    secondary: '#c7c7c7',
                    accent: '#f28c1b',
                    'base-100': '#ffffff',
                    'base-200': '#F2F5F7',
                    'base-300': '#E5E6E6',
                    neutral: '#2B3440',
                    'neutral-focus': '#343232',
                    info: '#0000ff',
                    success: '#008000',
                    warning: '#f28c1b',
                    error: '#ff3c00',
                    '--base-100': '#ffffff',
                    '--base-200': '#F2F5F7',
                    '--base-300': '#E5E6E6',
                    '--my-accent': '#f28c1b',
                    '--my-gray': '#888888',
                    '--code-punctuation': '#383a42',
                    '--code-attr': '#e45649',
                    '--code-string': '#50a14f',
                    '--code-number': '#986801',
                    '--code-keyword': '#0184bc',
                },
                dark: {
                    'color-scheme': 'dark',
                    primary: '#3b3b3b',
                    'base-content': '#DFDFDF',
                    secondary: '#d3d3d3',
                    accent: '#f28c1b',
                    'base-100': '#313131',
                    'base-200': '#222222',
                    'base-300': '#1c1c1c',
                    neutral: '#272626',
                    'neutral-focus': '#343232',
                    info: '#0000ff',
                    success: '#008000',
                    warning: '#f28c1b',
                    error: '#ff3c00',
                    '--base-100': '#313131',
                    '--base-200': '#222222',
                    '--base-300': '#1c1c1c',
                    '--code-punctuation': '#abb2bf',
                    '--code-attr': '#e06c75',
                    '--code-string': '#98c379',
                    '--code-number': '#d19a66',
                    '--code-keyword': '#4fb6d5',
                },
            },
        ],
    },
    plugins: [daisyui],
}
