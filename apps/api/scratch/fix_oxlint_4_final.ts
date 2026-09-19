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

replaceInFile("directory/presentation/__tests__/graphql.e2e.test.ts", [
  [/expect\(testOffer\.title\)/, "expect(testOffer!.title)"],
  [/expect\(testOffer\.price\)/, "expect(testOffer!.price)"],
  [/expect\(testOffer\.user\)\.toBeDefined\(\)/, "expect(testOffer!.user).toBeDefined()"],
  [/expect\(testOffer\.user\.id\)/, "expect(testOffer!.user.id)"],
  [/expect\(testOffer\.user\.username/, "expect(testOffer!.user.username"],
]);

replaceInFile("offer/presentation/__tests__/offer.e2e.test.ts", [
  [/expect\(body\[0\]\.title\)/, "expect(body[0]!.title)"],
]);

console.log("Done fixing the rest of TS errors.");
