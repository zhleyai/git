import React, { useState, useEffect } from 'react'
import axios from 'axios'

interface Repository {
  id: string
  name: string
  description: string | null
  default_branch: string
  created_at: string
}

interface StorageStats {
  totalRepositories: number
  totalSize: string
  recentActivity: Array<{
    repository: string
    action: string
    timestamp: string
  }>
}

function WarehousePage() {
  const [repositories, setRepositories] = useState<Repository[]>([])
  const [loading, setLoading] = useState(true)
  const [storageStats, setStorageStats] = useState<StorageStats>({
    totalRepositories: 0,
    totalSize: 'Unknown',
    recentActivity: []
  })

  useEffect(() => {
    fetchWarehouseData()
  }, [])

  const fetchWarehouseData = async () => {
    try {
      const response = await axios.get('/api/repositories')
      setRepositories(response.data)
      
      // Mock storage statistics for demonstration
      setStorageStats({
        totalRepositories: response.data.length,
        totalSize: `${(response.data.length * 1.5).toFixed(1)} MB`,
        recentActivity: response.data.slice(0, 5).map((repo: Repository) => ({
          repository: repo.name,
          action: 'Repository created',
          timestamp: repo.created_at
        }))
      })
    } catch (error) {
      console.error('Error fetching warehouse data:', error)
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return <div className="loading">Loading warehouse information...</div>
  }

  return (
    <div className="page-container">
      <h2>Repository Warehouse</h2>
      
      <div className="warehouse-overview">
        <div className="stats-grid">
          <div className="stat-card">
            <h3>Total Repositories</h3>
            <p className="stat-number">{storageStats.totalRepositories}</p>
          </div>
          <div className="stat-card">
            <h3>Storage Used</h3>
            <p className="stat-number">{storageStats.totalSize}</p>
          </div>
          <div className="stat-card">
            <h3>Active Repositories</h3>
            <p className="stat-number">{repositories.length}</p>
          </div>
        </div>
      </div>

      <div className="warehouse-section">
        <h3>Recent Activity</h3>
        {storageStats.recentActivity.length === 0 ? (
          <p>No recent activity.</p>
        ) : (
          <div className="activity-list">
            {storageStats.recentActivity.map((activity, index) => (
              <div key={index} className="activity-item">
                <div className="activity-info">
                  <strong>{activity.repository}</strong>
                  <span>{activity.action}</span>
                </div>
                <div className="activity-time">
                  {new Date(activity.timestamp).toLocaleDateString()}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="warehouse-section">
        <h3>Repository Storage Details</h3>
        {repositories.length === 0 ? (
          <p>No repositories in warehouse.</p>
        ) : (
          <div className="storage-list">
            {repositories.map((repo) => (
              <div key={repo.id} className="storage-item">
                <div className="storage-info">
                  <h4>{repo.name}</h4>
                  {repo.description && <p>{repo.description}</p>}
                  <div className="storage-meta">
                    <span>Branch: {repo.default_branch}</span>
                    <span>Size: ~{Math.random() * 5 + 0.5 | 0}.{Math.random() * 9 | 0} MB</span>
                    <span>Created: {new Date(repo.created_at).toLocaleDateString()}</span>
                  </div>
                </div>
                <div className="storage-actions">
                  <button className="action-button">Archive</button>
                  <button className="action-button">Backup</button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

export default WarehousePage