import { describe, expect, it } from "vitest";
import { createActor } from "xstate";
import { statusMachine } from "./statusMachine";

describe("statusMachine", () => {
  it("transitions idle -> checking -> ready", () => {
    const actor = createActor(statusMachine).start();
    expect(actor.getSnapshot().value).toBe("idle");

    actor.send({ type: "CHECK" });
    expect(actor.getSnapshot().value).toBe("checking");

    actor.send({ type: "READY" });
    expect(actor.getSnapshot().value).toBe("ready");
  });

  it("transitions checking -> degraded on FAIL", () => {
    const actor = createActor(statusMachine).start();
    actor.send({ type: "CHECK" });
    actor.send({ type: "FAIL" });
    expect(actor.getSnapshot().value).toBe("degraded");
  });
});
