/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
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
          500: 'var(--cinema-500)',
          gold: 'var(--cinema-gold)',
          'gold-light': 'var(--cinema-gold-light)',
          'gold-dark': 'var(--cinema-gold-dark)',
          velvet: 'var(--cinema-velvet)',
        },
        status: {
          success: 'var(--status-success)',
          'success-dim': 'var(--status-success-dim)',
          warning: 'var(--status-warning)',
          danger: 'var(--status-danger)',
          'danger-dim': 'var(--status-danger-dim)',
        },
        borderSubtle: 'var(--border-subtle)',
        // AI 原生组件语义令牌（P1）：tokens.css（幕后）/frontstage.css（幕前）各自定义
        ai: {
          surface: 'var(--ai-surface)',
          inset: 'var(--ai-inset)',
          field: 'var(--ai-field)',
          hover: 'var(--ai-hover)',
          'hover-2': 'var(--ai-hover-2)',
          ink: 'var(--ai-ink)',
          'ink-2': 'var(--ai-ink-2)',
          'ink-3': 'var(--ai-ink-3)',
          line: 'var(--ai-line)',
          'line-strong': 'var(--ai-line-strong)',
          accent: 'var(--ai-accent)',
          'accent-ink': 'var(--ai-accent-ink)',
          'accent-tint': 'var(--ai-accent-tint)',
          green: 'var(--ai-green)',
          red: 'var(--ai-red)',
          orange: 'var(--ai-orange)',
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
        body: [
          "'LXGW WenKai'",
          "'Noto Serif SC'",
          "'PingFang SC'",
          "'Microsoft YaHei'",
          'Georgia',
          'serif',
        ],
        mono: ['JetBrains Mono', 'monospace'],
      },
      animation: {
        'fade-up': 'fadeUp 0.6s ease-out',
        'fade-in': 'fadeIn 0.4s ease-out',
        'slide-left': 'slideLeft 0.5s ease-out',
        'pulse-slow': 'pulse 3s ease-in-out infinite',
        'spin-slow': 'spin 3s linear infinite',
        // AI 原生组件动画（P1）。ai-fade-up 区别于既有 fade-up（0.6s 大幕入场），勿复用。
        'pixel-on': 'pixel-on 650ms ease-in-out infinite',
        'shimmer-text': 'shimmer-text 1.4s linear infinite',
        'ai-fade-up': 'fade-up 350ms cubic-bezier(0.23, 1, 0.32, 1) both',
        'pop-in': 'pop-in 250ms cubic-bezier(0.23, 1, 0.32, 1) both',
        'stream-in': 'stream-in 420ms cubic-bezier(0.22, 0.61, 0.25, 1) both',
        'ai-spin': 'ai-spin 700ms linear infinite',
        'eq-bounce': 'eq-bounce 900ms ease-in-out infinite',
        'ai-sweep': 'ai-sweep 950ms ease-out both',
        'ai-blink': 'ai-blink 1.1s steps(2, start) infinite',
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
        'pixel-on': {
          '0%, 100%': { opacity: '0.15' },
          '50%': { opacity: '1' },
        },
        'shimmer-text': {
          '0%': { backgroundPosition: '200% 0' },
          '100%': { backgroundPosition: '-200% 0' },
        },
        'fade-up': {
          '0%': { opacity: '0', transform: 'translateY(6px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'pop-in': {
          '0%': { opacity: '0', transform: 'scale(0.92)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
        'stream-in': {
          '0%': { opacity: '0', filter: 'blur(6px)' },
          '100%': { opacity: '1', filter: 'blur(0)' },
        },
        'ai-spin': {
          '100%': { transform: 'rotate(360deg)' },
        },
        'eq-bounce': {
          '0%, 100%': { transform: 'scaleY(0.35)' },
          '50%': { transform: 'scaleY(1)' },
        },
        'ai-sweep': {
          '0%': { transform: 'translateX(-120%)', opacity: '0' },
          '12%': { opacity: '0.9' },
          '85%': { opacity: '0.9' },
          '100%': { transform: 'translateX(240%)', opacity: '0' },
        },
        'ai-blink': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0' },
        },
      },
      backdropBlur: {
        cinema: '12px',
      },
    },
  },
  plugins: [require('@tailwindcss/typography')],
};
