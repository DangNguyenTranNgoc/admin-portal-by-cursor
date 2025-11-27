use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::{
    application::data_catalog_service::{DataCatalogRepository, DatasetSchema},
    domain::{
        permission::{Permission, PermissionBits, PermissionRepository},
        user::{
            CreateUserCommand, User, UserCredentials, UserGroup, UserId, UserRepository,
            UserStatus, UserWithGroups,
        },
    },
    shared::errors::DomainError,
};

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<UserWithGroups>, DomainError> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT u.id,
                   u.email,
                   u.first_name,
                   u.last_name,
                   u.status,
                   u.last_login,
                   u.created_time,
                   u.updated_time,
                   u.password as password_hash,
                   u.salt,
                   g.id as group_id,
                   g.name as group_name
            FROM users u
            LEFT JOIN group_membership gm ON gm.user_id = u.id
            LEFT JOIN "group" g ON g.id = gm.group_id
            WHERE u.id = $1
            "#,
        )
        .bind(id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        Ok(group_rows(rows))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<UserWithGroups>, DomainError> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT u.id,
                   u.email,
                   u.first_name,
                   u.last_name,
                   u.status,
                   u.last_login,
                   u.created_time,
                   u.updated_time,
                   u.password as password_hash,
                   u.salt,
                   g.id as group_id,
                   g.name as group_name
            FROM users u
            LEFT JOIN group_membership gm ON gm.user_id = u.id
            LEFT JOIN "group" g ON g.id = gm.group_id
            WHERE u.email = $1
            "#,
        )
        .bind(email)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        Ok(group_rows(rows))
    }

    async fn list(&self) -> Result<Vec<UserWithGroups>, DomainError> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT u.id,
                   u.email,
                   u.first_name,
                   u.last_name,
                   u.status,
                   u.last_login,
                   u.created_time,
                   u.updated_time,
                   u.password as password_hash,
                   u.salt,
                   g.id as group_id,
                   g.name as group_name
            FROM users u
            LEFT JOIN group_membership gm ON gm.user_id = u.id
            LEFT JOIN "group" g ON g.id = gm.group_id
            ORDER BY u.id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        Ok(group_rows_list(rows))
    }

    async fn create(&self, cmd: CreateUserCommand) -> Result<UserWithGroups, DomainError> {
        let CreateUserCommand {
            email,
            first_name,
            last_name,
            password_hash,
            salt,
            groups,
        } = cmd;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        let record = sqlx::query_as::<_, UserInsertRow>(
            r#"
            INSERT INTO users (email, first_name, last_name, password, salt, status)
            VALUES ($1, $2, $3, $4, $5, 'active')
            RETURNING id, email, first_name, last_name, status, last_login, created_time, updated_time
            "#,
        )
        .bind(&email)
        .bind(&first_name)
        .bind(&last_name)
        .bind(&password_hash)
        .bind(&salt)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        for group_id in groups {
            sqlx::query(
                r#"
                INSERT INTO group_membership (user_id, group_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(record.id)
            .bind(group_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Unexpected(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        Ok(UserWithGroups {
            user: record.into_user()?,
            groups: vec![],
            credentials: Some(UserCredentials {
                password_hash,
                salt,
            }),
        })
    }
}

#[derive(FromRow)]
struct UserRow {
    id: i64,
    email: String,
    first_name: String,
    last_name: String,
    status: String,
    last_login: Option<DateTime<Utc>>,
    created_time: DateTime<Utc>,
    updated_time: DateTime<Utc>,
    password_hash: String,
    salt: String,
    group_id: Option<i64>,
    group_name: Option<String>,
}

#[derive(FromRow)]
struct UserInsertRow {
    id: i64,
    email: String,
    first_name: String,
    last_name: String,
    status: String,
    last_login: Option<DateTime<Utc>>,
    created_time: DateTime<Utc>,
    updated_time: DateTime<Utc>,
}

impl UserInsertRow {
    fn into_user(self) -> Result<User, DomainError> {
        Ok(User {
            id: UserId(self.id),
            email: self.email,
            first_name: self.first_name,
            last_name: self.last_name,
            status: UserStatus::try_from(self.status)?,
            last_login: self.last_login,
            created_at: self.created_time,
            updated_at: self.updated_time,
        })
    }
}

fn group_rows(rows: Vec<UserRow>) -> Option<UserWithGroups> {
    let list = group_rows_list(rows);
    list.into_iter().next()
}

fn group_rows_list(rows: Vec<UserRow>) -> Vec<UserWithGroups> {
    let mut grouped = vec![];
    let mut current_id: Option<i64> = None;
    let mut current: Option<UserWithGroups> = None;

    for row in rows {
        if current_id != Some(row.id) {
            if let Some(user) = current.take() {
                grouped.push(user);
            }

            let user = User {
                id: UserId(row.id),
                email: row.email.clone(),
                first_name: row.first_name.clone(),
                last_name: row.last_name.clone(),
                status: UserStatus::try_from(row.status.clone()).unwrap_or(UserStatus::Active),
                last_login: row.last_login,
                created_at: row.created_time,
                updated_at: row.updated_time,
            };

            current = Some(UserWithGroups {
                user,
                groups: vec![],
                credentials: Some(UserCredentials {
                    password_hash: row.password_hash.clone(),
                    salt: row.salt.clone(),
                }),
            });
            current_id = Some(row.id);
        }

        if let (Some(group_id), Some(group_name)) = (row.group_id, row.group_name.clone()) {
            if let Some(ref mut agg) = current {
                agg.groups.push(UserGroup {
                    id: group_id,
                    name: group_name,
                });
            }
        }
    }

    if let Some(user) = current {
        grouped.push(user);
    }

    grouped
}

pub struct PgPermissionRepository {
    pool: PgPool,
}

impl PgPermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepository for PgPermissionRepository {
    async fn find_permissions_for_groups(
        &self,
        group_ids: &[i64],
    ) -> Result<Vec<Permission>, DomainError> {
        if group_ids.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT resource, group_id, perm_value
            FROM permissions_group
            WHERE group_id = ANY($1)
            "#,
        )
        .bind(group_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let bits = map_perm_value(row.perm_value)?;
                Some(Permission {
                    resource: row.resource,
                    group_id: row.group_id,
                    bits,
                })
            })
            .collect())
    }
}

#[derive(FromRow)]
struct PermissionRow {
    resource: String,
    group_id: i64,
    perm_value: i32,
}

fn map_perm_value(value: i32) -> Option<PermissionBits> {
    match value {
        1 => Some(PermissionBits::READ),
        2 => Some(PermissionBits::WRITE),
        3 => Some(PermissionBits::READ | PermissionBits::WRITE),
        4 => Some(PermissionBits::DELETE),
        5 => Some(PermissionBits::DELETE | PermissionBits::READ),
        6 => Some(PermissionBits::DELETE | PermissionBits::WRITE),
        7 => Some(PermissionBits::full()),
        17 => Some(PermissionBits::full()),
        _ => None,
    }
}

pub struct PgDataCatalogRepository {
    pool: PgPool,
}

impl PgDataCatalogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DataCatalogRepository for PgDataCatalogRepository {
    async fn list_schemas(&self) -> Result<Vec<DatasetSchema>, DomainError> {
        let rows = sqlx::query_as::<_, DatasetRow>(
            r#"
            SELECT id, name, description, base_query
            FROM data_catalog
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| DatasetSchema {
                id: row.id,
                name: row.name,
                description: row.description,
                base_query: row.base_query,
            })
            .collect())
    }

    async fn resolve_query(&self, dataset_id: i64) -> Result<String, DomainError> {
        let row = sqlx::query_as::<_, DatasetRow>(
            r#"
            SELECT id, name, description, base_query
            FROM data_catalog
            WHERE id = $1
            "#,
        )
        .bind(dataset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Unexpected(e.to_string()))?;

        let Some(row) = row else {
            return Err(DomainError::UserNotFound);
        };

        Ok(row.base_query)
    }
}

#[derive(FromRow)]
struct DatasetRow {
    id: i64,
    name: String,
    description: Option<String>,
    base_query: String,
}
