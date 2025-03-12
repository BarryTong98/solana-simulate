package util

/*
#cgo LDFLAGS: -L${SRCDIR} -lsolana_simulate
#cgo darwin,arm64 LDFLAGS: -framework Security -framework CoreFoundation
#include <stdbool.h>
#include <stdlib.h>

extern bool simulate_transaction_with_accounts_c(const char* accounts_json, const char* tx_json);
*/
import "C"
import (
   "unsafe"
)

// SimulateTransactionWithAccounts calls the Rust function to simulate a Solana transaction
// using the provided accounts JSON and transaction JSON data
func SimulateTransactionWithAccounts(accountsJSON, txJSON string) bool {
   cAccountsJSON := C.CString(accountsJSON)
   cTxJSON := C.CString(txJSON)
   defer C.free(unsafe.Pointer(cAccountsJSON))
   defer C.free(unsafe.Pointer(cTxJSON))

   return bool(C.simulate_transaction_with_accounts_c(cAccountsJSON, cTxJSON))
}
