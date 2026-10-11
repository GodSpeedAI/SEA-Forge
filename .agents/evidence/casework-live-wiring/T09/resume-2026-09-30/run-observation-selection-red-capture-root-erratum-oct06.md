# Selector RED capture erratum — root comparisons

Root independently compared the accepted RED raw, exit and preflight03 archives
against their actual `/tmp/run-observation-selection-red-oct06-final-` captures:
`focused-01.raw`, `focused-01.exit`, `preflight-03.txt`. All three are byte-exact.
Those are the accepted actual-RED captures for joined session66279.

Earlier preflight02 transcription attempts are retained, never rewritten:

| Archive basename | Bytes | Equals actual preflight02 |
|---|---:|---|
| run-observation-selection-red-preflight-02-oct06.txt | 2416 | no |
| run-observation-selection-red-preflight-02-exact-oct06.txt | 2618 | no |
| run-observation-selection-red-preflight-02-native-oct06.txt | 2611 | yes |

Actual comparison source:
`/tmp/run-observation-selection-red-oct06-final-preflight-02.txt`.
The first two names are not claims of fidelity, despite the second's label.
The native archive has SHA-256
`1c343766d44e39529cd88c8a24c76236b264146ef85c1f5477a442e8a76504d9`.
No preflight02 variant is substituted for accepted preflight03, and no runtime
or resource claim relies on a failed transcription. These facts came from
independent Buffer.equals/byte-length/hash calculations on the complete files.
