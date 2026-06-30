import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// Web uses the deployment base path; desktop builds pass --base=./.
const getBase = () => {
  const baseArg = process.argv.find(arg => arg.startsWith('--base='))
  if (baseArg) {
    return baseArg.split('=')[1]
  }
  return process.env.VITE_BASE_URL || process.env.BASE_URL || '/'
}

export default defineConfig({
  base: getBase(),
  plugins: [vue()],
  server: {
    watch: {
      ignored: ['**/src-tauri/target/**']
    }
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  ssgOptions: {
    script: 'async',
    formatting: 'minify'
  }
})
