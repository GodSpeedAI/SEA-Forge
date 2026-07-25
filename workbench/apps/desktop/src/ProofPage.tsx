import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { useMachine } from "@xstate/react";
import { Button } from "@astryxdesign/core/Button";
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableHeaderCell,
  TableRow,
} from "@astryxdesign/core/Table";

import { statusMachine } from "./statusMachine";

// ponytail: stand-in for a real SFWP query (Task 3) — proves TanStack Query
// coexists with the rest of the stack, nothing more.
async function fetchProofRows(): Promise<Array<{ id: string; label: string }>> {
  await new Promise((resolve) => setTimeout(resolve, 10));
  return [
    { id: "authority", label: "Authority fabric" },
    { id: "evidence", label: "Evidence capture" },
    { id: "settlement", label: "Settlement declarations" },
  ];
}

export function ProofPage() {
  const search = useSearch({ strict: false }) as { filter?: string };
  const filter = search.filter ?? "";
  const navigate = useNavigate();
  const [filterInput, setFilterInput] = useState(filter);
  const { data: rows = [] } = useQuery({ queryKey: ["proof-rows"], queryFn: fetchProofRows });
  const [state, send] = useMachine(statusMachine);

  const visibleRows = rows.filter((row) =>
    row.label.toLowerCase().includes(filter.toLowerCase()),
  );

  return (
    <main style={{ padding: "var(--space-6)", display: "grid", gap: "var(--space-4)" }}>
      <h1 style={{ font: "var(--text-page-title)" }}>SEA Forge Workbench — stack proof</h1>

      <section style={{ display: "flex", gap: "var(--space-2)", alignItems: "center" }}>
        <Button
          label={state.matches("ready") ? "Recheck" : "Check readiness"}
          variant="primary"
          isLoading={state.matches("checking")}
          clickAction={async () => {
            send({ type: "CHECK" });
            await new Promise((resolve) => setTimeout(resolve, 250));
            send({ type: "READY" });
          }}
        />
        <span style={{ font: "var(--text-label)", color: "var(--fg-secondary)" }}>
          machine state: <code className="machine-value">{state.value as string}</code>
        </span>
      </section>

      <section style={{ display: "flex", gap: "var(--space-2)" }}>
        <input
          value={filterInput}
          onChange={(event) => setFilterInput(event.target.value)}
          placeholder="Filter rows…"
          style={{
            background: "var(--surface-panel)",
            color: "var(--fg-primary)",
            border: "var(--border-standard)",
            borderRadius: "var(--radius-control)",
            padding: "var(--space-2)",
          }}
        />
        <Button
          label="Apply filter"
          variant="secondary"
          onClick={() => navigate({ search: { filter: filterInput } as any })}
        />
      </section>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHeaderCell>ID</TableHeaderCell>
            <TableHeaderCell>Kernel surface</TableHeaderCell>
          </TableRow>
        </TableHeader>
        <TableBody>
          {visibleRows.map((row) => (
            <TableRow key={row.id}>
              <TableCell>
                <code className="machine-value">{row.id}</code>
              </TableCell>
              <TableCell>{row.label}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </main>
  );
}
