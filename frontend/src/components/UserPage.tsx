import React, { useState, useEffect } from 'react'
import axios from 'axios'

interface User {
  id: string
  name: string
  email: string
  created_at: string
}

function UserPage() {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState(true)
  const [users, setUsers] = useState<User[]>([])

  useEffect(() => {
    fetchUsers()
  }, [])

  const fetchUsers = async () => {
    try {
      const response = await axios.get('/api/users')
      setUsers(response.data)
      if (response.data.length > 0) {
        setUser(response.data[0]) // Set first user as current user for demo
      }
    } catch (error) {
      console.error('Error fetching users:', error)
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return <div className="loading">Loading user information...</div>
  }

  return (
    <div className="page-container">
      <h2>User Profile</h2>
      
      {user ? (
        <div className="user-profile">
          <div className="profile-card">
            <h3>{user.name}</h3>
            <p><strong>Email:</strong> {user.email}</p>
            <p><strong>User ID:</strong> {user.id}</p>
            <p><strong>Member since:</strong> {new Date(user.created_at).toLocaleDateString()}</p>
          </div>
        </div>
      ) : (
        <p>No user profile found. Please log in or create an account.</p>
      )}

      <div className="users-section">
        <h3>All Users</h3>
        {users.length === 0 ? (
          <p>No users found.</p>
        ) : (
          <div className="users-list">
            {users.map((u) => (
              <div key={u.id} className="user-card">
                <h4>{u.name}</h4>
                <p>{u.email}</p>
                <small>Joined: {new Date(u.created_at).toLocaleDateString()}</small>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

export default UserPage