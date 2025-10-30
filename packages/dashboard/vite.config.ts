import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { fileURLToPath } from 'url'
import { dirname } from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const apiPort = process.env.ORKEE_API_PORT || '4001'
const uiPort = parseInt(process.env.ORKEE_UI_PORT || process.env.VITE_PORT || '5173', 10)

export default defineConfig({
  plugins: [react()],
  envDir: `${__dirname}/../../`,
  base: './', // Use relative paths for assets - required for Tauri
  resolve: {
    alias: {
      '@': `${__dirname}/src`,
      '@orkee/tasks': `${__dirname}/../tasks/src`,
    },
  },
  define: {
    // Pass the dynamic API port to the frontend
    'import.meta.env.VITE_ORKEE_API_PORT': JSON.stringify(apiPort),
    'import.meta.env.VITE_ORKEE_UI_PORT': JSON.stringify(process.env.ORKEE_UI_PORT || ''),
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // Core React libraries
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],

          // UI component libraries - split by usage pattern
          'ui-radix': [
            '@radix-ui/react-dialog',
            '@radix-ui/react-dropdown-menu',
            '@radix-ui/react-select',
            '@radix-ui/react-tabs',
            '@radix-ui/react-accordion',
            '@radix-ui/react-popover',
            '@radix-ui/react-tooltip',
            '@radix-ui/react-switch',
            '@radix-ui/react-checkbox',
            '@radix-ui/react-label',
            '@radix-ui/react-scroll-area',
            '@radix-ui/react-separator',
            '@radix-ui/react-slot',
            '@radix-ui/react-collapsible',
            '@radix-ui/react-alert-dialog',
            '@radix-ui/react-avatar',
            '@radix-ui/react-progress',
          ],

          // AI libraries - large and specialized
          'ai-vendor': [
            'ai',
            '@ai-sdk/react',
            '@ai-sdk/anthropic',
            '@ai-sdk/openai',
            '@ai-sdk/ui-utils',
          ],

          // Data visualization - used only on specific pages
          'charts-vendor': ['recharts', 'cytoscape', 'cytoscape-dagre', 'cytoscape-fcose', 'react-cytoscapejs', '@xyflow/react'],

          // Utilities
          'utils-vendor': [
            '@tanstack/react-query',
            '@tanstack/react-query-devtools',
            'date-fns',
            'clsx',
            'tailwind-merge',
            'class-variance-authority',
            'zod',
            'zod-to-json-schema',
          ],

          // Markdown and syntax highlighting - heavy but used in specific features
          'markdown-vendor': [
            'react-markdown',
            'remark-gfm',
            'rehype-highlight',
            'rehype-sanitize',
            'highlight.js',
            'react-diff-viewer-continued',
          ],

          // Drag and drop - specialized feature
          'dnd-vendor': [
            '@dnd-kit/core',
            '@dnd-kit/sortable',
            '@dnd-kit/utilities',
          ],

          // Tauri APIs - desktop-specific
          'tauri-vendor': [
            '@tauri-apps/api',
            '@tauri-apps/plugin-http',
            '@tauri-apps/plugin-shell',
          ],
        },
      },
    },
    // Increase chunk size warning threshold since we're now properly splitting
    chunkSizeWarningLimit: 600,
  },
  server: {
    port: uiPort,
    strictPort: false, // Allow fallback to next available port
    host: 'localhost',
    proxy: {
      '/api': {
        target: `http://localhost:${apiPort}`,
        changeOrigin: true,
      },
    },
  },
})