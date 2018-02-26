-- Your SQL goes here
create table actions (
  id INTEGER PRIMARY key,
  executed TEXT not null,
  kind VARCHAR not null,
  command VARCHAR not null,
  location VARCHAR
);