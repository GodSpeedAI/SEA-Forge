import { describe, expect, test } from "bun:test";
import { harEntries } from "./L6";

describe("harEntries", () => {
  test("reads url, method and status out of an HAR 1.2 document", () => {
    const har = { log: { entries: [{ request: { url: "http://x/assets/MarkdownRenderer-a.js", method: "GET" }, response: { status: 200 } }, { request: { url: "http://x/api/session" }, response: {} }] } };
    expect(harEntries(har)).toEqual([
      { url: "http://x/assets/MarkdownRenderer-a.js", method: "GET", status: 200 },
      { url: "http://x/api/session", method: "", status: 0 },
    ]);
  });
  test("a document without entries is empty, never an exception (the journey then fails on the count)", () => {
    expect(harEntries({})).toEqual([]);
    expect(harEntries(null)).toEqual([]);
  });
});
