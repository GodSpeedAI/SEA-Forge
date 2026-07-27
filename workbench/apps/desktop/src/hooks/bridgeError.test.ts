import { describe, expect, it } from "vitest";
import { toError } from "./bridgeError";

describe("bridge error normalization", () => {
  it("preserves a bare string rejection as readable text", () => {
    // The host commands are `Result<T, String>`, so a failed `invoke` rejects
    // with a string. Reading `.message` off it directly would put the word
    // "undefined" in front of the operator exactly when the failure matters.
    expect(toError("SocketError::Disconnected").message).toBe("SocketError::Disconnected");
  });

  it("passes an Error through unchanged", () => {
    const original = new Error("already typed");
    expect(toError(original)).toBe(original);
  });

  it("never yields an empty message for a shapeless rejection", () => {
    expect(toError(undefined).message).toBe("unknown bridge failure");
    expect(toError({ code: 7 }).message).not.toBe("");
  });
});
