package domain

type Location struct {
	Room int `json:"room"`
	Door int `json:"door"`
}

type Room struct {
	Number  int
	Connect []Location
}

type Connection struct {
	From Location `json:"from"`
	To   Location `json:"to"`
}

type MapData struct {
	Rooms        []int        `json:"rooms"`
	StartingRoom int          `json:"startingRoom"`
	Connections  []Connection `json:"connections"`
}
