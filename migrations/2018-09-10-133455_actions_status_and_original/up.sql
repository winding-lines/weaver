-- Your SQL goes here
alter table actions2 add column status integer;

create table pages (
  id INTEGER PRIMARY key,
  normalized_url VARCHAR not null,
  title varchar
);

CREATE INDEX IF NOT EXISTS AccessedUrlsNormalizedName ON pages(normalized_url);

alter table commands add column page_id integer references pages(id);