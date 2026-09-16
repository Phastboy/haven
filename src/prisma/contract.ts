import { defineContract } from '@prisma/orm-postgres/contract-builder';

export const contract = defineContract({}, ({ field, model, rel }) => {
  const User = model('User', {
    fields: {
      id: field.id.uuidv7String(),
      email: field.text().unique(),
      username: field.text().optional(),
      name: field.text().optional(),
      createdAt: field.temporal.createdAtString(),
      updatedAt: field.temporal.updatedAtString(),
    },
  });

  const Post = model('Post', {
    fields: {
      id: field.id.uuidv7String(),
      title: field.text(),
      content: field.text().optional(),
      authorId: field.uuidString(),
      createdAt: field.temporal.createdAtString(),
      updatedAt: field.temporal.updatedAtString(),
    },
  });

  const Account = model('Account', {
    fields: {
      id: field.id.uuidv7String(),
      email: field.text().unique(),
      emailVerified: field.boolean().default(false),
      createdAt: field.temporal.createdAtString(),
      updatedAt: field.temporal.updatedAtString(),
    },
  });

  const Session = model('Session', {
    fields: {
      id: field.id.uuidv7String(),
      accountId: field.uuidString(),
      token: field.text().unique(),
      expiresAt: field.temporal.timestampString(),
      userAgent: field.text().optional(),
      ipAddress: field.text().optional(),
      createdAt: field.temporal.createdAtString(),
    },
  });

  const MagicLink = model('MagicLink', {
    fields: {
      id: field.id.uuidv7String(),
      email: field.text(),
      token: field.text().unique(),
      expiresAt: field.temporal.timestampString(),
      usedAt: field.temporal.timestampString().optional(),
      createdAt: field.temporal.createdAtString(),
    },
  });

  const OAuthCredential = model('OAuthCredential', {
    fields: {
      id: field.id.uuidv7String(),
      accountId: field.uuidString(),
      provider: field.text(),
      providerUserId: field.text(),
      accessToken: field.text(),
      refreshToken: field.text().optional(),
      tokenExpiresAt: field.temporal.timestampString().optional(),
      createdAt: field.temporal.createdAtString(),
      updatedAt: field.temporal.updatedAtString(),
    },
  });

  const AccountPlatformLink = model('AccountPlatformLink', {
    fields: {
      id: field.id.uuidv7String(),
      accountId: field.uuidString(),
      platformUserId: field.text(),
      platform: field.text(),
      createdAt: field.temporal.createdAtString(),
    },
  }).sql({
    indexes: [
      { kind: 'index', fields: ['accountId', 'platform'], unique: true, name: 'account_platform_link_unique' },
    ]
  });

  return {
    models: {
      User: User.relations({
        posts: rel.hasMany(Post, { by: 'authorId' }),
      }),
      Post: Post.relations({
        author: rel.belongsTo(User, { from: 'authorId', to: 'id' }),
      }),
      Account: Account.relations({
        sessions: rel.hasMany(Session, { by: 'accountId' }),
        oauthCredentials: rel.hasMany(OAuthCredential, { by: 'accountId' }),
        platformLinks: rel.hasMany(AccountPlatformLink, { by: 'accountId' }),
      }),
      Session: Session.relations({
        account: rel.belongsTo(Account, { from: 'accountId', to: 'id' }),
      }),
      OAuthCredential: OAuthCredential.relations({
        account: rel.belongsTo(Account, { from: 'accountId', to: 'id' }),
      }),
      AccountPlatformLink: AccountPlatformLink.relations({
        account: rel.belongsTo(Account, { from: 'accountId', to: 'id' }),
      }),
      MagicLink,
    },
  };
});
