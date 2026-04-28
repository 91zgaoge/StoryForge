/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        cinema: {
          50: '#f9f8f6',
          100: '#f0ede8',
          200: '#e0d9cf',
          300: '#cbc0b0',
          400: '#b5a590',
          500: '#a08f7a',
          600: '#8a7966',
          700: '#736354',
          800: '#5e5044',
          900: '#1a1612',
          950: '#0d0b09',
          gold: '#c9a96e',
          'gold-light': '#d9bc8e',
          'gold-dark': '#a88a4e',
        },
      },
      fontFamily: {
        display: ['"LXGW WenKai"', '"Noto Serif SC"', 'serif'],
        sans: ['"Inter"', '"Noto Sans SC"', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
