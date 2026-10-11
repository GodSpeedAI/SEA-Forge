# Cell root and socket contract

One cell root owns one set of records and one socket. The server, the CLI, and
the desktop host resolve both from the same rule, so no surface can end up
reading one cell while writing another.

## Resolution order

Every surface applies this order, highest first:

| Rank | Input | Effect |
|---|---|---|
| 1 | `SEA_FORGE_SOCKET` | Absolute socket path. Wins outright. The only supported way to split the socket from the cell that holds the records. |
| 2 | `SEA_FORGE_ROOT` | The cell root. Records live under it; the socket is `<root>/server.sock`. |
| 3 | *(nothing set)* | Server and CLI: `.sea-forge` under the current directory. Desktop host: `$HOME/.sea-forge`. |

A relative path is always made absolute before use, so a surface's working
directory never changes which cell it names.

Rank 3 differs by surface on purpose. A windowed application has no meaningful
working directory, so anchoring the desktop default at `$HOME` is the only
default that names a stable cell. **A packaged install sets `SEA_FORGE_ROOT`
for every surface** — ranks 1 and 2 are the documented procedure, and rank 3 is
a convenience for a single home-directory cell.

## Cell layout

```
<root>/                     # SEA_FORGE_ROOT, e.g. /tmp/cell or $HOME/.sea-forge
├── server.yaml             # server configuration (optional)
├── server.sock             # SFWP v1 socket, mode 0600
├── server.sock.lock        # exclusive owner lock
├── cases/
├── runs/<run_id>/
├── ledgers/
└── drafts/
```

`server.yaml` lives inside the cell it configures, so a `root:` key in that file
cannot name a different cell: the resolved root always wins.

An explicit absolute `socket_path` in `server.yaml` is honored verbatim; a
relative one composes under the root.

## Starting a cell

```sh
export SEA_FORGE_ROOT=/path/to/cell
sea-forge-server                       # binds $SEA_FORGE_ROOT/server.sock
sea-forge run --root "$SEA_FORGE_ROOT" --intent "..."
sea-forge-workbench                    # connects to $SEA_FORGE_ROOT/server.sock
```

The CLI takes the root as `--root` rather than reading the environment, so a
scripted run always states the cell it operates on.

## Exclusivity

A second server on the same root fails with an `AddrInUse` error against
`<root>/server.sock.lock` and exits. It does not unlink a live socket: two
servers sharing one root would interleave appends into the same JSONL ledger and
MMR, which have no cross-process reconciliation.

The socket is published by binding a staging path, setting mode `0600`, then
renaming into place, so it is owner-only from the first instant it is reachable
and atomically replaces a socket left behind by a crashed server.
