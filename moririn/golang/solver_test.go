package main

import (
	"encoding/json"
	"icfpc/api"
	"icfpc/solver/aleph"
	"io"
	"os"
	"testing"
)

func TestSolveWithInputJSON(t *testing.T) {
	// Read input.json file
	inputFile, err := os.Open("input.json")
	if err != nil {
		t.Fatalf("Error opening input.json: %v", err)
	}
	defer inputFile.Close()

	inputData, err := io.ReadAll(inputFile)
	if err != nil {
		t.Fatalf("Error reading input.json: %v", err)
	}
	var input GoInput
	if err := json.Unmarshal(inputData, &input); err != nil {
		t.Fatalf("Error parsing JSON: %v", err)
	}

	inputFile, err = os.Open("data/graph.json")
	if err != nil {
		t.Fatalf("Error opening graph.json: %v", err)
	}
	defer inputFile.Close()

	inputData, err = io.ReadAll(inputFile)
	if err != nil {
		t.Fatalf("Error reading graph.json: %v", err)
	}

	var req api.GuessRequest
	if err := json.Unmarshal(inputData, &req); err != nil {
		t.Fatalf("Error parsing graph.json: %v", err)
	}

	res := aleph.Solve(input.Plan, input.Results, input.MapData, req.Map)

	t.Logf("Solve result: %v", res)
}
