--liquibase formatted sql
--changeset copilot:0004 context:local,develop
CREATE TABLE public.inquiries (
  id uuid NOT NULL,
  name text NOT NULL DEFAULT '',
  email text NOT NULL DEFAULT '',
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  body text NOT NULL DEFAULT '',
  row_log text NOT NULL,
  CONSTRAINT pk_inquiries PRIMARY KEY (id)
);
