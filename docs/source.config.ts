import { defineConfig } from 'fumadocs-core/config'

export default defineConfig({
  source: {
    include: ['**/*.mdx', '**/*.md'],
    exclude: ['**/node_modules/**', '**/.git/**'],
  },
  docs: {
    dir: 'content/docs',
  },
})