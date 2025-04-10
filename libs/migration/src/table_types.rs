use std::fmt::{Display, Formatter};

use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum User {
    Table,
    UserId,
    Name,
    Email,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(crate) enum UserPass {
    Table,
    Id,
    UserId,
    Password,
}

pub(crate) enum UserPassFk {
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
pub(crate) enum Sessions {
    Table,
    Id,
    SessionId,
    UserId,
    Expiry,
}

pub(crate) enum SessionsIdx {
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

pub(crate) enum SessionsFk {
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
pub(crate) enum ShortLink {
    Table,
    Id,
    ShortUrl,
    OriginalUrl,
    UserId,
    ExpiryDate,
    CreatedAt,
    UpdatedAt,
}

pub(crate) enum ShortLinkIdx {
    ShortUrl,
    ExpiryDate,
}

impl Display for ShortLinkIdx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
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

pub(crate) enum ShortLinkFk {
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
pub(crate) enum Views {
    Table,
    Id,
    ShortLink,
    Headers,
    Ip,
    CacheHit,
    CreatedAt,
}

pub(crate) enum ViewsFk {
    ShortLink,
}

impl Display for ViewsFk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShortLink => write!(f, "fk_short_link"),
        }
    }
}

impl From<ViewsFk> for String {
    fn from(vi: ViewsFk) -> Self {
        vi.to_string()
    }
}
