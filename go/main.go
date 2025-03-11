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
	"io/ioutil"
	"log"
	"os"
	"unsafe"
)

// 修改后的函数，接受accounts.json的内容作为参数
func SimulateTransactionWithAccounts(accountsJSON string) bool {
	cAccountsJSON := C.CString(accountsJSON)
	defer C.free(unsafe.Pointer(cAccountsJSON))

	return bool(C.simulate_transaction_with_accounts_c(cAccountsJSON))
}

func main() {
	fmt.Println("Simulating transaction...")

	// 读取accounts.json文件
	jsonContent, err := ioutil.ReadFile("./accounts.json")
	if err != nil {
		log.Fatalf("无法读取accounts.json文件: %v", err)
		os.Exit(1)
	}

	// 将文件内容作为字符串传递给Rust函数
	result := SimulateTransactionWithAccounts(string(jsonContent))

	if result {
		fmt.Printf("Transaction simulation succeeded: %v\n", result)
	} else {
		fmt.Println("Transaction simulation failed!")
	}
}
