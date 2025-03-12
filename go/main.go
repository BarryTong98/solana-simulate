package main

/*
#cgo LDFLAGS: -L/Users/barry/binance/solana-simulate/target/release -lsolana_simulate
#cgo darwin,arm64 LDFLAGS: -framework Security -framework CoreFoundation
#include <stdbool.h>
#include <stdlib.h>

extern bool simulate_transaction_with_accounts_c(const char* accounts_json, const char* tx_json);
*/
import "C"
import (
	"fmt"
	"log"
	"os"
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

func main() {
	fmt.Println("Simulating transaction...")

	// Read accounts.json file
	accountsJSON, err := os.ReadFile("./accounts.json")
	if err != nil {
		log.Fatalf("Failed to read accounts.json file: %v", err)
		os.Exit(1)
	}

	// Read tx.json file
	txJSON, err := os.ReadFile("./tx.json")
	if err != nil {
		log.Fatalf("Failed to read tx.json file: %v", err)
		os.Exit(1)
	}

	// Pass both JSON strings to the Rust function
	result := SimulateTransactionWithAccounts(string(accountsJSON), string(txJSON))

	if result {
		fmt.Printf("Transaction simulation succeeded: %v\n", result)
	} else {
		fmt.Println("Transaction simulation failed!")
	}
}