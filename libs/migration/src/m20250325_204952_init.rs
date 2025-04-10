use std::fmt::{Display, Formatter};

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ShortLink::Table)
                    .if_not_exists()
                    .col(pk_auto(ShortLink::Id))
                    .col(string(ShortLink::Url).unique_key())
                    .col(string(ShortLink::ShortUrl).unique_key())
                    .col(string(ShortLink::OriginalUrl))
                    .col(uuid_null(ShortLink::UserId))
                    .col(timestamp_null(ShortLink::ExpiryDate))
                    .col(timestamp(ShortLink::CreatedAt))
                    .col(timestamp(ShortLink::UpdatedAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Views::Table)
                    .if_not_exists()
                    .col(pk_auto(Views::Id))
                    .col(integer(Views::ShortLink).not_null())
                    .col(integer(Views::NumViews).not_null().default(0))
                    .col(json_binary_null(Views::Headers))
                    .col(string_null(Views::IpAddress))
                    .col(integer_null(Views::IpLocation))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Location::Table)
                    .if_not_exists()
                    .col(pk_auto(Location::Id))
                    .col(double_null(Location::Latitude))
                    .col(double_null(Location::Longitude))
                    .col(integer_null(Location::MetroCode))
                    .col(string_null(Location::TimeCode))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_uuid(User::UserId))
                    .col(string(User::Name))
                    .col(string(User::Email).unique_key())
                    .col(timestamp(User::CreatedAt))
                    .col(timestamp(User::UpdatedAt))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(UserPass::Table)
                    .if_not_exists()
                    .col(pk_auto(UserPass::Id))
                    .col(uuid(UserPass::UserId))
                    .col(string(UserPass::Password))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(pk_auto(Sessions::Id))
                    .col(string(Sessions::SessionId))
                    .col(uuid(Sessions::UserId))
                    .col(timestamp(Sessions::Expiry))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(Views::Table)
                    .name(ViewsIdx::ShortLink)
                    .col(Views::ShortLink)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(ShortLink::Table)
                    .name(ShortLinkIdx::ShortUrl)
                    .col(ShortLink::ShortUrl)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(ShortLink::Table)
                    .name(ShortLinkIdx::Url)
                    .col(ShortLink::Url)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(ShortLink::Table)
                    .name(ShortLinkIdx::ExpiryDate)
                    .col(ShortLink::ExpiryDate)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(Sessions::Table)
                    .name(SessionsIdx::SessionId)
                    .col(Sessions::SessionId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(ShortLinkFk::UserId)
                    .from(ShortLink::Table, ShortLink::UserId)
                    .to(User::Table, User::UserId)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(UserPassFk::UserId)
                    .from(UserPass::Table, UserPass::UserId)
                    .to(User::Table, User::UserId)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(SessionsFk::UserId)
                    .from(Sessions::Table, Sessions::UserId)
                    .to(User::Table, User::UserId)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(ViewsFk::ShortLink)
                    .from(Views::Table, Views::ShortLink)
                    .to(ShortLink::Table, ShortLink::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(ViewsFk::IpLocation)
                    .from(Views::Table, Views::IpLocation)
                    .to(Location::Table, Location::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(ShortLink::Table)
                    .name(ShortLinkIdx::Url)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(ShortLink::Table)
                    .name(ShortLinkIdx::ShortUrl)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(ShortLink::Table)
                    .name(ShortLinkIdx::ExpiryDate)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(Sessions::Table)
                    .name(SessionsIdx::SessionId)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .table(Views::Table)
                    .name(ViewsIdx::ShortLink)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Views::Table)
                    .name(ViewsFk::IpLocation)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Views::Table)
                    .name(ViewsFk::ShortLink)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(ShortLink::Table)
                    .name(ShortLinkFk::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(UserPass::Table)
                    .name(UserPassFk::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Sessions::Table)
                    .name(SessionsFk::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ShortLink::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserPass::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Location::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Views::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
enum User {
    Table,
    UserId,
    Name,
    Email,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserPass {
    Table,
    Id,
    UserId,
    Password,
}

enum UserPassFk {
    UserId,
}

impl Display for UserPassFk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserId => write!(f, "fk_user_id"),
        }
    }
}

impl From<UserPassFk> for String {
    fn from(fk: UserPassFk) -> Self {
        fk.to_string()
    }
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    Id,
    SessionId,
    UserId,
    Expiry,
}

enum SessionsIdx {
    SessionId,
}

impl Display for SessionsIdx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionId => write!(f, "idx_session_id"),
        }
    }
}

impl From<SessionsIdx> for String {
    fn from(idx: SessionsIdx) -> Self {
        idx.to_string()
    }
}

enum SessionsFk {
    UserId,
}

impl Display for SessionsFk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserId => write!(f, "fk_user_id"),
        }
    }
}

impl From<SessionsFk> for String {
    fn from(fk: SessionsFk) -> Self {
        fk.to_string()
    }
}

#[derive(DeriveIden)]
enum ShortLink {
    Table,
    Id,
    Url,
    ShortUrl,
    OriginalUrl,
    UserId,
    ExpiryDate,
    CreatedAt,
    UpdatedAt,
}

enum ShortLinkIdx {
    Url,
    ShortUrl,
    ExpiryDate,
}

impl Display for ShortLinkIdx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url => write!(f, "idx_url"),
            Self::ShortUrl => write!(f, "idx_short_url"),
            Self::ExpiryDate => write!(f, "idx_expiry_date"),
        }
    }
}

impl From<ShortLinkIdx> for String {
    fn from(idx: ShortLinkIdx) -> Self {
        idx.to_string()
    }
}

enum ShortLinkFk {
    UserId,
}

impl Display for ShortLinkFk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserId => write!(f, "fk_user_id"),
        }
    }
}

impl From<ShortLinkFk> for String {
    fn from(fk: ShortLinkFk) -> Self {
        fk.to_string()
    }
}

#[derive(DeriveIden)]
enum Views {
    Table,
    Id,
    ShortLink,
    NumViews,
    Headers,
    IpAddress,
    IpLocation,
}

enum ViewsFk {
    ShortLink,
    IpLocation,
}

impl Display for ViewsFk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShortLink => write!(f, "fk_short_link"),
            Self::IpLocation => write!(f, "fk_ip_location"),
        }
    }
}

impl From<ViewsFk> for String {
    fn from(vi: ViewsFk) -> Self {
        vi.to_string()
    }
}

enum ViewsIdx {
    ShortLink,
}

impl Display for ViewsIdx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShortLink => write!(f, "idx_short_link"),
        }
    }
}

impl From<ViewsIdx> for String {
    fn from(vi: ViewsIdx) -> Self {
        vi.to_string()
    }
}

#[derive(DeriveIden)]
enum Location {
    Table,
    Id,
    Latitude,
    Longitude,
    MetroCode,
    TimeCode,
}
