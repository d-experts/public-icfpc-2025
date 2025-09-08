package main

import (
	"encoding/json"
	"fmt"
	"icfpc/domain"
	"icfpc/solver/aleph"
	"io"
	"log"
	"os"

	"github.com/joho/godotenv"
)

type GoInput struct {
	Plan    string         `json:"plan"`
	Results []int          `json:"results"`
	MapData domain.MapData `json:"mapData"`
}

func main() {
	log.Printf("Input: test")
	err := godotenv.Load()
	if err != nil {
		log.Fatal("Error loading .env file")
	}
	// Read JSON input from stdin
	inputData, err := io.ReadAll(os.Stdin)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading input: %v\n", err)
		os.Exit(1)
	}

	var input GoInput
	if err := json.Unmarshal(inputData, &input); err != nil {
		fmt.Fprintf(os.Stderr, "Error parsing JSON: %v\n", err)
		os.Exit(1)
	}

	// save input data
	err = os.WriteFile("input.json", inputData, 0644)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error saving input data: %v\n", err)
		os.Exit(1)
	}

	// res := aleph.Solve(input.Plan, input.Results, input.MapData, input.MapData)
	res := aleph.Solve(input.Plan, input.Results, input.MapData, input.MapData)
	// res を json にして標準出力に出す
	outputData, err := json.Marshal(res)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error marshaling output: %v\n", err)
		os.Exit(1)
	}
	fmt.Println(string(outputData))
}
