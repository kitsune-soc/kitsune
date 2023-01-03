use chrono::Utc;
use clap::{Args, Parser, Subcommand};
use kitsune::{
    config::Configuration,
    db::{
        self,
        model::{
            job,
            role::{self, Role},
            user,
        },
    },
    job::JobState,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter,
};
use std::error::Error;
use uuid::Uuid;

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;

#[derive(Args)]
struct AddRemoveRoleArgs {}

#[derive(Subcommand)]
enum RoleSubcommand {
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

#[derive(Subcommand)]
enum AppSubcommand {
    /// Clear succeeded jobs from database
    ///
    /// Succeeded jobs are kept in the database so administrators can aggregate some nice statistics.  
    /// However, they can fill up your database and aren't essential to anything.
    ClearSucceededJobs,

    /// Manage roles for local users
    #[clap(subcommand)]
    Role(RoleSubcommand),
}

/// CLI for the Kitsune social media server
#[derive(Parser)]
#[command(about, author, version)]
struct App {
    #[clap(subcommand)]
    subcommand: AppSubcommand,
}

async fn add_role(db_conn: DatabaseConnection, username: &str, role: Role) -> Result<()> {
    let Some(user) = user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .one(&db_conn)
        .await?
    else {
        eprintln!("User \"{username}\" not found!");
        return Ok(());
    };

    role::Model {
        id: Uuid::now_v7(),
        user_id: user.id,
        role,
        created_at: Utc::now(),
    }
    .into_active_model()
    .insert(&db_conn)
    .await?;

    Ok(())
}

async fn list_roles(db_conn: DatabaseConnection, username: &str) -> Result<()> {
    let Some(user) = user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .one(&db_conn)
        .await?
    else {
        eprintln!("User \"{username}\" not found!");
        return Ok(());
    };
    let roles = user.find_related(role::Entity).all(&db_conn).await?;

    println!("User \"{username}\" has the following roles:");
    for role in roles {
        println!("- {:?} (added at: {})", role.role, role.created_at);
    }

    Ok(())
}

async fn remove_role(db_conn: DatabaseConnection, username: &str, role: Role) -> Result<()> {
    let Some(user) = user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .one(&db_conn)
        .await?
    else {
        eprintln!("User \"{username}\" not found!");
        return Ok(());
    };

    role::Entity::delete_many()
        .filter(
            role::Column::Role
                .eq(role)
                .and(role::Column::UserId.eq(user.id)),
        )
        .exec(&db_conn)
        .await?;

    Ok(())
}

async fn clear_completed_jobs(db_conn: DatabaseConnection) -> Result<()> {
    let delete_result = job::Entity::delete_many()
        .filter(job::Column::State.eq(JobState::Succeeded))
        .exec(&db_conn)
        .await?;

    println!(
        "Deleted {} succeeded jobs from the database",
        delete_result.rows_affected
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config: Configuration = envy::from_env()?;
    let db_conn = db::connect(&config.database_url).await?;
    let cmd = App::parse();

    match cmd.subcommand {
        AppSubcommand::ClearSucceededJobs => clear_completed_jobs(db_conn).await?,
        AppSubcommand::Role(RoleSubcommand::Add { username, role }) => {
            add_role(db_conn, &username, role).await?
        }
        AppSubcommand::Role(RoleSubcommand::List { username }) => {
            list_roles(db_conn, &username).await?
        }
        AppSubcommand::Role(RoleSubcommand::Remove { username, role }) => {
            remove_role(db_conn, &username, role).await?
        }
    }

    Ok(())
}
