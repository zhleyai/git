use crate::entities::user;
use anyhow::Result;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        username: String,
        email: String,
        password_hash: String,
        full_name: Option<String>,
        is_admin: bool,
    ) -> Result<user::Model> {
        let user = user::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set(username),
            email: Set(email),
            password_hash: Set(password_hash),
            full_name: Set(full_name),
            is_active: Set(true),
            is_admin: Set(is_admin),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let result = user.insert(&self.db).await?;
        Ok(result)
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<user::Model>> {
        let user = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(&self.db)
            .await?;
        Ok(user)
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<user::Model>> {
        let user = user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await?;
        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<user::Model>> {
        let user = user::Entity::find_by_id(id).one(&self.db).await?;
        Ok(user)
    }

    /// List all users
    pub async fn list_users(&self) -> Result<Vec<user::Model>> {
        let users = user::Entity::find().all(&self.db).await?;
        Ok(users)
    }

    /// Update user
    pub async fn update_user(
        &self,
        id: Uuid,
        username: Option<String>,
        email: Option<String>,
        password_hash: Option<String>,
        full_name: Option<String>,
        is_active: Option<bool>,
        is_admin: Option<bool>,
    ) -> Result<Option<user::Model>> {
        if let Some(existing_user) = user::Entity::find_by_id(id).one(&self.db).await? {
            let mut user_active: user::ActiveModel = existing_user.into();

            if let Some(username) = username {
                user_active.username = Set(username);
            }
            if let Some(email) = email {
                user_active.email = Set(email);
            }
            if let Some(password_hash) = password_hash {
                user_active.password_hash = Set(password_hash);
            }
            if let Some(full_name) = full_name {
                user_active.full_name = Set(Some(full_name));
            }
            if let Some(is_active) = is_active {
                user_active.is_active = Set(is_active);
            }
            if let Some(is_admin) = is_admin {
                user_active.is_admin = Set(is_admin);
            }

            user_active.updated_at = Set(Utc::now().into());
            let result = user_active.update(&self.db).await?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Delete user
    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        user::Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    /// Check if username exists
    pub async fn username_exists(&self, username: &str) -> Result<bool> {
        let count = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .count(&self.db)
            .await?;
        Ok(count > 0)
    }

    /// Check if email exists
    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let count = user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .count(&self.db)
            .await?;
        Ok(count > 0)
    }

    /// Authenticate user with username/email and password
    pub async fn authenticate(
        &self, 
        username_or_email: &str, 
        password: &str
    ) -> Result<Option<user::Model>> {
        // Try to find user by username first, then by email
        let user = match self.get_user_by_username(username_or_email).await? {
            Some(user) => Some(user),
            None => self.get_user_by_email(username_or_email).await?,
        };

        if let Some(user) = user {
            // Verify password (this would use proper bcrypt verification in production)
            if self.verify_password(password, &user.password_hash)? {
                Ok(Some(user))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Hash password (placeholder - would use bcrypt in production)
    pub fn hash_password(&self, password: &str) -> Result<String> {
        // For now, just prefix with "hashed_"
        // In production, use: bcrypt::hash(password, bcrypt::DEFAULT_COST)?
        Ok(format!("hashed_{}", password))
    }

    /// Verify password against hash (placeholder - would use bcrypt in production)  
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        // For now, just check if hash matches "hashed_" + password
        // In production, use: bcrypt::verify(password, hash)?
        Ok(hash == format!("hashed_{}", password))
    }
}