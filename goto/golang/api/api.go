package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"time"

	"icfpc/domain"

	"github.com/joho/godotenv"
)

type Client struct {
	httpClient *http.Client
	id         string
	baseURL    string
}

func NewClient() *Client {
	err := godotenv.Load()
	if err != nil {
		log.Printf("Warning: .env file not found: %v", err)
	}

	id := os.Getenv("CLIENT_ID")
	if id == "" {
		log.Println("Warning: CLIENT_ID not found in .env file, using empty string")
	}

	baseURL := os.Getenv("API_BASE_URL")
	if baseURL == "" {
		baseURL = "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com/"
		log.Println("Warning: API_BASE_URL not found in .env file, using default:", baseURL)
	}

	return &Client{
		httpClient: &http.Client{
			Timeout: 10 * time.Second,
		},
		id:      id,
		baseURL: baseURL,
	}
}

type SelectRequest struct {
	Id   string `json:"id"`
	Name string `json:"problemName"`
}

type SelectResponse struct {
	Name string `json:"problemName"`
}

func (c *Client) apiRequest(url string, body io.Reader, response any) error {
	req, err := http.NewRequest("POST", url, body)
	if err != nil {
		return fmt.Errorf("failed to create data request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("failed to send data request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("data request failed with status %d: %s", resp.StatusCode, string(body))
	}
	if err := json.NewDecoder(resp.Body).Decode(&response); err != nil {
		return fmt.Errorf("failed to decode data response: %w", err)
	}

	return nil
}

func (c *Client) Select(name string) error {
	selectReq := SelectRequest{
		Id:   c.id,
		Name: name,
	}

	jsonData, err := json.Marshal(selectReq)
	if err != nil {
		return fmt.Errorf("failed to marshal data request: %w", err)
	}

	var res SelectResponse
	if err := c.apiRequest(c.baseURL+"select", bytes.NewBuffer(jsonData), &res); err != nil {
		return fmt.Errorf("Select request failed: %w", err)
	}

	if res.Name != name {
		return fmt.Errorf("unexpected problem name in response: got %s, want %s", res.Name, name)
	}

	fmt.Printf("Selected problem: %s\n", res.Name)
	return nil
}

type ExploreRequest struct {
	Id    string   `json:"id"`
	Plans []string `json:"plans"`
}

type ExploreResponse struct {
	Results    [][]int `json:"results"`
	QueryCount int     `json:"queryCount"`
}

func (c *Client) Explore(plans []string) (*ExploreResponse, error) {
	exploreReq := ExploreRequest{
		Id:    c.id,
		Plans: plans,
	}

	jsonData, err := json.Marshal(exploreReq)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal explore request: %w", err)
	}

	var res ExploreResponse
	if err := c.apiRequest(c.baseURL+"explore", bytes.NewBuffer(jsonData), &res); err != nil {
		return nil, fmt.Errorf("Explore request failed: %w", err)
	}

	fmt.Printf("Explore response: %d results, queryCount: %d\n", len(res.Results), res.QueryCount)
	return &res, nil
}

type GuessRequest struct {
	Id  string         `json:"id"`
	Map domain.MapData `json:"map"`
}

type GuessResponse struct {
	Correct bool `json:"correct"`
}

func (c *Client) Guess(rooms []domain.Room, start int) (*GuessResponse, error) {
	// Convert rooms to the format needed for the API
	roomSizes := make([]int, len(rooms))
	connections := []domain.Connection{}

	for i, room := range rooms {
		roomSizes[i] = room.Number
		// Build connections based on room connectivity
		for doorIdx, loc := range room.Connect {
			if loc.Room > i || (loc.Room == i && loc.Door >= doorIdx) {
				// Only add connections once (from lower room index to higher)
				connections = append(connections, domain.Connection{
					From: domain.Location{Room: i, Door: doorIdx},
					To:   domain.Location{Room: loc.Room, Door: loc.Door},
				})
			}
		}
	}
	fmt.Printf("Guessing with %d rooms, starting at room %d\n", len(rooms), start)

	guessReq := GuessRequest{
		Id: c.id,
		Map: domain.MapData{
			Rooms:        roomSizes,
			StartingRoom: start,
			Connections:  connections,
		},
	}
	fmt.Println(guessReq)

	jsonData, err := json.Marshal(guessReq)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal guess request: %w", err)
	}

	var res GuessResponse
	if err := c.apiRequest(c.baseURL+"guess", bytes.NewBuffer(jsonData), &res); err != nil {
		return nil, fmt.Errorf("Guess request failed: %w", err)
	}

	fmt.Printf("Guess response: correct = %v\n", res.Correct)
	return &res, nil
}
