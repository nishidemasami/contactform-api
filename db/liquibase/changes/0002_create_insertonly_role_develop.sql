--liquibase formatted sql
--changeset copilot:0002 context:develop
CREATE ROLE insertonly WITH LOGIN;
