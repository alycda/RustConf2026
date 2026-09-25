// AI-generated Swift binding over nativelib (count_e_acute).
//
// The native contract is "input is UTF-8". This binding, reaching for a
// familiar-looking encoding, hands the bytes over as ISO Latin-1 instead.
// For pure-ASCII input the two encodings are byte-identical, so the tests
// pass and it ships. For any accented character the bytes diverge silently:
// no crash, just a wrong count.
//
// One planted flaw. See REVIEW.md.

import Foundation

func countEAcute(_ s: String) -> Int64 {
    // THE FLAW: encode as ISO Latin-1, not UTF-8. 'é' becomes the single byte
    // 0xE9, which is not valid UTF-8 on the native side — so it is never
    // counted. The correct call is `s.utf8CString` / `s.withCString`.
    var bytes = Array(s.data(using: .isoLatin1) ?? Data())
    bytes.append(0)  // NUL-terminate (still a well-formed C string — in contract)
    return bytes.withUnsafeBufferPointer { buf in
        buf.baseAddress!.withMemoryRebound(to: CChar.self, capacity: buf.count) {
            count_e_acute($0)
        }
    }
}

let input = "café résumé"          // three 'é'
let expected: Int64 = 3
let got = countEAcute(input)
print("count_e_acute(\"\(input)\") = \(got) (expected \(expected))")

if got != expected {
    print("FLAW MANIFESTED: encoding mismatch silently produced the wrong count.")
    exit(0)  // flaw confirmed present
} else {
    FileHandle.standardError.write("flaw did NOT manifest (encoding looks fixed)\n".data(using: .utf8)!)
    exit(1)
}
