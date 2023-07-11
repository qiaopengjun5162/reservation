-- Add down migration script here
DROP EXTENSION btree_gist;
DROP SCHEMA IF EXISTS rsvp CASCADE;
