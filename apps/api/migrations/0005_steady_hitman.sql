CREATE TABLE "MessageContext" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"messageId" varchar(36) NOT NULL,
	"offerId" varchar(36),
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "MessageContext" ADD CONSTRAINT "MessageContext_messageId_Message_id_fk" FOREIGN KEY ("messageId") REFERENCES "public"."Message"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "MessageContext" ADD CONSTRAINT "MessageContext_offerId_Offer_id_fk" FOREIGN KEY ("offerId") REFERENCES "public"."Offer"("id") ON DELETE set null ON UPDATE no action;