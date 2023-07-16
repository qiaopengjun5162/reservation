use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader { expected: String, found: String },
    #[error("Database error: {0}")]
    DbError(sqlx::Error),

    #[error("Invalid start or end time for the reservation")]
    InvalidTime,

    #[error("{0}")]
    ConflictReservation(String),

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("unknown error")]
    Unknown,

    #[error("No reservation found by the given condition")]
    NotFound,
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictReservation(err.detail().unwrap().to_string())
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            _ => Error::DbError(e),
        }
    }
}

// TODO: write a parser
// ("Key (resource_id, timespan)=(ocean-view-room-713, [\"2023-08-16 22:00:00+00\",\"2023-08-30 22:00:00+00\"))
// conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2023-08-15 22:00:00+00\",\"2023-08-25 22:00:00+00\"))."

// pub struct ReservationConflictInfo {
//     a: ReservationWindow,
//     b: ReservationWindow,
// }

// pub struct ReservationWindow {
//     rid: String,
//     start: DateTime<Utc>,
//     end: DateTime<Utc>,
// }
