package main

/*
#cgo LDFLAGS: -L/Users/barry/binance/solana-simulate/target/release -lsolana_simulate
#cgo darwin,arm64 LDFLAGS: -framework Security -framework CoreFoundation
#include <stdbool.h>

extern bool simulate_transaction_c();
*/
import "C"
import (
	"fmt"
)

func SimulateTransaction() bool {
	return bool(C.simulate_transaction_c())
}

func main() {
	fmt.Println("Simulating transaction...")

	// Call the Rust implementation
	result := SimulateTransaction()

	if result {
		fmt.Printf("%v", result)
	} else {
		fmt.Println("Transaction simulation failed!")
	}
}
