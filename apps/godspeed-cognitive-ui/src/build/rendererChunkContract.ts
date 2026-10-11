/** The Rollup output fields needed to verify the renderer chunk graph. */
export type RendererChunkBundle = Readonly<Record<string, RendererChunkBundleEntry>>

type RendererChunkBundleEntry = RendererChunkAsset | RendererChunkOutput

type RendererChunkAsset = Readonly<{
	type: 'asset'
}>

type RendererChunkOutput = Readonly<{
	type: 'chunk'
	fileName: string
	isDynamicEntry: boolean
	dynamicImports: readonly string[]
	modules: Readonly<Record<string, unknown>>
}>

const rendererModulePaths = [
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

const registryModulePath = 'src/artifacts/registry.tsx'

function normalizedModuleId(moduleId: string): string {
	const suffixStart = moduleId.search(/[?#]/)
	const path = suffixStart === -1 ? moduleId : moduleId.slice(0, suffixStart)
	return path.replaceAll('\\', '/')
}

function containsModulePath(modules: Readonly<Record<string, unknown>>, requiredPath: string): boolean {
	return Object.keys(modules).some((moduleId) => {
		const path = normalizedModuleId(moduleId)
		return path === requiredPath || path.endsWith(`/${requiredPath}`)
	})
}

function chunksContainingModule(bundle: RendererChunkBundle, requiredPath: string): RendererChunkOutput[] {
	return Object.values(bundle).filter(
		(entry): entry is RendererChunkOutput =>
			entry.type === 'chunk' && containsModulePath(entry.modules, requiredPath),
	)
}

/** Assert that each lazy renderer is emitted separately and linked from the registry chunk. */
export function assertRendererChunks(bundle: RendererChunkBundle): void {
	const rendererChunks = rendererModulePaths.map((modulePath) => {
		const matches = chunksContainingModule(bundle, modulePath)
		if (matches.length === 0) {
			throw new Error(`Missing renderer module ${modulePath}`)
		}
		if (matches.length > 1) {
			throw new Error(`Renderer module ${modulePath} appears in multiple output chunks`)
		}
		return { modulePath, chunk: matches[0]! }
	})

	const rendererFileNames = new Set<string>()
	for (const { modulePath, chunk } of rendererChunks) {
		if (rendererFileNames.has(chunk.fileName)) {
			throw new Error(`Renderer chunks must have distinct fileNames; ${modulePath} shares ${chunk.fileName}`)
		}
		rendererFileNames.add(chunk.fileName)
		if (!chunk.isDynamicEntry) {
			throw new Error(`Renderer module ${modulePath} is not emitted as a dynamic entry`)
		}
	}

	const registryChunks = chunksContainingModule(bundle, registryModulePath)
	if (registryChunks.length === 0) {
		throw new Error(`Missing registry module ${registryModulePath}`)
	}
	if (registryChunks.length > 1) {
		throw new Error(`Registry module ${registryModulePath} appears in multiple output chunks`)
	}

	const registryChunk = registryChunks[0]!
	for (const { modulePath, chunk } of rendererChunks) {
		if (!registryChunk.dynamicImports.includes(chunk.fileName)) {
			throw new Error(`Registry chunk is missing dynamic import for ${modulePath} (${chunk.fileName})`)
		}
	}
}
