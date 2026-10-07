-- Automatically create haven_rust database for local development on initial volume creation
SELECT 'CREATE DATABASE haven_rust'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'haven_rust')\gexec
