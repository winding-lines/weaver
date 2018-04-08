-- Your SQL goes here
create table commands (
  id INTEGER PRIMARY key,
  kind VARCHAR not null,
  command VARCHAR not null
);

create table locations (
  id integer PRIMARY key,
  location varchar not null
);

create table epics (
  id integer PRIMARY key,
  name varchar not null
);

-- Your SQL goes here
create table actions2 (
  id INTEGER PRIMARY key,
  command_id integer REFERENCES commands(id),
  executed TEXT not null,
  location_id integer references locations(id),
  epic_id integer references epics(id),
  sent boolean,
  annotation text
);



