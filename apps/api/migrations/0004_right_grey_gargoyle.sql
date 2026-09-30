ALTER TABLE "Offer" ADD COLUMN "currency" varchar(3) DEFAULT 'NGN' NOT NULL;--> statement-breakpoint
ALTER TABLE "Order" ADD COLUMN "currency" varchar(3) DEFAULT 'NGN' NOT NULL;--> statement-breakpoint
CREATE UNIQUE INDEX "Thread_participant_pair_key" ON "Thread" USING btree ("participant1Id","participant2Id");