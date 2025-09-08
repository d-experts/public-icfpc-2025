package aleph

import (
	"fmt"
	"icfpc/api"
	"icfpc/domain"
	"math/rand"
	"strconv"
)

var client = api.NewClient()

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

type Changed struct {
	Label    int
	Door     int
	NewLabel int
}

type Rooms struct {
	positions []domain.Room
	rooms     []domain.Room
	plan      []Changed
	labels    [][]int
	visited   []int
	// Correct       []domain.Room
	// CorrectLabels []int
}

func NewRooms(plan string, response []int, mapData domain.MapData) *Rooms {
	positions := []domain.Room{}
	rooms := []domain.Room{}
	labels := [][]int{}
	newPlan := []Changed{}
	for i := 0; i < len(mapData.Rooms); i++ {
		positions = append(positions, domain.Room{
			Number:  mapData.Rooms[i],
			Connect: make([]domain.Location, 6),
		})
		labels = append(labels, []int{mapData.Rooms[i]})
	}
	for _, conn := range mapData.Connections {
		from := conn.From
		to := conn.To
		positions[from.Room].Connect[from.Door] = to
		positions[to.Room].Connect[to.Door] = from
	}
	for j := 0; j < 2; j++ {
		for i := 0; i < len(mapData.Rooms); i++ {
			rooms = append(rooms, domain.Room{
				Number:  mapData.Rooms[i],
				Connect: NewLocation(),
			})
		}
	}
	newPlan = append(newPlan, Changed{
		NewLabel: response[1],
		Door:     -1,
		Label:    response[0],
	})
	for i := 3; i < len(plan); i += 4 {
		door, err := strconv.Atoi(string(plan[i]))
		if err != nil {
			panic(err)
		}
		if plan[i+1] != '[' {
			panic("invalid plan")
		}
		label, err := strconv.Atoi(string(plan[i+2]))
		if err != nil {
			panic(err)
		}
		newPlan = append(newPlan, Changed{
			NewLabel: label,
			Door:     door,
			Label:    response[i/2+1],
		})
	}
	return &Rooms{
		positions: positions,
		rooms:     rooms,
		labels:    labels,
		plan:      newPlan,
		visited:   make([]int, len(positions)),
	}
}

func (r *Rooms) GetRooms() []domain.Room {
	return r.rooms
}

func (r *Rooms) changeLabel(room, label int) {
	pos := room % r.posN()
	idx := room / r.posN()
	if len(r.labels[pos]) == 2 {
		r.labels[pos][idx] = label
	} else {
		other := r.labels[pos][0]
		if idx == 0 {
			r.labels[pos] = []int{label, other}
		} else {
			r.labels[pos] = []int{other, label}
		}
	}
	if r.labels[pos][0] == r.labels[pos][1] {
		r.labels[pos] = r.labels[pos][:1]
	}
}

func (r *Rooms) getLabel(room int) int {
	pos := room % r.posN()
	idx := room / r.posN()
	if len(r.labels[pos]) == 1 {
		return r.labels[pos][0]
	}
	return r.labels[pos][idx]
}

func (r *Rooms) findRoomByPos(pos, label int) int {
	if len(r.labels[pos]) == 1 {
		return -1
	}
	for i, l := range r.labels[pos] {
		if l == label {
			return i*r.posN() + pos
		}
	}
	return -1
}

func (r *Rooms) posN() int {
	return len(r.positions)
}

func (r *Rooms) connectRoom(from_room, from_door, to_room int, history *[]domain.Location) bool {
	connect := func(room1, door1, room2, door2 int) bool {
		exist_room := r.rooms[room1].Connect[door1].Room
		// fmt.Println("connect:", room1, door1, "->", room2, door2, "exist_room:", exist_room)
		if exist_room != -1 {
			if exist_room != room2 {
				r.removeConnections(history)
				return false // conflict
			}
			return true
		}
		r.rooms[room1].Connect[door1] = domain.Location{Room: room2, Door: door2}
		*history = append(*history, domain.Location{Room: room1, Door: door1})
		return true
	}

	from_pos := from_room % len(r.positions)
	to_door := r.positions[from_pos].Connect[from_door].Door
	if !connect(from_room, from_door, to_room, to_door) {
		return false
	}
	if !connect(to_room, to_door, from_room, from_door) {
		return false
	}
	from_room2 := (from_room + r.posN()) % len(r.rooms)
	to_room2 := (to_room + r.posN()) % len(r.rooms)
	if !connect(from_room2, from_door, to_room2, to_door) {
		return false
	}
	if !connect(to_room2, to_door, from_room2, from_door) {
		return false
	}
	return true
}

func (r *Rooms) removeConnections(history *[]domain.Location) {
	for _, loc := range *history {
		r.rooms[loc.Room].Connect[loc.Door] = domain.Location{Room: -1, Door: -1}
	}
	*history = []domain.Location{}
}

func (r *Rooms) dfs_connect_to(depth int, from_room, to_room int) bool {
	changed := r.plan[depth]
	history := []domain.Location{}
	if r.getLabel(to_room) != changed.Label {
		fmt.Println("label mismatch:", r.getLabel(to_room), "expected:", changed.Label)
		return false
	}
	// if r.CorrectLabels[correct_room] != changed.Label {
	// 	fmt.Println("****label mismatch:", r.getLabel(from_room), "expected:", r.CorrectLabels[correct_room], "****")
	// }
	// r.CorrectLabels[correct_room] = changed.NewLabel
	r.changeLabel(to_room, changed.NewLabel)
	ok := r.connectRoom(from_room, changed.Door, to_room, &history)
	defer func() {
		r.changeLabel(to_room, changed.Label)
		r.removeConnections(&history)
		// r.CorrectLabels[correct_room] = changed.Label
	}()

	if !ok {
		return false
	}

	if !r.dfs(depth+1, to_room) {
		return false
	}

	history = []domain.Location{}
	return true
}

func (r *Rooms) dfs(depth int, from_room int) bool {
	if len(r.plan) == depth {
		return true
	}
	// if r.Correct[correct_room].Number != r.rooms[from_room].Number {
	// 	fmt.Println("****room number mismatch:", r.rooms[from_room].Number, "expected:", r.Correct[correct_room].Number, "****")
	// }
	changed := r.plan[depth]
	from_pos := from_room % r.posN()
	to_room_candidate := r.rooms[from_room].Connect[changed.Door].Room
	to_pos := r.positions[from_pos].Connect[changed.Door].Room
	fmt.Println("candidate:", to_room_candidate)
	if to_room_candidate == -1 {
		// to_room を探す必要がある
		to_room_candidate = r.findRoomByPos(to_pos, changed.Label)
		fmt.Println("findRoomByPos:", to_room_candidate)
	}
	if r.visited[to_pos] == 0 {
		fmt.Println("visiting new position:", to_pos)
		to_room_candidate = to_pos
	}
	r.visited[to_pos]++
	defer func() { r.visited[to_pos]-- }()

	var to_rooms []int
	if to_room_candidate != -1 {
		to_rooms = []int{to_room_candidate}
	} else {
		to_rooms = []int{to_pos, to_pos + r.posN()}
	}
	fmt.Println("DFS depth:", depth, "from_room:", from_room, "to_rooms:", to_rooms, "from_door:", changed.Door, "to_door:", r.positions[from_pos].Connect[changed.Door].Door, "label:", changed.Label, "new_label:", changed.NewLabel)
	fmt.Println("labels:", r.labels[to_pos], "visited:", r.visited)

	// correct_next := r.Correct[correct_room].Connect[changed.Door].Room
	for _, to_room := range to_rooms {
		fmt.Println("try to connect to", to_room)
		if r.dfs_connect_to(depth, from_room, to_room) {
			return true
		}
	}
	return false
}

func guess(r *Rooms, start_room int) bool {
	res, err := client.Guess(r.GetRooms(), start_room)
	if err != nil {
		fmt.Println("Guess error:", err)
		return false
	}
	return res.Correct
}

func (r *Rooms) Complete() {
	cnt := 0
	for from_room := range r.rooms {
		for _, loc := range r.rooms[from_room].Connect {
			if loc.Room == -1 {
				cnt++
			}
		}
	}
	fmt.Println("unconnected doors:", cnt, "/", 6*len(r.rooms))

	for from_room := range r.rooms {
		for from_door, loc := range r.rooms[from_room].Connect {
			if loc.Room != -1 {
				continue
			}
			from_pos := from_room % r.posN()
			to_pos := r.positions[from_pos].Connect[from_door].Room
			to_room := (from_room - from_pos + to_pos + r.posN()*rand.Intn(2)) % r.posN()
			ok := r.connectRoom(from_room, from_door, to_room, &[]domain.Location{})
			if !ok {
				panic(fmt.Sprintf("cannot connect %d-%d to %d", from_room, from_door, to_room))
			}
		}
	}
}

func (r *Rooms) Size() int {
	return len(r.rooms)
}

func (r *Rooms) StartDFS(start_room int, correct_room int) bool {
	r.changeLabel(start_room, r.plan[0].NewLabel)
	r.visited[start_room%r.posN()]++
	res := r.dfs(1, start_room)
	r.visited[start_room%r.posN()]--
	r.changeLabel(start_room, r.plan[0].Label)
	return res
}

func Solve(plan string, results []int, inputMap domain.MapData, graph domain.MapData) bool {
	fmt.Println("plan:", plan)
	fmt.Println("results:", results)

	rooms := NewRooms(plan, results, inputMap)
	// fmt.Println("MapData:", graph)
	// for i := 0; i < len(graph.Rooms); i++ {
	// 	rooms.Correct = append(rooms.Correct, domain.Room{
	// 		Number:  graph.Rooms[i],
	// 		Connect: make([]domain.Location, 6),
	// 	})
	// 	rooms.CorrectLabels = append(rooms.CorrectLabels, graph.Rooms[i])
	// }
	// for _, conn := range graph.Connections {
	// 	from := conn.From
	// 	to := conn.To
	// 	rooms.Correct[from.Room].Connect[from.Door] = to
	// 	rooms.Correct[to.Room].Connect[to.Door] = from
	// }

	start_room := inputMap.StartingRoom
	dfs_res := rooms.StartDFS(start_room, graph.StartingRoom)
	fmt.Println("DFS result:", dfs_res)

	rooms.Complete()

	return guess(rooms, start_room)
}
