import * as fs from "fs";
import * as path from "path";

const SRC = path.join(__dirname, "../src");

function replaceInFile(filepath: string, replacements: [RegExp | string, string][]) {
  const fullPath = path.join(SRC, filepath);
  if (!fs.existsSync(fullPath)) return;
  let content = fs.readFileSync(fullPath, "utf8");
  for (const [search, replace] of replacements) {
    content = content.replace(search, replace);
  }
  fs.writeFileSync(fullPath, content);
}

// 1. src/user/presentation/__tests__/user.e2e.test.ts
replaceInFile("user/presentation/__tests__/user.e2e.test.ts", [
  [/as any\[\];/g, "as { id: string }[];"],
  [
    /as any;/g,
    "as { id?: string; username?: string; name?: string; profilePictureUrl?: string; [key: string]: unknown };",
  ],
]);

// 2. src/user/application/__tests__/user-use-cases.test.ts
replaceInFile("user/application/__tests__/user-use-cases.test.ts", [
  [
    /mockUserRepo as any/g,
    'mockUserRepo as unknown as import("../../domain/user.repository").IUserRepository',
  ],
]);

// 3. src/auth/presentation/__tests__/google-oauth.e2e.test.ts
replaceInFile("auth/presentation/__tests__/google-oauth.e2e.test.ts", [
  [/as any;/g, "as { token?: string; [key: string]: unknown };"],
]);

// 4. src/auth/presentation/__tests__/session.e2e.test.ts
replaceInFile("auth/presentation/__tests__/session.e2e.test.ts", [
  [/as any;/g, "as { token?: string; [key: string]: unknown };"],
]);

// 5. src/auth/presentation/__tests__/magic-link.e2e.test.ts
replaceInFile("auth/presentation/__tests__/magic-link.e2e.test.ts", [
  [/as any;/g, "as { token?: string; [key: string]: unknown };"],
]);

// 6. src/auth/application/__tests__/logout.test.ts
replaceInFile("auth/application/__tests__/logout.test.ts", [
  [
    /mockSessionRepo as any/g,
    'mockSessionRepo as unknown as import("../../domain/repositories.interface").ISessionRepository',
  ],
]);

// 7. src/auth/application/__tests__/get-session.use-case.test.ts
replaceInFile("auth/application/__tests__/get-session.use-case.test.ts", [
  [
    /mockRepo as any, mockTokenService as any/g,
    'mockRepo as unknown as import("../../domain/repositories.interface").ISessionRepository, mockTokenService as unknown as import("../../domain/services.interface").ITokenService',
  ],
  [/\} as any;/g, '} as unknown as import("../../domain/session.schema").SessionWithAccount;'],
  [
    /\} as any as ISessionRepository;/g,
    '} as unknown as import("../../domain/repositories.interface").ISessionRepository;',
  ],
]);

// 8. src/auth/application/__tests__/get-session.test.ts
replaceInFile("auth/application/__tests__/get-session.test.ts", [
  [
    /mockSessionRepo as any/g,
    'mockSessionRepo as unknown as import("../../domain/repositories.interface").ISessionRepository',
  ],
]);

// 9. src/auth/application/__tests__/google-oauth.test.ts
replaceInFile("auth/application/__tests__/google-oauth.test.ts", [
  [
    /mockAccountRepo as any/g,
    'mockAccountRepo as unknown as import("../../domain/repositories.interface").IAccountRepository',
  ],
  [
    /mockOauthRepo as any/g,
    'mockOauthRepo as unknown as import("../../domain/repositories.interface").IOAuthAccountRepository',
  ],
  [
    /mockSessionRepo as any/g,
    'mockSessionRepo as unknown as import("../../domain/repositories.interface").ISessionRepository',
  ],
  [
    /mockGoogleService as any/g,
    'mockGoogleService as unknown as import("../../domain/services.interface").IGoogleOAuthService',
  ],
  [
    /mockProfileCreator as any/g,
    'mockProfileCreator as unknown as import("../../application/use-cases/create-profile.use-case").CreateProfileUseCase',
  ],
]);

// 10. src/auth/application/__tests__/request-magic-link.test.ts
replaceInFile("auth/application/__tests__/request-magic-link.test.ts", [
  [
    /mockMagicLinkRepo as any/g,
    'mockMagicLinkRepo as unknown as import("../../domain/repositories.interface").IMagicLinkRepository',
  ],
  [
    /mockEmailService as any/g,
    'mockEmailService as unknown as import("../../domain/services.interface").IEmailService',
  ],
]);

console.log("Done fixing the rest of as any usages.");
