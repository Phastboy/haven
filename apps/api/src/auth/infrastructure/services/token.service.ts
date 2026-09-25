import * as crypto from "crypto";
import type { Config } from "../../../config";

export class TokenService {
  readonly #secret: string;

  constructor(config: Config) {
    this.#secret = config.TOKEN_SECRET;
  }

  generate(bytes: number = 32): string {
    return crypto.randomBytes(bytes).toString("hex");
  }

  hash(token: string): string {
    return crypto.createHmac("sha256", this.#secret).update(token).digest("hex");
  }
}
