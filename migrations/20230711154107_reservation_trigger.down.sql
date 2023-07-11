-- Add down migration script here
DROP TRIGGER IF EXISTS reservations_trigger ON rsvp.reservations;
DROP FUNCTION IF EXISTS rsvp.reservations_trigger();
DROP TABLE IF EXISTS rsvp.reservation_changes CASCADE;
