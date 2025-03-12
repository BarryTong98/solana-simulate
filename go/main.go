package main

import (
	"fmt"
	"log"
	"os"

	"github.com/BarryTong98/solana-simulate/util"
)

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
	result := util.SimulateTransactionWithAccounts(string(accountsJSON), string(txJSON))

	if result {
		fmt.Printf("Transaction simulation succeeded: %v\n", result)
	} else {
		fmt.Println("Transaction simulation failed!")
	}
}
