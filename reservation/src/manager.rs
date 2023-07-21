use crate::{ReservationId, ReservationManager, Rsvp};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, PgPool, Row};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        // if rsvp.start.is_none() || rsvp.end.is_none() {
        //     return Err(abi::Error::InvalidTime);
        // }
        rsvp.validate()?;

        let status = abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);

        // let start = abi::convert_to_utc_time(rsvp.start.as_ref().unwrap().clone());
        // let end = abi::convert_to_utc_time(rsvp.end.as_ref().unwrap().clone());

        // let timespan: PgRange<DateTime<Utc>> = (start..end).into();
        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();
        // generate a insert sql for the reservation
        // execute sql
        let id: Uuid = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id"
        )
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?
        .get(0);

        rsvp.id = id.to_string();

        Ok(rsvp)
    }

    async fn change_status(&self, _id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }
    /// delete reservation
    async fn delete(&self, _id: ReservationId) -> Result<(), abi::Error> {
        todo!()
    }
    /// get reservation by id
    async fn get(&self, _id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }
    /// query reservation
    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        todo!()
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {

    use abi::ReservationConflictInfo;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        // let start: DateTime<FixedOffset> = "2023-08-15T15:00:00-0700".parse().unwrap();
        // let end: DateTime<FixedOffset> = "2023-08-25T15:00:00-0700".parse().unwrap();
        let rsvp = abi::Reservation::new_pending(
            "tyrid",
            "ocean-view-room-713",
            "2023-08-15T15:00:00-0700".parse().unwrap(),
            "2023-08-25T15:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm. Please help to upgrade to executive room if possible.",
        );
        // let rsvp = abi::Reservation {
        //     id: "".to_string(),
        //     user_id: "tyrid".to_string(),
        //     resource_id: "ocean-view-room-713".to_string(),
        //     start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
        //     end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
        //     note: "I'll arrive at 3pm. Please help to upgrade to executive room if possible."
        //         .to_string(),
        //     status: abi::ReservationStatus::Pending as i32,
        // };
        let rsvp = manager.reserve(rsvp).await.unwrap();
        // assert!(rsvp.id != "");
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_conflict_reservation_should_reject() {
        let manager = ReservationManager::new(migrated_pool.clone());

        let rsvp1 = abi::Reservation::new_pending(
            "tyrid",
            "ocean-view-room-713",
            "2023-07-25T15:00:00-0700".parse().unwrap(),
            "2023-07-28T12:00:00-0700".parse().unwrap(),
            "hello.",
        );

        let rsvp2 = abi::Reservation::new_pending(
            "aliceid",
            "ocean-view-room-713",
            "2023-07-26T15:00:00-0700".parse().unwrap(),
            "2023-07-30T12:00:00-0700".parse().unwrap(),
            "hello.",
        );

        let _rsvp1 = manager.reserve(rsvp1).await.unwrap();
        let err = manager.reserve(rsvp2).await.unwrap_err();
        println!("{:?}", err);
        if let abi::Error::ConflictReservation(ReservationConflictInfo::Parsed(info)) = err {
            assert_eq!(info.old.rid, "ocean-view-room-713");
            assert_eq!(info.old.start.to_rfc3339(), "2023-07-25T22:00:00+00:00");
            assert_eq!(info.old.end.to_rfc3339(), "2023-07-28T19:00:00+00:00");
            assert_eq!(info.new.rid, "ocean-view-room-713");
            assert_eq!(info.new.start.to_rfc3339(), "2023-07-26T22:00:00+00:00");
            assert_eq!(info.new.end.to_rfc3339(), "2023-07-30T19:00:00+00:00");
        } else {
            panic!("expect conflict reservation error");
        }
    }

    #[test]
    fn reserve_conflict_reservation_should_reject_test() {
        const ERR_MSG: &str = "Key (resource_id, timespan)=(ocean-view-room-713, [\"2023-07-26 22:00:00+00\",\"2023-07-30 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2023-07-25 22:00:00+00\",\"2023-07-28 19:00:00+00\")).";

        let info = ERR_MSG.parse().unwrap();
        match info {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.old.rid, "ocean-view-room-713");
                assert_eq!(conflict.old.start.to_rfc3339(), "2023-07-25T22:00:00+00:00");
                assert_eq!(conflict.old.end.to_rfc3339(), "2023-07-28T19:00:00+00:00");
                assert_eq!(conflict.new.start.to_rfc3339(), "2023-07-26T22:00:00+00:00");
                assert_eq!(conflict.new.end.to_rfc3339(), "2023-07-30T19:00:00+00:00");
            }
            ReservationConflictInfo::Unparsed(_) => panic!("测试测试panic了哦弄个"),
        }
    }
}
