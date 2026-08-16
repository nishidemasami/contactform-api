--liquibase formatted sql
--changeset copilot:0001 context:local
CREATE ROLE insertonly WITH LOGIN PASSWORD 'insertonly';
