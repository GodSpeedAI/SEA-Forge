# Unit1 RED01 command provenance correction

This new note corrects only the command presentation in `run_observation_manager_unit1_red01_independent_negative_verdict_oct07.md`; that first-written verdict remains unchanged.

The root handoff specifies the command, flags, and isolated test, but abbreviates the cache path as `/tmp/...setupfix1`. The archived preflight/output/exit captures do not contain the exact shell command or fully expanded cache directory. Therefore the fenced command in the verdict is a normalized transcription with an explicit placeholder, not a byte-exact command transcript. I make no claim that the exact GOCACHE value or complete invocation was preserved in those three capture files. The semantic result, exit status, output hashes, and preflight source/resource identities are directly supported by the archived artifacts and remain unchanged.
