package main

/*
#cgo LDFLAGS: -L/Users/barry/binance/solana-simulate/target/release -lsolana_simulate
#cgo darwin,arm64 LDFLAGS: -framework Security -framework CoreFoundation
#include <stdbool.h>
#include <stdlib.h>

extern bool simulate_transaction_with_accounts_c(const char* accounts_json);
*/
import "C"
import (
	"fmt"
	"unsafe"
)

func SimulateTransactionWithAccounts(accountsJSON string) bool {
	cAccountsJSON := C.CString(accountsJSON)
	defer C.free(unsafe.Pointer(cAccountsJSON))

	return bool(C.simulate_transaction_with_accounts_c(cAccountsJSON))
}

func main() {
	fmt.Println("Simulating transaction...")

	accountsPath := "./accounts.json" //
	result := SimulateTransactionWithAccounts(accountsPath)

	if result {
		fmt.Printf("Transaction simulation succeeded: %v\n", result)
	} else {
		fmt.Println("Transaction simulation failed!")
	}
}
