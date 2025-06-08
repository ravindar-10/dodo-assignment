-- This file should undo anything in `up.sql`;

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS balances;
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS otps;
DROP TABLE IF EXISTS users;

-- Drop the UUID extension
DROP EXTENSION IF EXISTS "uuid-ossp";
