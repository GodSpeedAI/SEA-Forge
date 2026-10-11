import type { Journey, Ctx } from "../ladder";

export const smoke: Journey = {
  id: "SMOKE",
  title: "App boots and renders UI",
  depends_on: [],
  settles: "app-boots",
  unlocks: [],
  async run(ctx: Ctx) {
    await ctx.step("open app", async () => {
      await ctx.b.open(`${ctx.base}/?source=local`);
    });

    await ctx.step("wait for UI", async () => {
      await ctx.b.waitFor(
        `(document.querySelector('#root')?.children.length ?? 0) > 0`,
        15000,
        "root element with children"
      );
    });

    await ctx.step("screenshot boot", () => {
      ctx.shot("boot");
    });

    await ctx.step("verify DOM", async () => {
      const hasRoot = await ctx.b.eval<boolean>(
        `!!document.querySelector('#root')`
      );
      ctx.expect(hasRoot, "root element should exist");
    });
  },
};
