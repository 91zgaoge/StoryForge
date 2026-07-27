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
        paper: {
          50: 'var(--paper-50)',
          100: 'var(--paper-100)',
          200: 'var(--paper-200)',
          300: 'var(--paper-300)',
        },
        ink: {
          500: 'var(--ink-500)',
          700: 'var(--ink-700)',
          900: 'var(--ink-900)',
        },
        terracotta: {
          DEFAULT: 'var(--terracotta)',
          light: 'var(--terracotta-light)',
          dark: 'var(--terracotta-dark)',
        },
        cinema: {
          950: 'var(--cinema-950)',
          900: 'var(--cinema-900)',
          850: 'var(--cinema-850)',
          800: 'var(--cinema-800)',
          700: 'var(--cinema-700)',
          600: 'var(--cinema-600)',
          gold: 'var(--cinema-gold)',
          'gold-light': 'var(--cinema-gold-light)',
          'gold-dark': 'var(--cinema-gold-dark)',
          velvet: 'var(--cinema-velvet)',
        },
      },
      borderRadius: {
        paper: 'var(--radius-sm)',
        panel: 'var(--radius-md)',
      },
      boxShadow: {
        panel: 'var(--shadow-panel)',
        float: 'var(--shadow-float)',
      },
      transitionTimingFunction: {
        spring: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
      },
      fontFamily: {
        display: ['Cinzel', 'serif'],
        body: ["'LXGW WenKai'", "'Noto Serif SC'", "'PingFang SC'", "'Microsoft YaHei'", 'Georgia', 'serif'],
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
  plugins: [
    require('@tailwindcss/typography'),
  ],
}
