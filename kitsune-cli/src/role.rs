use crate::Result;
use chrono::Utc;
use clap::{Args, Subcommand, ValueEnum};
use kitsune_db::{
    custom::Role as DbRole,
    entity::{
        prelude::{Users, UsersRoles},
        users, users_roles,
    },
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter,
};
use uuid::Uuid;

#[derive(Subcommand)]
pub enum RoleSubcommand {
    /// Add a role to a user
    Add {
        /// Username of the user you want to add the role to
        username: String,

        /// Name of the role you want to add
        role: Role,
    },

    /// List all roles of a user
    List { username: String },

    /// Remove a role from a user
    Remove {
        /// Username of the user you want to add the role to
        username: String,

        /// Name of the role you want to add
        role: Role,
    },
}

#[derive(Args)]
struct AddRemoveRoleArgs {}

#[derive(Clone, Copy, ValueEnum)]
pub enum Role {
    Administrator,
}

impl From<Role> for DbRole {
    fn from(value: Role) -> Self {
        match value {
            Role::Administrator => DbRole::Administrator,
        }
    }
}

async fn add_role(db_conn: DatabaseConnection, username: &str, role: Role) -> Result<()> {
    let Some(user) = Users::find()
        .filter(users::Column::Username.eq(username))
        .one(&db_conn)
        .await?
    else {
        eprintln!("User \"{username}\" not found!");
        return Ok(());
    };

    users_roles::Model {
        id: Uuid::now_v7(),
        user_id: user.id,
        role: role.into(),
        created_at: Utc::now().into(),
    }
    .into_active_model()
    .insert(&db_conn)
    .await?;

    Ok(())
}

async fn list_roles(db_conn: DatabaseConnection, username: &str) -> Result<()> {
    let Some(user) = Users::find()
        .filter(users::Column::Username.eq(username))
        .one(&db_conn)
        .await?
    else {
        eprintln!("User \"{username}\" not found!");
        return Ok(());
    };
    let roles = user.find_related(UsersRoles).all(&db_conn).await?;

    println!("User \"{username}\" has the following roles:");
    for role in roles {
        println!("- {:?} (added at: {})", role.role, role.created_at);
    }

    Ok(())
}

async fn remove_role(db_conn: DatabaseConnection, username: &str, role: Role) -> Result<()> {
    let Some(user) = Users::find()
        .filter(users::Column::Username.eq(username))
        .one(&db_conn)
        .await?
    else {
        eprintln!("User \"{username}\" not found!");
        return Ok(());
    };

    UsersRoles::delete_many()
        .filter(
            users_roles::Column::Role
                .eq(DbRole::from(role))
                .and(users_roles::Column::UserId.eq(user.id)),
        )
        .exec(&db_conn)
        .await?;

    Ok(())
}

pub async fn handle(cmd: RoleSubcommand, db_conn: DatabaseConnection) -> Result<()> {
    match cmd {
        RoleSubcommand::Add { username, role } => add_role(db_conn, &username, role).await?,
        RoleSubcommand::List { username } => list_roles(db_conn, &username).await?,
        RoleSubcommand::Remove { username, role } => remove_role(db_conn, &username, role).await?,
    }

    Ok(())
}
