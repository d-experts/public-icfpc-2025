package solver

// const N = 3

// var QUESTIONS = map[int]string{
// 	3:  "probatio",
// 	6:  "primus",
// 	12: "secundus",
// 	18: "tertius",
// 	24: "quartus",
// 	30: "quintus",
// }

// var client = api.NewClient()

// type Rooms struct {
// 	rooms   []domain.Room
// 	paths   []string
// 	visited map[string]int
// }

// func NewLocation() []domain.Location {
// 	return []domain.Location{
// 		{Room: -1, Door: -1},
// 		{Room: -1, Door: -1},
// 		{Room: -1, Door: -1},
// 		{Room: -1, Door: -1},
// 		{Room: -1, Door: -1},
// 		{Room: -1, Door: -1},
// 	}
// }

// func NewRooms(init_trail []int) *Rooms {
// 	sign := trailToSign(init_trail)
// 	return &Rooms{
// 		rooms: []domain.Room{
// 			{Number: init_trail[0], Connect: NewLocation()},
// 		},
// 		paths:   []string{""},
// 		visited: map[string]int{sign: 0},
// 	}
// }

// func (r *Rooms) GetRoom(path string) int {
// 	for j, p := range r.paths {
// 		if path == p {
// 			return j
// 		}
// 	}
// 	fmt.Println("GetRoom: not found", path)
// 	return -1
// }

// func (r *Rooms) Connect(from_room, from_door int, to_room_trail []int) int {
// 	sign := trailToSign(to_room_trail)
// 	to_room, ok := r.visited[sign]
// 	if !ok {
// 		to_room = len(r.rooms)
// 		r.rooms = append(r.rooms, domain.Room{
// 			Number:  to_room_trail[0],
// 			Connect: NewLocation(),
// 		})
// 		r.paths = append(r.paths, r.paths[from_room]+strconv.Itoa(from_door))
// 		r.visited[sign] = to_room
// 	}

// 	r.rooms[from_room].Connect[from_door].Room = to_room
// 	return to_room
// }

// func trailToSign(trail []int) string {
// 	var sign string
// 	for _, d := range trail {
// 		sign += strconv.Itoa(d)
// 	}
// 	return sign
// }

// func explore(inputs []string) [][]int {
// 	plans := make([]string, len(inputs))

// 	for i, in := range inputs {
// 		plans[i] = in + strings.Repeat("1", 2*N)
// 	}
// 	res, err := client.Explore(plans)
// 	if err != nil {
// 		panic(err)
// 	}
// 	return res.Results
// }

// func (r *Rooms) GetRooms() []domain.Room {
// 	return r.rooms
// }

// func guess(r *Rooms) bool {
// 	res, err := client.Guess(r.GetRooms(), 0)
// 	if err != nil {
// 		fmt.Println("Guess error:", err)
// 		return false
// 	}
// 	return res.Correct
// }

// func toString(room1 []int) string {
// 	var result string
// 	for _, r := range room1 {
// 		result += strconv.Itoa(r)
// 	}
// 	return result
// }

// func completeRoom(r *Rooms, plans []string, responses [][]int) {
// 	for i, p := range plans {
// 		response := responses[i]
// 		from_room := r.GetRoom(p[:len(p)-1])
// 		fmt.Println(trailToSign(response))
// 		for j := 0; j < N; j++ {
// 			from_door := 1
// 			if j == 0 {
// 				from_door = int(p[len(p)-1] - '0')
// 			}
// 			to_room := r.Connect(from_room, from_door, response[len(p)+j:len(p)+j+N])
// 			from_room = to_room
// 		}
// 	}
// }

// func (r *Rooms) CompleteDoor() {
// 	for i, room1 := range r.rooms {
// 		for door1, loc := range room1.Connect {
// 			room2 := loc.Room
// 			for door2 := 0; door2 < 6; door2++ {
// 				if r.rooms[room2].Connect[door2].Room == i && r.rooms[room2].Connect[door2].Door == -1 {
// 					r.rooms[room2].Connect[door2].Door = door1
// 					r.rooms[i].Connect[door1].Door = door2
// 					break
// 				}
// 			}
// 		}
// 	}
// }

// func (r *Rooms) Size() int {
// 	return len(r.rooms)
// }

// func (r *Rooms) GetAllUnknownPath() []string {
// 	var plans []string
// 	for i, room := range r.rooms {
// 		for door, loc := range room.Connect {
// 			if loc.Room == -1 {
// 				plans = append(plans, r.paths[i]+strconv.Itoa(door))
// 			}
// 		}
// 	}
// 	return plans
// }

// func SolveSign111() bool {
// 	c := api.NewClient()

// 	c.Select(QUESTIONS[N])
// 	plans := []string{
// 		"",
// 		"0",
// 		"1",
// 		"2",
// 		"3",
// 		"4",
// 		"5",
// 	}
// 	res := explore(plans)
// 	rooms := NewRooms(res[0][:N])
// 	completeRoom(rooms, plans[1:], res[1:])

// 	for {
// 		if rooms.Size() > N {
// 			fmt.Errorf("too many rooms: %d", rooms.Size())
// 			return false
// 		}
// 		plans = rooms.GetAllUnknownPath()
// 		if len(plans) == 0 {
// 			break
// 		}
// 		res = explore(plans)
// 		completeRoom(rooms, plans, res)
// 	}

// 	rooms.CompleteDoor()

// 	return guess(rooms)
// }
