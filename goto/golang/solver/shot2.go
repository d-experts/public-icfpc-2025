package solver

import (
	"fmt"
	"icfpc/api"
	"icfpc/domain"
	"math/rand"
	"strconv"
	"strings"
)

const N = 12

var QUESTIONS = map[int]string{
	3:  "probatio",
	6:  "primus",
	12: "secundus",
	18: "tertius",
	24: "quartus",
	30: "quintus",
}

const sig_len = 3

var client = api.NewClient()

func randDoor(length int) string {
	var result string
	for i := 0; i < length; i++ {
		n := rand.Intn(5) + 1 // 1〜5の乱数
		result += strconv.Itoa(n)
	}
	return result
}

func NewFirstSign(first_sig_len, rand_walk_len int, limit int) string {
	sign := strings.Repeat("0", first_sig_len)
	for len(sign)+rand_walk_len+first_sig_len <= limit {
		sign += randDoor(rand_walk_len) + strings.Repeat("0", first_sig_len)
	}
	return sign
}

type Rooms struct {
	rooms   []domain.Room
	visited map[string]int
	path    []map[int]string
	routes  [][]string
}

func NewLocation() []domain.Location {
	return []domain.Location{
		{Room: -1, Door: -1},
		{Room: -1, Door: -1},
		{Room: -1, Door: -1},
		{Room: -1, Door: -1},
		{Room: -1, Door: -1},
		{Room: -1, Door: -1},
	}
}

func NewRooms() *Rooms {
	return &Rooms{
		rooms:   []domain.Room{},
		visited: map[string]int{},
		path:    []map[int]string{},
	}
}

func (r *Rooms) AddRoom(room_trail []int) int {
	if room := r.FindRoom(room_trail); room != -1 {
		return room
	}
	room := len(r.rooms)
	sign := trailToSign(room_trail)
	r.rooms = append(r.rooms, domain.Room{
		Number:  room_trail[0],
		Connect: NewLocation(),
	})
	r.visited[sign] = room
	r.path = append(r.path, make(map[int]string))
	return room
}

func (r *Rooms) ConnectPath(from_room, to_room int, doors string) {
	r.path[from_room][to_room] = doors
}

func (r *Rooms) Connect(from_room, from_door int, to_room_trail []int) int {
	to_room := r.AddRoom(to_room_trail)
	r.rooms[from_room].Connect[from_door].Room = to_room
	r.path[from_room][to_room] = strconv.Itoa(from_door)
	return to_room
}

func trailToSign(trail []int) string {
	var sign string
	for _, d := range trail {
		sign += strconv.Itoa(d)
	}
	return sign
}

func explore(sign string) []int {
	res, err := client.Explore([]string{sign})
	if err != nil {
		panic(err)
	}
	return res.Results[0]
}

func (r *Rooms) GetRooms() []domain.Room {
	return r.rooms
}

func guess(r *Rooms) bool {
	res, err := client.Guess(r.GetRooms(), 0)
	if err != nil {
		fmt.Println("Guess error:", err)
		return false
	}
	return res.Correct
}

func (r *Rooms) Walk(room, door int) int {
	return r.rooms[room].Connect[door].Room
}

func (r *Rooms) FindRoom(trail []int) int {
	sign := trailToSign(trail)
	room, ok := r.visited[sign]
	if !ok {
		return -1
	}
	return room
}

// func completeRoom(rooms *Rooms, plan string, response []int) {
// 	from_room := 0
// 	for i := 0; i < first_walk_len; i += first_sig_len + first_walk_len {
// 		from_room = rooms.AddRoom(response[i : i+sig_len])
// 		for j := 1; i+j < first_walk_len && j+sig_len <= first_sig_len; j++ {
// 			to_room := rooms.Connect(from_room, 0, response[i+j:i+j+sig_len])
// 			from_room = to_room
// 		}
// 	}
// 	fmt.Println("first rooms:", rooms.Size())
// 	nextRoom := func(room int) int {
// 		for i := 0; i < sig_len; i++ {
// 			if room == -1 {
// 				return -1
// 			}
// 			room = rooms.Walk(room, 0)
// 		}
// 		return room
// 	}

// 	for i := first_walk_len; i < len(plan); i += sig_len {
// 		fmt.Println(from_room, plan[i:])

// 		for ; i < len(plan); i++ {
// 			if plan[i+1] == '0' || from_room == -1 {
// 				break
// 			}
// 			from_room = rooms.Walk(from_room, response[i])
// 		}
// 		if from_room == -1 {
// 			for ; i < len(plan) && plan[i] != '0'; i++ {
// 			}
// 			from_room = rooms.FindRoom(response[i : i+sig_len])
// 			from_room = nextRoom(from_room)
// 			continue
// 		}
// 		door, err := strconv.Atoi(string(plan[i]))
// 		if err != nil {
// 			panic(err)
// 		}
// 		i++
// 		to_room := rooms.Connect(from_room, door, response[i:i+sig_len])
// 		fmt.Println(rooms.Size(), from_room, door, to_room)
// 		from_room = nextRoom(to_room)
// 	}
// }

func (r *Rooms) CompleteDoor() {
	for i, room1 := range r.rooms {
		for door1, loc := range room1.Connect {
			room2 := loc.Room
			for door2 := 0; door2 < 6; door2++ {
				if r.rooms[room2].Connect[door2].Room == i && r.rooms[room2].Connect[door2].Door == -1 {
					r.rooms[room2].Connect[door2].Door = door1
					r.rooms[i].Connect[door1].Door = door2
					break
				}
			}
		}
	}
}

func (r *Rooms) CheckAllDoorIsOpen() bool {
	roomSize := r.Size()
	fmt.Println("find rooms:", roomSize)
	if roomSize < N {
		return false
	}
	cnt := 0
	for _, room := range r.rooms {
		for _, loc := range room.Connect {
			if loc.Room == -1 {
				cnt++
			}
		}
	}
	fmt.Println("find doors:", 6*N-cnt, "/", 6*N)
	return cnt == 0
}

func (r *Rooms) Size() int {
	return len(r.rooms)
}
func (r *Rooms) CheckDoor0() int {
	cnt := 0
	for _, room := range r.rooms {
		if room.Connect[0].Room != -1 {
			cnt++
		}
	}
	return cnt
}
func (r *Rooms) CheckReachableRooms() int {
	for i := 0; i < r.Size(); i++ {
		r.routes = append(r.routes, make([]string, r.Size()))
		for room, route := range r.path[i] {
			r.routes[i][room] = route
		}
	}

	for k := 0; k < r.Size(); k++ {
		for i := 0; i < r.Size(); i++ {
			for j := 0; j < r.Size(); j++ {
				if i == j {
					continue
				}
				if r.routes[i][k] != "" && r.routes[k][j] != "" {
					if r.routes[i][j] == "" || len(r.routes[i][j]) > len(r.routes[i][k])+len(r.routes[k][j]) {
						r.routes[i][j] = r.routes[i][k] + r.routes[k][j]
					}
				}
			}
		}
	}

	cnt := 0
	for i := 0; i < r.Size(); i++ {
		for j := 0; j < r.Size(); j++ {
			if i == j || r.routes[i][j] != "" {
				cnt++
			}
		}
	}
	return cnt
}

// func (r *Rooms) NextPlan() {
// 	var plans []string
// 	var doors [][]domain.Location
// 	from_room := 0

// 	NewPlan := func(from_room, to_room, door int) string {
// 		plan := r.routes[from_room][to_room]
// 		plan += strconv.Itoa(door)
// 		plan += strings.Repeat("0", sig_len)
// 		return plan

// 	}

// 	for room_idx, room := range r.rooms {
// 		for door := 0; door < 6; door++ {
// 			if room.Connect[door].Room != -1 {
// 				continue
// 			}
// 			doors = append(doors, domain.Location{
// 				Room: room_idx,
// 				Door: door,
// 			})
// 			step := NewPlan(from_room, room_idx, door)
// 			plan = append(plan, step)
// 		}
// 	}

// 	fmt.Println("next plan len:", len(strings.Join(plan, "")))
// 	res := explore(strings.Join(plan, ""))
// 	plan_len := 0

// 	for i, p := range plan {
// 		loc := doors[i]
// 		plan_len += len(p)
// 		to_room := r.FindRoom(res[plan_len-sig_len : plan_len])
// 		r.rooms[loc.Room].Connect[loc.Door].Room = to_room
// 	}
// }

func SolveShot1() (bool, int) {
	c := api.NewClient()

	c.Select(QUESTIONS[N])

	// rooms := NewRooms()

	first_rand_walk := 1
	plan := NewFirstSign(sig_len+3, first_rand_walk, 8*N)
	plan += NewFirstSign(sig_len, first_rand_walk, 10*N)
	res := explore(plan)
	visited := make(map[string]int)
	opened := []map[int]bool{}
	before_room := -1
	for i := 0; i < len(plan); i += first_rand_walk {
		if before_room != -1 {
			d, _ := strconv.Atoi(string(plan[i-1]))
			opened[before_room][d] = true
		}
		for ; i+sig_len <= len(plan) && plan[i+sig_len-1] == '0'; i++ {
			sign := trailToSign(res[i : i+sig_len])
			if _, ok := visited[sign]; !ok {
				id := len(visited)
				visited[sign] = id
				opened = append(opened, make(map[int]bool))
			}
			fmt.Print(visited[sign], ",")
			before_room = visited[sign]
		}
		i += sig_len - 1
		fmt.Println()
	}
	fmt.Println("visited:", len(visited))
	cnt := 0
	for _, o := range opened {
		cnt += len(o)
	}
	return len(visited) == N, cnt

	// if rooms.Size() < N {
	// 	return false
	// }

	// if !rooms.CheckAllDoorIsOpen() {
	// 	return false
	// }

	// rooms.CompleteDoor()

	// return guess(rooms)
}
