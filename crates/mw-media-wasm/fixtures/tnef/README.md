# Independent TNEF test corpus

Source: https://github.com/jstedfast/MimeKit/tree/a250ce09775904ad1af532fc1f92c5d01cc89e37/UnitTests/TestData/tnef
Pinned revision: a250ce09775904ad1af532fc1f92c5d01cc89e37. License: MIT, retained in LICENSE.MimeKit. Downloaded unchanged TNEF inputs, extraction lists, and expected extracted files (except compressed-RTF body projections, unsupported by this preview). SHA256.json records exact bytes.

The data-before-name corpus contains historical BAT/SYS samples as inert test bytes. Tests only compare bytes; nothing is executed. Fixtures are never deployed as active content. Unicode/named/multivalue MAPI properties and binary attachment byte equality (text fixture CRLF/LF normalized to account for upstream Git normalization) are verified against this independent implementation's corpus. This is interoperability evidence, not a claim that every TNEF property or OLE/RTF rendering is supported.
