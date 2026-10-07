import { describe, expect, test } from 'bun:test'
import { assertRendererChunks, type RendererChunkBundle } from './rendererChunkContract'

// Keep this literal contract independent of the later bundle-check implementation.
const rendererModules = [
	'src/artifacts/renderers/DiffRenderer.tsx',
	'src/artifacts/renderers/TextRenderer.tsx',
	'src/artifacts/renderers/MarkdownRenderer.tsx',
	'src/artifacts/renderers/TableRenderer.tsx',
	'src/artifacts/renderers/ChartRenderer.tsx',
	'src/artifacts/renderers/JsonRenderer.tsx',
	'src/artifacts/renderers/GraphRenderer.tsx',
	'src/artifacts/renderers/TraceRenderer.tsx',
	'src/artifacts/renderers/TimelineRenderer.tsx',
] as const

type Chunk = Extract<RendererChunkBundle[string], { type: 'chunk' }>

const appRoot = '/workspace/apps/godspeed-cognitive-ui/'

function moduleID(suffix: string): string {
	return `${appRoot}${suffix}`
}

function makeRendererChunk(suffix: string, fileName: string): Chunk {
	return {
		type: 'chunk',
		fileName,
		isDynamicEntry: true,
		dynamicImports: [],
		modules: { [moduleID(suffix)]: {} },
	}
}

function makeValidBundle(): Record<string, RendererChunkBundle[string]> {
	const renderers = rendererModules.map((suffix, index) => ({
		suffix,
		fileName: `assets/renderers/renderer-${index + 1}.js`,
	}))
	const dynamicImports = renderers.map((renderer) => renderer.fileName)
	dynamicImports.push('assets/shared-dynamic.js')

	const bundle: Record<string, RendererChunkBundle[string]> = {
		'assets/main.js': {
			type: 'chunk',
			fileName: 'assets/main.js',
			isDynamicEntry: false,
			dynamicImports,
			modules: { [`${moduleID('src/artifacts/registry.tsx')}?import#registry`]: {} },
		},
		'assets/shared-dynamic.js': {
			type: 'chunk',
			fileName: 'assets/shared-dynamic.js',
			isDynamicEntry: true,
			dynamicImports: [],
			modules: { [`${moduleID('src/artifacts/renderer-shared.ts')}?commonjs-proxy`]: {} },
		},
		'assets/shared-static.js': {
			type: 'chunk',
			fileName: 'assets/shared-static.js',
			isDynamicEntry: false,
			dynamicImports: [],
			modules: { [moduleID('src/shared/renderer-utils.ts')]: {} },
		},
		'assets/renderer.css': { type: 'asset' },
	}
	for (const renderer of renderers) {
		bundle[renderer.fileName] = makeRendererChunk(renderer.suffix, renderer.fileName)
	}
	return bundle
}

function expectContractError(bundle: RendererChunkBundle, message: string): void {
	expect(() => assertRendererChunks(bundle)).toThrow(message)
}

describe('emitted renderer chunk contract', () => {
	test('accepts nine distinct dynamic renderer entries linked from the registry', () => {
		const bundle = makeValidBundle()
		const before = structuredClone(bundle)

		expect(() => assertRendererChunks(bundle)).not.toThrow()
		expect(bundle).toEqual(before)
	})

	test('matches full renderer suffixes in Windows paths with query and hash suffixes', () => {
		const bundle = makeValidBundle()
		for (const suffix of rendererModules) {
			const fileName = Object.keys(bundle).find((name) => {
				const entry = bundle[name]
				return entry?.type === 'chunk' && Object.keys(entry.modules).some((id) => id.endsWith(suffix))
			})
			expect(fileName).toBeDefined()
			if (!fileName) continue
			const entry = bundle[fileName]
			if (entry?.type !== 'chunk') continue
			const oldModuleID = Object.keys(entry.modules).find((id) => id.endsWith(suffix))
			expect(oldModuleID).toBeDefined()
			if (!oldModuleID) continue
			const windowsModuleID = `C:\\work\\sea-rs\\apps\\godspeed-cognitive-ui\\${suffix.replaceAll('/', '\\')}?v=abc#module`
			const modules = { ...entry.modules }
			delete modules[oldModuleID]
			modules[windowsModuleID] = {}
			bundle[fileName] = { ...entry, modules }
		}
		const before = structuredClone(bundle)

		expect(() => assertRendererChunks(bundle)).not.toThrow()
		expect(bundle).toEqual(before)
	})

	test('allows unrelated shared chunks and additional dynamic dependencies', () => {
		const bundle = makeValidBundle()
		const before = structuredClone(bundle)

		expect(() => assertRendererChunks(bundle)).not.toThrow()
		expect(bundle).toEqual(before)
	})

	test('rejects a missing renderer module even when a same-named asset exists', () => {
		const bundle = makeValidBundle()
		const missing = 'src/artifacts/renderers/DiffRenderer.tsx'
		const fileName = Object.keys(bundle).find((name) => {
			const entry = bundle[name]
			return entry?.type === 'chunk' && Object.keys(entry.modules).some((id) => id.endsWith(missing))
		})!
		delete bundle[fileName]
		bundle[missing] = { type: 'asset' }

		expectContractError(bundle, missing)
	})

	test('rejects one renderer module placed in two output chunks', () => {
		const bundle = makeValidBundle()
		const duplicate = rendererModules[0]
		const original = Object.keys(bundle).find((name) => {
			const entry = bundle[name]
			return entry?.type === 'chunk' && Object.keys(entry.modules).some((id) => id.endsWith(duplicate))
		})!
		bundle['assets/renderers/duplicate-diff.js'] = makeRendererChunk(duplicate, 'assets/renderers/duplicate-diff.js')
		const main = bundle['assets/main.js']
		if (main?.type === 'chunk') {
			bundle['assets/main.js'] = {
				...main,
				dynamicImports: [...main.dynamicImports, 'assets/renderers/duplicate-diff.js'],
			}
		}
		expect(bundle[original]?.type).toBe('chunk')

		expectContractError(bundle, duplicate)
	})

	test('rejects two renderer modules coalesced into one output filename', () => {
		const bundle = makeValidBundle()
		const diff = 'src/artifacts/renderers/DiffRenderer.tsx'
		const text = 'src/artifacts/renderers/TextRenderer.tsx'
		const diffFile = Object.keys(bundle).find((name) => {
			const entry = bundle[name]
			return entry?.type === 'chunk' && Object.keys(entry.modules).some((id) => id.endsWith(diff))
		})!
		const textFile = Object.keys(bundle).find((name) => {
			const entry = bundle[name]
			return entry?.type === 'chunk' && Object.keys(entry.modules).some((id) => id.endsWith(text))
		})!
		const diffChunk = bundle[diffFile]
		const textChunk = bundle[textFile]
		if (diffChunk?.type !== 'chunk' || textChunk?.type !== 'chunk') throw new Error('invalid fixture setup')
		delete bundle[diffFile]
		delete bundle[textFile]
		const coalescedFile = 'assets/renderers/diff-and-text.js'
		bundle[coalescedFile] = {
			...diffChunk,
			fileName: coalescedFile,
			modules: { ...diffChunk.modules, ...textChunk.modules },
		}
		const main = bundle['assets/main.js']
		if (main?.type === 'chunk') {
			bundle['assets/main.js'] = {
				...main,
				dynamicImports: main.dynamicImports.filter((name) => name !== diffFile && name !== textFile).concat(coalescedFile),
			}
		}

		expectContractError(bundle, 'distinct')
	})

	test('rejects a renderer chunk that is not a dynamic entry', () => {
		const bundle = makeValidBundle()
		const suffix = 'src/artifacts/renderers/GraphRenderer.tsx'
		const fileName = Object.keys(bundle).find((name) => {
			const entry = bundle[name]
			return entry?.type === 'chunk' && Object.keys(entry.modules).some((id) => id.endsWith(suffix))
		})!
		const entry = bundle[fileName]
		if (entry?.type === 'chunk') bundle[fileName] = { ...entry, isDynamicEntry: false }

		expectContractError(bundle, suffix)
	})

	test('rejects a renderer chunk not linked by a registry dynamic import', () => {
		const bundle = makeValidBundle()
		const suffix = 'src/artifacts/renderers/TraceRenderer.tsx'
		const fileName = Object.keys(bundle).find((name) => {
			const entry = bundle[name]
			return entry?.type === 'chunk' && Object.keys(entry.modules).some((id) => id.endsWith(suffix))
		})!
		const main = bundle['assets/main.js']
		if (main?.type === 'chunk') {
			bundle['assets/main.js'] = {
				...main,
				dynamicImports: main.dynamicImports.filter((name) => name !== fileName),
			}
		}

		expectContractError(bundle, suffix)
	})

	test('rejects missing registry module placement', () => {
		const bundle = makeValidBundle()
		const main = bundle['assets/main.js']
		if (main?.type !== 'chunk') throw new Error('invalid fixture setup')
		bundle['assets/main.js'] = { ...main, modules: {} }

		expectContractError(bundle, 'registry')
	})

	test('rejects duplicate registry module placement', () => {
		const bundle = makeValidBundle()
		bundle['assets/duplicate-registry.js'] = {
			type: 'chunk',
			fileName: 'assets/duplicate-registry.js',
			isDynamicEntry: false,
			dynamicImports: [],
			modules: { [`${moduleID('src/artifacts/registry.tsx')}?import`]: {} },
		}

		expectContractError(bundle, 'registry')
	})

	test('rejects an unrelated module with a renderer basename', () => {
		const bundle = makeValidBundle()
		const missing = 'src/artifacts/renderers/DiffRenderer.tsx'
		const fileName = Object.keys(bundle).find((name) => {
			const entry = bundle[name]
			return entry?.type === 'chunk' && Object.keys(entry.modules).some((id) => id.endsWith(missing))
		})!
		delete bundle[fileName]
		bundle['assets/renderers/impostor.js'] = {
			type: 'chunk',
			fileName: 'assets/renderers/impostor.js',
			isDynamicEntry: true,
			dynamicImports: [],
			modules: { '/workspace/test-fixtures/DiffRenderer.tsx': {} },
		}

		expectContractError(bundle, missing)
	})

	test('rejects a renderer suffix whose src prefix is not a path segment', () => {
		const bundle = makeValidBundle()
		const missing = 'src/artifacts/renderers/DiffRenderer.tsx'
		const fileName = Object.keys(bundle).find((name) => {
			const entry = bundle[name]
			return entry?.type === 'chunk' && Object.keys(entry.modules).some((id) => id.endsWith(missing))
		})!
		const entry = bundle[fileName]
		if (entry?.type !== 'chunk') throw new Error('invalid fixture setup')
		const originalModuleID = Object.keys(entry.modules).find((id) => id.endsWith(missing))!
		const modules = { ...entry.modules }
		delete modules[originalModuleID]
		modules['/workspace/not-src/artifacts/renderers/DiffRenderer.tsx'] = {}
		bundle[fileName] = { ...entry, modules }

		expectContractError(bundle, missing)
	})

	test('rejects a registry suffix whose src prefix is not a path segment', () => {
		const bundle = makeValidBundle()
		const missing = 'src/artifacts/registry.tsx'
		const main = bundle['assets/main.js']
		if (main?.type !== 'chunk') throw new Error('invalid fixture setup')
		const originalModuleID = Object.keys(main.modules).find((id) => id.includes(missing))!
		const modules = { ...main.modules }
		delete modules[originalModuleID]
		modules['/workspace/not-src/artifacts/registry.tsx'] = {}
		bundle['assets/main.js'] = { ...main, modules }

		expectContractError(bundle, missing)
	})
})
