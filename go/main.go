// main.go
package main

import (
    "fmt"
    "log"
    "os"

    "github.com/BarryTong98/solana-simulate/util"
)

func main() {
    fmt.Println("Simulating program...")

    // Program ID
    programID := "HuTkmnrv4zPnArMqpbMbFhfwzTR7xfWQZHH1aQKzDKFZ"

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

    // Path to program.so file
    programSoPath := "/Users/barry/binance/solana-simulate/src/program.so"

    // Call Rust function for simulation
    result := util.SimulateProgramWithSO(
        programID,
        string(accountsJSON),
        string(txJSON),
        programSoPath,
    )

    if result {
        fmt.Println("Program simulation succeeded!")
    } else {
        fmt.Println("Program simulation failed!")
    }
}