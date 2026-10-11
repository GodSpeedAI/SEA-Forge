import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import type { Plugin } from 'vite'
import { assertRendererChunks } from './src/build/rendererChunkContract'

// Fixed port so the `casework-ui-up` / `-down` / `-status` recipes can manage the dev server.
// /api is proxied to the Go casework service (the UI's only backend) so the browser stays same-origin.
const api = { '/api': { target: 'http://127.0.0.1:4179', changeOrigin: false } }

function rendererChunkContractPlugin(): Plugin {
  return {
    name: 'renderer-chunk-contract',
    apply: 'build',
    generateBundle(_options, bundle) {
      assertRendererChunks(bundle)
      console.info('[renderer-chunk-contract] verified nine distinct renderer chunks')
    },
  }
}

export default defineConfig({
  plugins: [react(), rendererChunkContractPlugin()],
  server: { host: '127.0.0.1', port: 4178, strictPort: true, proxy: api },
  preview: { host: '127.0.0.1', port: 4178, strictPort: true, proxy: api },
})
