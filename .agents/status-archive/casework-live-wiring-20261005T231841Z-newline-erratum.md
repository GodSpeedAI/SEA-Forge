# Status archive newline clarification

Native Add File patch construction added one trailing newline to each archived
status file. Root compared the full archive contents with the saved original
strings: each is exactly the original plus that single trailing LF. No status
content was lost or changed. The archives remain unchanged; they are prior
handoff context, not raw execution evidence. The initial byte-for-byte wording
in the new CURRENT_STATUS was too strong and is corrected there.
