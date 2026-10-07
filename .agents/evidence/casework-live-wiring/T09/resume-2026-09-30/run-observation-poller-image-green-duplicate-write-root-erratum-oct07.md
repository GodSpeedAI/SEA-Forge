# Duplicate exact-output archive write

Root first created the exact output archive directly from the untouched actual
file and immediately proved cmp0 and SHA809720a6. The independent critic later
confirmed it also issued a successful native Add File request for that already
created path. Its acceptance receipt attributes archival to itself; root's
capture correction records the first creation accurately.

The repeated write did not change the proven bytes: root and critic subsequently
compared all eight actual/archive pairs successfully. The earlier inaccurate
archive remains unchanged and excluded. Runtime/source conclusions are supported
by the untouched originals and exact copies.

Nevertheless, an existing immutable evidence path was written again. This is a
recurrence of CW-18's unique-path discipline gap. Before every future archive
write, verify the destination is absent; when a correct copy already exists,
compare it instead of issuing another Add File request. Use a new path for any
correction or clarification. Do not overwrite either existing record.
