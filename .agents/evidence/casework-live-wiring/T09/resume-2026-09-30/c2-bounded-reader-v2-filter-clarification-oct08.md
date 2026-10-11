# Version2 filter completion clarification

Root selects deferred unknown-cursor refusal for new version2 paging only.
Legacy retains bounded all-or-nothing refusal with no partial event vector.
Bind exact from_cursor/to_cursor filters into a signed continuation and require
matching resumed request values. They filter outputs after global validation;
they cannot replace origin proof, skip unread rows or shorten coverage to head.

Prior incomplete v2 pages are validated prefix outputs, not evidence that the
requested range or global pinned head is complete. If a requested bound is
still absent at the actual pinned head, the completing request fails typed
input_error with no page or completion marker. Unknown from_cursor cannot emit
matching events before discovery. No arbitrary cursor index, two-pass full
preflight, synthetic cursor or unbounded scan is introduced.

Go global frontier acknowledgement may reflect only actually validated page
progress. Ready current state and successful replay completion require all
normative journal/capture/head gates; incomplete pages do not satisfy them.
This is a concrete v2 wire clarification for proposal repair and independent
review, to be included in the targeted operator decision before source release.
It does not modify legacy behavior, authority or the existing live EventFrame.
