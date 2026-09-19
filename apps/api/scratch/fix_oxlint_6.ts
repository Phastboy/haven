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

// Fix incorrect mock type conversions in google-oauth.e2e.test.ts
replaceInFile("auth/presentation/__tests__/google-oauth.e2e.test.ts", [
  [
    /\} \) as \{ token\?: string; \[key: string\]: unknown \};/g,
    "} ) as unknown as { token?: string; [key: string]: unknown };",
  ],
  [
    /\] as \{ token\?: string; \[key: string\]: unknown \};/g,
    "] as unknown as { token?: string; [key: string]: unknown };",
  ],
  [
    /expect\(account!\.emailVerified\)\.toBe\(true\);/,
    "expect((account as unknown as { emailVerified: boolean }).emailVerified).toBe(true);",
  ],
  [
    /expect\(oauth!\.providerUserId\)\.toBe\(googleId\);/,
    "expect((oauth as unknown as { providerUserId: string }).providerUserId).toBe(googleId);",
  ],
  [/account!\.id/g, "(account as unknown as { id: string }).id"],
]);

// Fix magic-link.e2e.test.ts
replaceInFile("auth/presentation/__tests__/magic-link.e2e.test.ts", [
  [
    /\] as \{ token\?: string; \[key: string\]: unknown \};/g,
    "] as unknown as { token?: string; [key: string]: unknown };",
  ],
  [
    /expect\(magicLink!\.usedAt\)\.toBeNull\(\);/,
    "expect((magicLink as unknown as { usedAt: Date | null }).usedAt).toBeNull();",
  ],
  [
    /expect\(account!\.emailVerified\)\.toBe\(true\);/,
    "expect((account as unknown as { emailVerified: boolean }).emailVerified).toBe(true);",
  ],
  [/account!\.id/g, "(account as unknown as { id: string }).id"],
]);

// Fix session.e2e.test.ts
replaceInFile("auth/presentation/__tests__/session.e2e.test.ts", [
  [
    /\] as \{ token\?: string; \[key: string\]: unknown \};/g,
    "] as unknown as { token?: string; [key: string]: unknown };",
  ],
  [
    /expect\(data\.account\)\.toBeDefined\(\);/,
    "expect((data as unknown as { account: unknown }).account).toBeDefined();",
  ],
  [
    /expect\(data\.account\.id\)\.toBe\(testAccountId\);/,
    "expect((data as unknown as { account: { id: string } }).account.id).toBe(testAccountId);",
  ],
]);

// Fix user.e2e.test.ts
replaceInFile("user/presentation/__tests__/user.e2e.test.ts", [
  [
    /expect\(data\.bio\)\.toBe\("New Bio"\);/,
    'expect((data as unknown as { bio: string }).bio).toBe("New Bio");',
  ],
]);

// Fix module paths
const moduleReplacements: [RegExp, string][] = [
  [
    /import\("\.\.\/\.\.\/domain\/repositories\.interface"\)\.ISessionRepository/g,
    'import("../../domain/ports/ISessionRepository").ISessionRepository',
  ],
  [
    /import\("\.\.\/\.\.\/domain\/repositories\.interface"\)\.IAccountRepository/g,
    'import("../../domain/ports/IAccountRepository").IAccountRepository',
  ],
  [
    /import\("\.\.\/\.\.\/domain\/repositories\.interface"\)\.IOAuthAccountRepository/g,
    'import("../../domain/ports/IOAuthCredentialRepository").IOAuthCredentialRepository',
  ],
  [
    /import\("\.\.\/\.\.\/domain\/repositories\.interface"\)\.IMagicLinkRepository/g,
    'import("../../domain/ports/IMagicLinkRepository").IMagicLinkRepository',
  ],
  [
    /import\("\.\.\/\.\.\/domain\/services\.interface"\)\.IGoogleOAuthService/g,
    'import("../../domain/ports/IGoogleTokenService").IGoogleTokenService',
  ],
  [
    /import\("\.\.\/\.\.\/domain\/services\.interface"\)\.IEmailService/g,
    'import("../../domain/ports/IEmailService").IEmailService',
  ],
  [
    /import\("\.\.\/\.\.\/domain\/services\.interface"\)\.ITokenService/g,
    'import("../../infrastructure/services/token.service").TokenService',
  ],
  [
    /import\("\.\.\/\.\.\/application\/use-cases\/create-profile\.use-case"\)\.CreateProfileUseCase/g,
    'import("../../domain/ports/IProfileCreator").IProfileCreator',
  ],
];

replaceInFile("auth/application/__tests__/get-session.use-case.test.ts", moduleReplacements);
replaceInFile("auth/application/__tests__/get-session.test.ts", moduleReplacements);
replaceInFile("auth/application/__tests__/google-oauth.test.ts", moduleReplacements);
replaceInFile("auth/application/__tests__/logout.test.ts", moduleReplacements);
replaceInFile("auth/application/__tests__/request-magic-link.test.ts", moduleReplacements);

console.log("Done fixing the rest of TS errors.");
