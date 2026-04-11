import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: '127.0.0.1',
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    hmr: {
      protocol: 'ws',
      host: '127.0.0.1',
      port: 5173,
    },
    cors: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  base: './',
  build: {
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: 'dist',
    emptyOutDir: true,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@components': path.resolve(__dirname, './src/components'),
      '@pages': path.resolve(__dirname, './src/pages'),
      '@hooks': path.resolve(__dirname, './src/hooks'),
      '@stores': path.resolve(__dirname, './src/stores'),
      '@types': path.resolve(__dirname, './src/types'),
      '@utils': path.resolve(__dirname, './src/utils'),
      '@services': path.resolve(__dirname, './src/services'),
    },
  },
}))
