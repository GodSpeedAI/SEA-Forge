import { describe, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach } from "vitest";
import { CoreViewport } from "../core/CoreViewport";
import { ContextIdentity } from "../core/chrome/ContextIdentity";
import { ContextReadout } from "../core/chrome/ContextReadout";
import { InteractionHints } from "../core/chrome/InteractionHints";
import { RenderFailureSurface } from "../core/chrome/RenderFailureSurface";
import { Composer } from "../composer/Composer";
import { SurfaceRail } from "../shell/SurfaceRail";
import { CoreTuningPanel } from "../dev/CoreTuningPanel";
import { DEFAULT_VISUAL_PARAMS } from "../core/CoreVisualState";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ to, children, className, ...props }: any) => (
    <a href={to} className={className} {...props}>
      {children}
    </a>
  ),
  useLocation: () => ({ pathname: "/" }),
  useNavigate: () => () => {},
}));

afterEach(cleanup);

describe("spatial shell (white-labeled donor chrome)", () => {
  it("CoreViewport mounts the persistent canvas and surfaces WebGL failure usefully", () => {
    // jsdom has no WebGL2: the renderer must fail closed with an error
    // surface, never a black screen.
    render(<CoreViewport />);
    expect(screen.getByTestId("core-viewport")).toBeInTheDocument();
    expect(screen.getByTestId("core-canvas")).toBeInTheDocument();
    const failure = screen.getByTestId("render-failure");
    expect(failure).toBeInTheDocument();
    expect(failure).toHaveAttribute("role", "alert");
    expect(screen.getByText("WEBGL2 UNAVAILABLE")).toBeInTheDocument();
  });

  it("RenderFailureSurface renders all fatal taxonomy fields", () => {
    render(
      <RenderFailureSurface
        info={{ title: "CONTEXT LOST", message: "GPU dropped.", detail: "webglcontextlost" }}
      />,
    );
    expect(screen.getByText("CONTEXT LOST")).toBeInTheDocument();
    expect(screen.getByText("webglcontextlost")).toBeInTheDocument();
  });

  it("demo copy is white-labeled out of every product chrome piece", () => {
    const { container } = render(
      <div>
        <ContextIdentity />
        <ContextReadout focus="CORE" surface="home" />
        <InteractionHints />
      </div>,
    );
    const text = container.textContent ?? "";
    for (const banned of [
      "GARGANTUA",
      "EVENT CONTROL",
      "Schwarzschild",
      "ISCO",
      "Disk Plasma",
      "Time Dilation",
    ]) {
      expect(text).not.toContain(banned);
    }
    expect(screen.getByTestId("context-identity")).toBeInTheDocument();
    expect(screen.getByTestId("context-readout")).toBeInTheDocument();
    expect(screen.getByTestId("interaction-hints")).toBeInTheDocument();
  });

  it("Composer renders only projected affordances and routes through intents", () => {
    render(
      <Composer
        surfaceId="home"
        affordances={[
          { id: "new-case", label: "Compose new case", path: "/cases/new" },
          { id: "no-cell", label: "No cell", disabled: true, reason: "down" },
        ]}
      />,
    );
    const composer = screen.getByTestId("composer");
    expect(composer).toBeInTheDocument();
    expect(screen.getByText("Compose new case")).toBeInTheDocument();
    expect(screen.getByText("No cell")).toBeDisabled();
  });

  it("SurfaceRail exposes the nine spatial surfaces with CORE first", () => {
    render(<SurfaceRail surfaceId="home" inboxCount={2} />);
    const rail = screen.getByTestId("surface-rail");
    expect(rail).toBeInTheDocument();
    expect(rail.getAttribute("aria-label")).toBe("Surfaces");
    const links = rail.querySelectorAll("a");
    expect(links.length).toBe(9);
    expect(links[0].textContent).toContain("CORE");
    expect(screen.getByText("2")).toBeInTheDocument();
  });

  it("CoreTuningPanel preserves donor ranges and stays collapsible", () => {
    render(
      <CoreTuningPanel params={{ ...DEFAULT_VISUAL_PARAMS }} fps={60} onChange={() => {}} />,
    );
    expect(screen.getByTestId("core-tuning-opener")).toBeInTheDocument();
    expect(screen.queryByTestId("core-tuning-panel")).not.toBeInTheDocument();
  });
});
