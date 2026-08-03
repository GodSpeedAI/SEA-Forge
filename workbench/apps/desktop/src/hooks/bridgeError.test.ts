import { describe, expect, it } from "vitest";
import { BridgeGovernedError, toError } from "./bridgeError";

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

  it("preserves a structured host refusal without flattening its governance fields", () => {
    const error = toError({
      error: "this cell configures no identity bindings",
      error_class: "identity_unconfigured",
      no_side_effect: true,
      next_lawful_action: "Configure an identity binding",
    });

    expect(error).toBeInstanceOf(BridgeGovernedError);
    expect(error).toMatchObject({
      errorClass: "identity_unconfigured",
      noSideEffect: true,
      nextLawfulAction: "Configure an identity binding",
    });
  });

  it("never yields an empty message for a shapeless rejection", () => {
    expect(toError(undefined).message).toBe("unknown bridge failure");
    expect(toError({ code: 7 }).message).not.toBe("");
  });
});
