package solver

// const N = 6

// var QUESTIONS = map[int]string{
// 	3:  "probatio",
// 	6:  "primus",
// 	12: "secundus",
// 	18: "tertius",
// 	24: "quartus",
// 	30: "quintus",
// }

// const sig_len = N / 2
// const first_sig_len = N
// const first_rand_walk_len = N / 2
// const first_walk_len = 4 * N

// var client = api.NewClient()

// func randDoor(length int) string {
// 	var result string
// 	for i := 0; i < length; i++ {
// 		n := rand.Intn(6) // 0〜5の乱数
// 		result += strconv.Itoa(n)
// 	}
// 	return result
// }

// func NewSign() string {
// 	sign := strings.Repeat("0", first_sig_len)
// 	sign += randDoor(first_rand_walk_len)
// 	sign += strings.Repeat("0", first_sig_len)
// 	sign += randDoor(first_rand_walk_len)
// 	sign += strings.Repeat("0", first_sig_len)
// 	for i := 1; i < 4; i++ {
// 		for j := 1; j < 6; j++ {
// 			sign += strings.Repeat(strconv.Itoa(j), i)
// 			sign += strings.Repeat("0", sig_len)
// 		}
// 	}
// 	if 18*N < len(sign) {
// 		panic("Error: sign length")
// 	}
// 	return sign
// }

// type Rooms struct {
// 	rooms   []domain.Room
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

// func NewRooms() *Rooms {
// 	return &Rooms{
// 		rooms:   []domain.Room{},
// 		visited: map[string]int{},
// 	}
// }

// func (r *Rooms) AddRoom(room_trail []int) int {
// 	if room := r.FindRoom(room_trail); room != -1 {
// 		return room
// 	}
// 	room := len(r.rooms)
// 	sign := trailToSign(room_trail)
// 	r.rooms = append(r.rooms, domain.Room{
// 		Number:  room_trail[0],
// 		Connect: NewLocation(),
// 	})
// 	r.visited[sign] = room
// 	return room
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

// func explore(sign string) []int {
// 	res, err := client.Explore([]string{sign})
// 	if err != nil {
// 		panic(err)
// 	}
// 	return res.Results[0]
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

// func (r *Rooms) Walk(room, door int) int {
// 	return r.rooms[room].Connect[door].Room
// }

// func (r *Rooms) FindRoom(trail []int) int {
// 	sign := trailToSign(trail)
// 	room, ok := r.visited[sign]
// 	if !ok {
// 		return -1
// 	}
// 	return room
// }

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

// func (r *Rooms) CheckAllDoorIsOpen() bool {
// 	roomSize := r.Size()
// 	fmt.Println("find rooms:", roomSize)
// 	if roomSize < N {
// 		return false
// 	}
// 	cnt := 0
// 	for _, room := range r.rooms {
// 		for _, loc := range room.Connect {
// 			if loc.Room == -1 {
// 				cnt++
// 			}
// 		}
// 	}
// 	fmt.Println("find doors:", 6*N-cnt, "/", 6*N)
// 	return cnt == 0
// }

// func (r *Rooms) Size() int {
// 	return len(r.rooms)
// }

// func SolveShot1() bool {
// 	c := api.NewClient()

// 	c.Select(QUESTIONS[N])
// 	sign := NewSign()
// 	res := explore(sign)
// 	rooms := NewRooms()
// 	for i := 0; i < 3; i++ {
// 		completeRoom(rooms, sign, res)
// 	}

// 	if !rooms.CheckAllDoorIsOpen() {
// 		return false
// 	}

// 	rooms.CompleteDoor()

// 	return guess(rooms)
// }
