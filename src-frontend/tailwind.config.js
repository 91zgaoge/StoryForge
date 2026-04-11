/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        cinema: {
          950: '#050508',
          900: '#0a0a0f',
          850: '#0f0f16',
          800: '#151520',
          700: '#1e1e2e',
          600: '#2a2a3c',
          500: '#3a3a50',
          gold: '#d4af37',
          'gold-light': '#e8c547',
          'gold-dark': '#b8941f',
          velvet: '#4c1d95',
          amber: '#f59e0b',
          rust: '#c2410c',
        },
      },
      fontFamily: {
        display: ['Cinzel', 'serif'],
        body: ['Crimson Pro', 'Georgia', 'serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      animation: {
        'fade-up': 'fadeUp 0.6s ease-out',
        'fade-in': 'fadeIn 0.4s ease-out',
        'slide-left': 'slideLeft 0.5s ease-out',
        'pulse-slow': 'pulse 3s ease-in-out infinite',
        'spin-slow': 'spin 3s linear infinite',
      },
      keyframes: {
        fadeUp: {
          '0%': { opacity: '0', transform: 'translateY(20px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideLeft: {
          '0%': { opacity: '0', transform: 'translateX(20px)' },
          '100%': { opacity: '1', transform: 'translateX(0)' },
        },
      },
      backdropBlur: {
        cinema: '12px',
      },
    },
  },
  plugins: [],
}
