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
                    .col(big_unsigned(ShortLink::Views).default(0))
                    .col(timestamp(ShortLink::CreatedAt))
                    .col(timestamp(ShortLink::UpdatedAt))
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
                    .name(ShortLinkIdx::OriginalUrl)
                    .col(ShortLink::OriginalUrl)
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
                    .name(ShortLinkIdx::OriginalUrl)
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
            .await
    }
}

#[derive(DeriveIden)]
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

impl ToString for UserPassFk {
    fn to_string(&self) -> String {
        match self {
            Self::UserId => "fk_user_id".to_owned(),
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

impl ToString for SessionsIdx {
    fn to_string(&self) -> String {
        match self {
            Self::SessionId => "idx_session_id".to_owned(),
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

impl ToString for SessionsFk {
    fn to_string(&self) -> String {
        match self {
            Self::UserId => "fk_user_id".to_owned(),
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
    Views,
}

enum ShortLinkIdx {
    Url,
    OriginalUrl,
    ShortUrl,
    ExpiryDate,
}

impl ToString for ShortLinkIdx {
    fn to_string(&self) -> String {
        match self {
            Self::Url => "idx_url".to_owned(),
            Self::OriginalUrl => "idx_original_url".to_owned(),
            Self::ShortUrl => "idx_short_url".to_owned(),
            Self::ExpiryDate => "idx_expiry_date".to_owned(),
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

impl ToString for ShortLinkFk {
    fn to_string(&self) -> String {
        match self {
            Self::UserId => "fk_user_id".to_owned(),
        }
    }
}

impl From<ShortLinkFk> for String {
    fn from(fk: ShortLinkFk) -> Self {
        fk.to_string()
    }
}
