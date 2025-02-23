use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Integer,
};

use crate::models::{ApiToken, User, Version};
use crate::schema::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[repr(i32)]
#[diesel(sql_type = Integer)]
pub enum VersionAction {
    Publish = 0,
    Yank = 1,
    Unyank = 2,
}

impl From<VersionAction> for &'static str {
    fn from(action: VersionAction) -> Self {
        match action {
            VersionAction::Publish => "publish",
            VersionAction::Yank => "yank",
            VersionAction::Unyank => "unyank",
        }
    }
}

impl From<VersionAction> for String {
    fn from(action: VersionAction) -> Self {
        let string: &'static str = action.into();

        string.into()
    }
}

impl FromSql<Integer, Pg> for VersionAction {
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> deserialize::Result<Self> {
        match <i32 as FromSql<Integer, Pg>>::from_sql(bytes)? {
            0 => Ok(VersionAction::Publish),
            1 => Ok(VersionAction::Yank),
            2 => Ok(VersionAction::Unyank),
            n => Err(format!("unknown version action: {n}").into()),
        }
    }
}

impl ToSql<Integer, Pg> for VersionAction {
    fn to_sql(&self, out: &mut Output<'_, '_, Pg>) -> serialize::Result {
        ToSql::<Integer, Pg>::to_sql(&(*self as i32), &mut out.reborrow())
    }
}

#[derive(Debug, Clone, Copy, Queryable, Identifiable, Associations)]
#[diesel(
    table_name = version_owner_actions,
    check_for_backend(diesel::pg::Pg),
    belongs_to(Version),
    belongs_to(User, foreign_key = user_id),
    belongs_to(ApiToken, foreign_key = api_token_id),
)]
pub struct VersionOwnerAction {
    pub id: i32,
    pub version_id: i32,
    pub user_id: i32,
    pub api_token_id: Option<i32>,
    pub action: VersionAction,
    pub time: NaiveDateTime,
}

impl VersionOwnerAction {
    pub fn all(conn: &mut PgConnection) -> QueryResult<Vec<Self>> {
        version_owner_actions::table.load(conn)
    }

    pub fn by_version(
        conn: &mut PgConnection,
        version: &Version,
    ) -> QueryResult<Vec<(Self, User)>> {
        use version_owner_actions::dsl::version_id;

        version_owner_actions::table
            .filter(version_id.eq(version.id))
            .inner_join(users::table)
            .order(version_owner_actions::dsl::id)
            .load(conn)
    }

    pub fn for_versions(
        conn: &mut PgConnection,
        versions: &[Version],
    ) -> QueryResult<Vec<Vec<(Self, User)>>> {
        Ok(Self::belonging_to(versions)
            .inner_join(users::table)
            .order(version_owner_actions::dsl::id)
            .load(conn)?
            .grouped_by(versions))
    }
}

pub fn insert_version_owner_action(
    conn: &mut PgConnection,
    version_id_: i32,
    user_id_: i32,
    api_token_id_: Option<i32>,
    action_: VersionAction,
) -> QueryResult<VersionOwnerAction> {
    use version_owner_actions::dsl::{action, api_token_id, user_id, version_id};

    diesel::insert_into(version_owner_actions::table)
        .values((
            version_id.eq(version_id_),
            user_id.eq(user_id_),
            api_token_id.eq(api_token_id_),
            action.eq(action_),
        ))
        .get_result(conn)
}
