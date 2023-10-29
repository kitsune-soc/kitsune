use crate::Result;
use clap::{Args, Subcommand, ValueEnum};
use diesel::{BelongingToDsl, BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use kitsune_db::{
    function::lower,
    model::{
        user::User,
        user_role::{NewUserRole, Role as DbRole, UserRole},
    },
};
use speedy_uuid::Uuid;

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

async fn add_role(db_conn: &mut AsyncPgConnection, username_str: &str, role: Role) -> Result<()> {
    use kitsune_db::schema::{
        users::dsl::{username, users},
        users_roles,
    };

    let user = users
        .filter(username.eq(username_str))
        .first::<User>(db_conn)
        .await?;

    let new_role = NewUserRole {
        id: Uuid::now_v7(),
        user_id: user.id,
        role: role.into(),
    };

    diesel::insert_into(users_roles::table)
        .values(&new_role)
        .execute(db_conn)
        .await?;

    Ok(())
}

async fn list_roles(db_conn: &mut AsyncPgConnection, username_str: &str) -> Result<()> {
    use kitsune_db::schema::users;

    let user: User = users::table
        .filter(lower(users::username).eq(lower(username_str)))
        .first(db_conn)
        .await?;

    let roles = UserRole::belonging_to(&user)
        .load::<UserRole>(db_conn)
        .await?;

    println!("User \"{username_str}\" has the following roles:");
    for role in roles {
        println!("- {:?} (added at: {})", role.role, role.created_at);
    }

    Ok(())
}

async fn remove_role(
    db_conn: &mut AsyncPgConnection,
    username_str: &str,
    role: Role,
) -> Result<()> {
    use kitsune_db::schema::{users, users_roles};

    let user = users::table
        .filter(lower(users::username).eq(lower(username_str)))
        .first::<User>(db_conn)
        .await?;

    diesel::delete(
        users_roles::table.filter(
            users_roles::role
                .eq(DbRole::from(role))
                .and(users_roles::user_id.eq(user.id)),
        ),
    )
    .execute(db_conn)
    .await?;

    Ok(())
}

pub async fn handle(cmd: RoleSubcommand, db_conn: &mut AsyncPgConnection) -> Result<()> {
    match cmd {
        RoleSubcommand::Add { username, role } => add_role(db_conn, &username, role).await?,
        RoleSubcommand::List { username } => list_roles(db_conn, &username).await?,
        RoleSubcommand::Remove { username, role } => remove_role(db_conn, &username, role).await?,
    }

    Ok(())
}
