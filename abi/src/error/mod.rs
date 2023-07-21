mod conflict;

// use sqlx::postgres::PgDatabaseError;
// use sqlx_core::error::DatabaseError; // 使用 sqlx_core 中的 trait
use thiserror::Error;

pub use conflict::{ReservationConflict, ReservationConflictInfo, ReservationWindow};

#[derive(Error, Debug)]
pub enum Error {
    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader { expected: String, found: String },
    #[error("Database error")]
    DbError(sqlx::Error),

    #[error("Database error1")]
    DbError1(sqlx_core::error::Error),

    #[error("Invalid start or end time for the reservation")]
    InvalidTime,

    #[error("Conflict reservation")]
    ConflictReservation(ReservationConflictInfo),

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("unknown error")]
    Unknown,

    #[error("No reservation found by the given condition")]
    NotFound,
}

// impl From<sqlx::Error> for Error {
//     fn from(e: sqlx::Error) -> Self {
//         match e {
//             sqlx::Error::Database(e) => {
//                 let err: &PgDatabaseError = e.downcast_ref();
//                 match (err.code(), err.schema(), err.table()) {
//                     ("23P01", Some("rsvp"), Some("reservations")) => {
//                         Error::ConflictReservation(err.detail().unwrap().parse().unwrap())
//                     }
//                     _ => Error::DbError(sqlx::Error::Database(e)),
//                 }
//             }
//             _ => Error::DbError(e),
//         }
//     }
// }

impl From<sqlx_core::error::Error> for Error {
    fn from(e: sqlx_core::error::Error) -> Self {
        match e {
            // 使用正确的类型：sqlx_core::postgres::PgDatabaseError
            sqlx_core::error::Error::Database(e) => {
                let err: &sqlx_core::postgres::PgDatabaseError = e.downcast_ref();

                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Error::DbError1(sqlx_core::error::Error::Database(e)),
                }
            }
            sqlx_core::error::Error::RowNotFound => Error::NotFound,
            _ => Error::DbError1(e),
        }
    }
}
