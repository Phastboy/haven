import { pgTable, varchar, text, boolean, timestamp, uniqueIndex, pgEnum, integer } from 'drizzle-orm/pg-core';
import { relations } from 'drizzle-orm';

export const users = pgTable('User', {
  id: varchar('id', { length: 36 }).primaryKey(),
  accountId: varchar('accountId', { length: 36 }).unique().notNull().references(() => accounts.id, { onDelete: 'cascade' }),
  username: text('username').unique(),
  name: text('name'),
  bio: text('bio'),
  profilePictureUrl: text('profilePictureUrl'),
  createdAt: timestamp('createdAt', { withTimezone: true }).notNull().defaultNow(),
  updatedAt: timestamp('updatedAt', { withTimezone: true }).notNull().defaultNow(),
});

export const offerStatusEnum = pgEnum('OfferStatus', ['ACTIVE', 'PAUSED', 'ARCHIVED']);
export const offerTypeEnum = pgEnum('OfferType', ['PRODUCT', 'SERVICE', 'APPOINTMENT']);

export const offers = pgTable('Offer', {
  id: varchar('id', { length: 36 }).primaryKey(),
  userId: varchar('userId', { length: 36 }).notNull().references(() => users.id, { onDelete: 'cascade' }),
  title: varchar('title', { length: 255 }).notNull(),
  description: text('description'),
  price: integer('price').notNull().default(0), // stored in cents
  status: offerStatusEnum('status').notNull().default('ACTIVE'),
  offerType: offerTypeEnum('offerType').notNull().default('PRODUCT'),
  images: text('images').array(),
  createdAt: timestamp('createdAt', { withTimezone: true }).notNull().defaultNow(),
  updatedAt: timestamp('updatedAt', { withTimezone: true }).notNull().defaultNow(),
});

export const orderStatusEnum = pgEnum('OrderStatus', ['PENDING', 'ACCEPTED', 'REJECTED', 'CANCELLED', 'COMPLETED']);

export const orders = pgTable('Order', {
  id: varchar('id', { length: 36 }).primaryKey(),
  offerId: varchar('offerId', { length: 36 }).notNull().references(() => offers.id, { onDelete: 'cascade' }),
  requesterId: varchar('requesterId', { length: 36 }).notNull().references(() => users.id, { onDelete: 'cascade' }),
  price: integer('price').notNull(), // stored in cents (snapshot of offer price at time of order)
  quantity: integer('quantity').notNull().default(1),
  status: orderStatusEnum('status').notNull().default('PENDING'),
  message: text('message'),
  createdAt: timestamp('createdAt', { withTimezone: true }).notNull().defaultNow(),
  updatedAt: timestamp('updatedAt', { withTimezone: true }).notNull().defaultNow(),
});

export const fulfillmentStatusEnum = pgEnum('FulfillmentStatus', ['PENDING', 'DELIVERED', 'REVISION_REQUESTED', 'COMPLETED']);

export const fulfillments = pgTable('Fulfillment', {
  id: varchar('id', { length: 36 }).primaryKey(),
  orderId: varchar('orderId', { length: 36 }).notNull().unique().references(() => orders.id, { onDelete: 'cascade' }),
  status: fulfillmentStatusEnum('status').notNull().default('PENDING'),
  deliveryMessage: text('deliveryMessage'),
  reviewDeadline: timestamp('reviewDeadline', { withTimezone: true }),
  createdAt: timestamp('createdAt', { withTimezone: true }).notNull().defaultNow(),
  updatedAt: timestamp('updatedAt', { withTimezone: true }).notNull().defaultNow(),
});

export const accounts = pgTable('Account', {
  id: varchar('id', { length: 36 }).primaryKey(),
  email: text('email').unique().notNull(),
  emailVerified: boolean('emailVerified').notNull().default(false),
  createdAt: timestamp('createdAt', { withTimezone: true }).notNull().defaultNow(),
  updatedAt: timestamp('updatedAt', { withTimezone: true }).notNull().defaultNow(),
});

export const sessions = pgTable('Session', {
  id: varchar('id', { length: 36 }).primaryKey(),
  accountId: varchar('accountId', { length: 36 }).notNull().references(() => accounts.id, { onDelete: 'cascade' }),
  token: text('token').unique().notNull(),
  expiresAt: timestamp('expiresAt', { withTimezone: true }).notNull(),
  userAgent: text('userAgent'),
  ipAddress: text('ipAddress'),
  createdAt: timestamp('createdAt', { withTimezone: true }).notNull().defaultNow(),
});

export const magicLinks = pgTable('MagicLink', {
  id: varchar('id', { length: 36 }).primaryKey(),
  email: text('email').notNull(),
  token: text('token').unique().notNull(),
  expiresAt: timestamp('expiresAt', { withTimezone: true }).notNull(),
  usedAt: timestamp('usedAt', { withTimezone: true }),
  createdAt: timestamp('createdAt', { withTimezone: true }).notNull().defaultNow(),
});

export const oauthCredentials = pgTable('OAuthCredential', {
  id: varchar('id', { length: 36 }).primaryKey(),
  accountId: varchar('accountId', { length: 36 }).notNull().references(() => accounts.id, { onDelete: 'cascade' }),
  provider: text('provider').notNull(),
  providerUserId: text('providerUserId').notNull(),
  accessToken: text('accessToken').notNull(),
  refreshToken: text('refreshToken'),
  tokenExpiresAt: timestamp('tokenExpiresAt', { withTimezone: true }),
  createdAt: timestamp('createdAt', { withTimezone: true }).notNull().defaultNow(),
  updatedAt: timestamp('updatedAt', { withTimezone: true }).notNull().defaultNow(),
}, (t) => [uniqueIndex('OAuthCredential_provider_providerUserId_key').on(t.provider, t.providerUserId)]);

export const accountPlatformLinks = pgTable('AccountPlatformLink', {
  id: varchar('id', { length: 36 }).primaryKey(),
  accountId: varchar('accountId', { length: 36 }).notNull().references(() => accounts.id, { onDelete: 'cascade' }),
  platformUserId: text('platformUserId').notNull(),
  platform: text('platform').notNull(),
  createdAt: timestamp('createdAt', { withTimezone: true }).notNull().defaultNow(),
}, (t) => [uniqueIndex('account_platform_link_unique').on(t.accountId, t.platform)]);

// Relations
export const sessionsRelations = relations(sessions, ({ one }) => ({
  account: one(accounts, {
    fields: [sessions.accountId],
    references: [accounts.id],
  }),
}));

export const usersRelations = relations(users, ({ one, many }) => ({
  account: one(accounts, {
    fields: [users.accountId],
    references: [accounts.id],
  }),
  offers: many(offers),
  orders: many(orders),
}));

export const offersRelations = relations(offers, ({ one, many }) => ({
  user: one(users, {
    fields: [offers.userId],
    references: [users.id],
  }),
  orders: many(orders),
}));

export const ordersRelations = relations(orders, ({ one }) => ({
  offer: one(offers, {
    fields: [orders.offerId],
    references: [offers.id],
  }),
  requester: one(users, {
    fields: [orders.requesterId],
    references: [users.id],
  }),
  fulfillment: one(fulfillments, {
    fields: [orders.id],
    references: [fulfillments.orderId],
  }),
}));

export const fulfillmentsRelations = relations(fulfillments, ({ one }) => ({
  order: one(orders, {
    fields: [fulfillments.orderId],
    references: [orders.id],
  }),
}));

export const oauthCredentialsRelations = relations(oauthCredentials, ({ one }) => ({
  account: one(accounts, {
    fields: [oauthCredentials.accountId],
    references: [accounts.id],
  }),
}));

export const accountPlatformLinksRelations = relations(accountPlatformLinks, ({ one }) => ({
  account: one(accounts, {
    fields: [accountPlatformLinks.accountId],
    references: [accounts.id],
  }),
}));

// Inferred Types
export type AccountRecord = typeof accounts.$inferSelect;
export type NewAccount = typeof accounts.$inferInsert;

export type SessionRecord = typeof sessions.$inferSelect;
export type NewSession = typeof sessions.$inferInsert;

export type MagicLinkRecord = typeof magicLinks.$inferSelect;
export type NewMagicLink = typeof magicLinks.$inferInsert;

export type OAuthCredentialRecord = typeof oauthCredentials.$inferSelect;
export type NewOAuthCredential = typeof oauthCredentials.$inferInsert;

export type AccountPlatformLinkRecord = typeof accountPlatformLinks.$inferSelect;
export type NewAccountPlatformLink = typeof accountPlatformLinks.$inferInsert;

export type UserRecord = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;

export type OfferRecord = typeof offers.$inferSelect;
export type NewOffer = typeof offers.$inferInsert;

export type OrderRecord = typeof orders.$inferSelect;
export type NewOrder = typeof orders.$inferInsert;

export type FulfillmentRecord = typeof fulfillments.$inferSelect;
export type NewFulfillment = typeof fulfillments.$inferInsert;
