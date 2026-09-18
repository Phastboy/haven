CREATE TYPE "public"."OfferStatus" AS ENUM('ACTIVE', 'PAUSED', 'ARCHIVED');--> statement-breakpoint
CREATE TYPE "public"."OfferType" AS ENUM('PRODUCT', 'SERVICE', 'APPOINTMENT');--> statement-breakpoint
CREATE TABLE "AccountPlatformLink" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"accountId" varchar(36) NOT NULL,
	"platformUserId" text NOT NULL,
	"platform" text NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "Account" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"email" text NOT NULL,
	"emailVerified" boolean DEFAULT false NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "Account_email_unique" UNIQUE("email")
);
--> statement-breakpoint
CREATE TABLE "MagicLink" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"email" text NOT NULL,
	"token" text NOT NULL,
	"expiresAt" timestamp with time zone NOT NULL,
	"usedAt" timestamp with time zone,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "MagicLink_token_unique" UNIQUE("token")
);
--> statement-breakpoint
CREATE TABLE "OAuthCredential" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"accountId" varchar(36) NOT NULL,
	"provider" text NOT NULL,
	"providerUserId" text NOT NULL,
	"accessToken" text NOT NULL,
	"refreshToken" text,
	"tokenExpiresAt" timestamp with time zone,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "Offer" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"userId" varchar(36) NOT NULL,
	"title" varchar(255) NOT NULL,
	"description" text,
	"price" integer DEFAULT 0 NOT NULL,
	"status" "OfferStatus" DEFAULT 'ACTIVE' NOT NULL,
	"offerType" "OfferType" DEFAULT 'PRODUCT' NOT NULL,
	"images" text[],
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "Session" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"accountId" varchar(36) NOT NULL,
	"token" text NOT NULL,
	"expiresAt" timestamp with time zone NOT NULL,
	"userAgent" text,
	"ipAddress" text,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "Session_token_unique" UNIQUE("token")
);
--> statement-breakpoint
CREATE TABLE "User" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"accountId" varchar(36) NOT NULL,
	"username" text,
	"name" text,
	"bio" text,
	"profilePictureUrl" text,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "User_accountId_unique" UNIQUE("accountId"),
	CONSTRAINT "User_username_unique" UNIQUE("username")
);
--> statement-breakpoint
ALTER TABLE "AccountPlatformLink" ADD CONSTRAINT "AccountPlatformLink_accountId_Account_id_fk" FOREIGN KEY ("accountId") REFERENCES "public"."Account"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "OAuthCredential" ADD CONSTRAINT "OAuthCredential_accountId_Account_id_fk" FOREIGN KEY ("accountId") REFERENCES "public"."Account"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "Offer" ADD CONSTRAINT "Offer_userId_User_id_fk" FOREIGN KEY ("userId") REFERENCES "public"."User"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "Session" ADD CONSTRAINT "Session_accountId_Account_id_fk" FOREIGN KEY ("accountId") REFERENCES "public"."Account"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "User" ADD CONSTRAINT "User_accountId_Account_id_fk" FOREIGN KEY ("accountId") REFERENCES "public"."Account"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "account_platform_link_unique" ON "AccountPlatformLink" USING btree ("accountId","platform");--> statement-breakpoint
CREATE UNIQUE INDEX "OAuthCredential_provider_providerUserId_key" ON "OAuthCredential" USING btree ("provider","providerUserId");