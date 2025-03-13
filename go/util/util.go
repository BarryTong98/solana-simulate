// util.go
package util

/*
#cgo LDFLAGS: -L${SRCDIR} -lsolana_simulate
#cgo darwin,arm64 LDFLAGS: -framework Security -framework CoreFoundation
#include <stdbool.h>
#include <stdlib.h>

extern bool simulate_transaction(
    const char* program_id,
    const char* accounts_json,
    const char* tx_json,
    const char* program_so_base64
);
*/
import "C"
import (
    "unsafe"
)

// SimulateTransaction calls the Rust function to simulate a Solana program
func SimulateTransaction(programID, accountsJSON, txJSON string, programSoBase64 string) bool {
    // Convert to C strings
    cProgramID := C.CString(programID)
    cAccountsJSON := C.CString(accountsJSON)
    cTxJSON := C.CString(txJSON)
    cProgramSoBase64 := C.CString(programSoBase64)

    // Ensure C strings are freed
    defer C.free(unsafe.Pointer(cProgramID))
    defer C.free(unsafe.Pointer(cAccountsJSON))
    defer C.free(unsafe.Pointer(cTxJSON))
    defer C.free(unsafe.Pointer(cProgramSoBase64))

    return bool(C.simulate_transaction(
        cProgramID,
        cAccountsJSON,
        cTxJSON,
        cProgramSoBase64,
    ))
}