import React from 'react'
import { useEffect, useState } from 'react'
import { getHealth } from './api'

const App: React.FC = () => {
  const [backendStatus, setBackendStatus] = useState<string>('Checking...')

  useEffect(() => {
    getHealth()
      .then(() => setBackendStatus('Backend: OK'))
      .catch(() => setBackendStatus('Backend: unreachable'))
  }, [])

  return (
    <div style={{ fontFamily: 'system-ui, sans-serif', padding: '1.5rem' }}>
      <h1>Rustfolio</h1>
      <p>A Rust-powered portfolio tracker and analytics dashboard.</p>
      <p><strong>{backendStatus}</strong></p>
      <hr />
      <h2>Next steps</h2>
      <ol>
        <li>Implement portfolio, position, and transaction models in the backend.</li>
        <li>Create REST endpoints for basic CRUD operations.</li>
        <li>Replace this placeholder UI with a real dashboard and charts.</li>
      </ol>
    </div>
  )
}

export default App
