-- Add up migration script here
CREATE SCHEMA rsvp;
-- AWS RDS support this: https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/CHAP_PostgreSQL.html
CREATE EXTENSION btree_gist;
-- TODO: consider to create a role for the application
