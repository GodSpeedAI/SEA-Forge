import { createRoot } from 'react-dom/client'

import { App } from './App'
import { buildFixtureEnvironment } from './bootstrap'
import './styles.css'

const container = document.getElementById('root')
if (!container) throw new Error('#root is missing from index.html')

const env = buildFixtureEnvironment()
void env.start()
createRoot(container).render(<App environment={env} />)
