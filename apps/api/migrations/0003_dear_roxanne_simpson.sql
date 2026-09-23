CREATE TABLE "Message" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"threadId" varchar(36) NOT NULL,
	"senderId" varchar(36) NOT NULL,
	"content" text NOT NULL,
	"readAt" timestamp with time zone,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "Thread" (
	"id" varchar(36) PRIMARY KEY NOT NULL,
	"participant1Id" varchar(36) NOT NULL,
	"participant2Id" varchar(36) NOT NULL,
	"createdAt" timestamp with time zone DEFAULT now() NOT NULL,
	"updatedAt" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "Message" ADD CONSTRAINT "Message_threadId_Thread_id_fk" FOREIGN KEY ("threadId") REFERENCES "public"."Thread"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "Message" ADD CONSTRAINT "Message_senderId_User_id_fk" FOREIGN KEY ("senderId") REFERENCES "public"."User"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "Thread" ADD CONSTRAINT "Thread_participant1Id_User_id_fk" FOREIGN KEY ("participant1Id") REFERENCES "public"."User"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "Thread" ADD CONSTRAINT "Thread_participant2Id_User_id_fk" FOREIGN KEY ("participant2Id") REFERENCES "public"."User"("id") ON DELETE cascade ON UPDATE no action;