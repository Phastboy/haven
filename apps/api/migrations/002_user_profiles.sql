-- Migration: 002_user_profiles
-- Description: Add accountId, bio, profilePictureUrl to User, remove email

-- First, we need to clear out the existing Users since we are adding a NOT NULL constraint
-- for accountId which we don't have data for (since User was disconnected previously).
-- In a real prod environment with data, we would backfill this. Since this is an early stage,
-- truncating the User table is acceptable.
TRUNCATE TABLE "User" CASCADE;

ALTER TABLE "User" DROP COLUMN "email";
ALTER TABLE "User" ADD COLUMN "accountId" VARCHAR(36) NOT NULL;
ALTER TABLE "User" ADD COLUMN "bio" TEXT;
ALTER TABLE "User" ADD COLUMN "profilePictureUrl" TEXT;

-- Add the unique constraint on accountId
ALTER TABLE "User" ADD CONSTRAINT "User_accountId_key" UNIQUE ("accountId");

-- Add foreign key constraint
ALTER TABLE "User" ADD CONSTRAINT "User_accountId_fkey" FOREIGN KEY ("accountId") REFERENCES "Account"("id") ON DELETE CASCADE;
