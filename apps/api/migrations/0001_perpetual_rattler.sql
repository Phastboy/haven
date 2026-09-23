CREATE TYPE "public"."OrderStatus" AS ENUM('PENDING', 'ACCEPTED', 'REJECTED', 'CANCELLED', 'COMPLETED');--> statement-breakpoint
CREATE TABLE "Order" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"offerId" varchar(36) NOT NULL,
	"requesterId" varchar(36) NOT NULL,
	"price" integer NOT NULL,
	"quantity" integer DEFAULT 1 NOT NULL,
	"status" "OrderStatus" DEFAULT 'PENDING' NOT NULL,
	"message" text,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "Order" ADD CONSTRAINT "Order_offerId_Offer_id_fk" FOREIGN KEY ("offerId") REFERENCES "public"."Offer"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "Order" ADD CONSTRAINT "Order_requesterId_User_id_fk" FOREIGN KEY ("requesterId") REFERENCES "public"."User"("id") ON DELETE cascade ON UPDATE no action;