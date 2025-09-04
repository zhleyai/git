import React, { useState, useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom'
import axios from 'axios'
import UserPage from './components/UserPage'
import WarehousePage from './components/WarehousePage'
import ExplorationPage from './components/ExplorationPage'
import './App.css'

interface Repository {
  id: string
  name: string
  description: string | null
  default_branch: string
  created_at: string
}

function App() {
  const [repositories, setRepositories] = useState<Repository[]>([])
  const [loading, setLoading] = useState(true)
  const [newRepoName, setNewRepoName] = useState('')
  const [newRepoDescription, setNewRepoDescription] = useState('')
  const [showCreateForm, setShowCreateForm] = useState(false)

  useEffect(() => {
    fetchRepositories()
  }, [])

  const fetchRepositories = async () => {
    try {
      const response = await axios.get('/api/repositories')
      setRepositories(response.data)
    } catch (error) {
      console.error('Error fetching repositories:', error)
    } finally {
      setLoading(false)
    }
  }

  const createRepository = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newRepoName.trim()) return

    try {
      await axios.post('/api/repositories', {
        name: newRepoName.trim(),
        description: newRepoDescription.trim() || null
      })
      setNewRepoName('')
      setNewRepoDescription('')
      setShowCreateForm(false)
      fetchRepositories()
    } catch (error) {
      console.error('Error creating repository:', error)
    }
  }

  if (loading) {
    return <div className="loading">Loading...</div>
  }

  return (
    <Router>
      <div className="App">
        <header className="App-header">
          <h1>Git Server</h1>
          <nav>
            <Link to="/">Repositories</Link>
            <Link to="/users">Users</Link>
            <Link to="/warehouse">Warehouse</Link>
            <Link to="/explore">Explore</Link>
          </nav>
        </header>

        <main className="App-main">
          <Routes>
            <Route path="/" element={
              <div>
                <div className="header-actions">
                  <h2>Repositories</h2>
                  <button 
                    onClick={() => setShowCreateForm(!showCreateForm)}
                    className="create-button"
                  >
                    {showCreateForm ? 'Cancel' : 'Create Repository'}
                  </button>
                </div>

                {showCreateForm && (
                  <form onSubmit={createRepository} className="create-form">
                    <div className="form-group">
                      <label htmlFor="repo-name">Repository Name:</label>
                      <input
                        id="repo-name"
                        type="text"
                        value={newRepoName}
                        onChange={(e) => setNewRepoName(e.target.value)}
                        placeholder="Enter repository name"
                        required
                      />
                    </div>
                    <div className="form-group">
                      <label htmlFor="repo-description">Description (optional):</label>
                      <textarea
                        id="repo-description"
                        value={newRepoDescription}
                        onChange={(e) => setNewRepoDescription(e.target.value)}
                        placeholder="Enter repository description"
                        rows={3}
                      />
                    </div>
                    <div className="form-actions">
                      <button type="submit" className="submit-button">
                        Create Repository
                      </button>
                    </div>
                  </form>
                )}

                <div className="repositories-list">
                  {repositories.length === 0 ? (
                    <p className="no-repositories">No repositories found. Create one to get started!</p>
                  ) : (
                    repositories.map((repo) => (
                      <div key={repo.id} className="repository-card">
                        <h3>{repo.name}</h3>
                        {repo.description && <p className="description">{repo.description}</p>}
                        <div className="repository-meta">
                          <span>Default branch: {repo.default_branch}</span>
                          <span>Created: {new Date(repo.created_at).toLocaleDateString()}</span>
                        </div>
                        <div className="repository-actions">
                          <div className="clone-urls">
                            <div>
                              <label>HTTPS:</label>
                              <code>git clone http://localhost:8080/git/{repo.name}</code>
                            </div>
                            <div>
                              <label>SSH:</label>
                              <code>git clone ssh://git@localhost:2222/{repo.name}</code>
                            </div>
                          </div>
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            } />
            <Route path="/users" element={<UserPage />} />
            <Route path="/warehouse" element={<WarehousePage />} />
            <Route path="/explore" element={<ExplorationPage />} />
          </Routes>
        </main>
      </div>
    </Router>
  )
}

export default App