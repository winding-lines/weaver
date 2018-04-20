-- Your SQL goes here
create table hosts (
  id integer PRIMARY key,
  name varchar not null
);

alter table actions2 add column host_id integer references hosts(id);