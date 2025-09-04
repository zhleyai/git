import React, { useState, useEffect } from 'react'
import axios from 'axios'

interface Repository {
  id: string
  name: string
  description: string | null
  default_branch: string
  created_at: string
}

interface User {
  id: string
  name: string
  email: string
  created_at: string
}

function ExplorationPage() {
  const [repositories, setRepositories] = useState<Repository[]>([])
  const [users, setUsers] = useState<User[]>([])
  const [loading, setLoading] = useState(true)
  const [searchTerm, setSearchTerm] = useState('')
  const [activeTab, setActiveTab] = useState<'repositories' | 'users'>('repositories')

  useEffect(() => {
    fetchExplorationData()
  }, [])

  const fetchExplorationData = async () => {
    try {
      const [reposResponse, usersResponse] = await Promise.all([
        axios.get('/api/repositories'),
        axios.get('/api/users')
      ])
      setRepositories(reposResponse.data)
      setUsers(usersResponse.data)
    } catch (error) {
      console.error('Error fetching exploration data:', error)
    } finally {
      setLoading(false)
    }
  }

  const filteredRepositories = repositories.filter(repo =>
    repo.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    (repo.description && repo.description.toLowerCase().includes(searchTerm.toLowerCase()))
  )

  const filteredUsers = users.filter(user =>
    user.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    user.email.toLowerCase().includes(searchTerm.toLowerCase())
  )

  if (loading) {
    return <div className="loading">Loading exploration data...</div>
  }

  return (
    <div className="page-container">
      <h2>Explore</h2>
      
      <div className="exploration-controls">
        <div className="search-bar">
          <input
            type="text"
            placeholder="Search repositories, users, or content..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="search-input"
          />
        </div>
        
        <div className="tab-controls">
          <button 
            className={`tab-button ${activeTab === 'repositories' ? 'active' : ''}`}
            onClick={() => setActiveTab('repositories')}
          >
            Repositories ({filteredRepositories.length})
          </button>
          <button 
            className={`tab-button ${activeTab === 'users' ? 'active' : ''}`}
            onClick={() => setActiveTab('users')}
          >
            Users ({filteredUsers.length})
          </button>
        </div>
      </div>

      <div className="exploration-content">
        {activeTab === 'repositories' ? (
          <div className="repositories-exploration">
            <h3>Discover Repositories</h3>
            {filteredRepositories.length === 0 ? (
              <p>No repositories found matching your search.</p>
            ) : (
              <div className="exploration-grid">
                {filteredRepositories.map((repo) => (
                  <div key={repo.id} className="exploration-card">
                    <div className="card-header">
                      <h4>{repo.name}</h4>
                      <span className="branch-tag">{repo.default_branch}</span>
                    </div>
                    {repo.description && (
                      <p className="card-description">{repo.description}</p>
                    )}
                    <div className="card-meta">
                      <span>Created: {new Date(repo.created_at).toLocaleDateString()}</span>
                    </div>
                    <div className="card-actions">
                      <button className="explore-button">Explore</button>
                      <button className="clone-button">Clone</button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        ) : (
          <div className="users-exploration">
            <h3>Discover Users</h3>
            {filteredUsers.length === 0 ? (
              <p>No users found matching your search.</p>
            ) : (
              <div className="exploration-grid">
                {filteredUsers.map((user) => (
                  <div key={user.id} className="exploration-card">
                    <div className="card-header">
                      <h4>{user.name}</h4>
                    </div>
                    <p className="card-description">{user.email}</p>
                    <div className="card-meta">
                      <span>Joined: {new Date(user.created_at).toLocaleDateString()}</span>
                    </div>
                    <div className="card-actions">
                      <button className="explore-button">View Profile</button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}

export default ExplorationPage