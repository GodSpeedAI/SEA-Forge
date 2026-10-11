# UI bounded diagnostic root capture supplement

Root independently compared the three diagnostic raw archives against actual originals, all byte-exact:

- Cursor: `/tmp/renderer-diag-cursor-20261006-112430-2.raw`; actual exit file0 newline.
- Default listener: `/tmp/renderer-diag-listener-default-20261006-112456-2.raw`; actual exit file1 newline.
- Authorized local-only listener: `/tmp/renderer-diag-listener-escalated-20261006-112514-3838991.raw`; actual exit file0 newline.

Adjacent new `.exit` files preserve those actual exit bytes. Preflight values are reported in the independent diagnostic record, but separate raw preflight captures were not supplied with these three raw archives; root cannot byte-verify that part. These bounded diagnoses do not replace the failed canonical UI gate or establish a new full-gate pass.
