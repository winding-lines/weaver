-- Your SQL goes here
create table url_restrictions (
  id INTEGER PRIMARY key,
  kind VARCHAR not null,
  url_expr VARCHAR not null
);
