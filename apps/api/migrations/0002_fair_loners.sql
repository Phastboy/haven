CREATE TYPE "public"."FulfillmentStatus" AS ENUM('PENDING', 'DELIVERED', 'REVISION_REQUESTED', 'COMPLETED');--> statement-breakpoint
CREATE TABLE "Fulfillment" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"orderId" varchar(36) NOT NULL,
	"status" "FulfillmentStatus" DEFAULT 'PENDING' NOT NULL,
	"deliveryMessage" text,
	"reviewDeadline" timestamp with time zone,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "Fulfillment_orderId_unique" UNIQUE("orderId")
);
--> statement-breakpoint
ALTER TABLE "Fulfillment" ADD CONSTRAINT "Fulfillment_orderId_Order_id_fk" FOREIGN KEY ("orderId") REFERENCES "public"."Order"("id") ON DELETE cascade ON UPDATE no action;