package solver

// import (
// 	"fmt"
// 	"icfpc/api"
// 	"icfpc/domain"
// 	"math/rand"
// 	"strconv"
// )

// const N = 30

// var client = api.NewClient()
// var visited map[string]int
// var rooms []domain.Room
// var paths []string
// var sign = randDoor(2 * N)
// var sign2 = randDoor(2 * N)

// const SIG = 2

// func randDoor(length int) string {
// 	var result string
// 	for i := 0; i < length; i++ {
// 		n := rand.Intn(6) // 0〜5の乱数
// 		result += strconv.Itoa(n)
// 	}
// 	return result
// }

// func explore(inputs []string) [][]int {
// 	plans := make([]string, len(inputs)*SIG)
// 	for i, in := range inputs {
// 		plans[i*SIG] = in + sign
// 		plans[i*SIG+1] = in + sign2
// 	}
// 	res, err := client.Explore(plans)
// 	if err != nil {
// 		panic(err)
// 	}
// 	return res.Results
// }

// func toString2(room1 []int, room2 []int) string {
// 	var result string
// 	for _, r := range room1 {
// 		result += strconv.Itoa(r)
// 	}
// 	result += "-"
// 	for _, r := range room2 {
// 		result += strconv.Itoa(r)
// 	}
// 	return result
// }

// func getBeforeRoom(plan string) int {
// 	for j, path := range paths {
// 		if path == plan[:len(plan)-1] {
// 			return j
// 		}
// 	}
// 	panic("not found")
// }

// func completeRoom(plans []string, responses [][]int) {
// 	for i, p := range plans {
// 		response1 := responses[SIG*i]
// 		response2 := responses[SIG*i+1]
// 		sig := toString(response1[len(p):], response2[len(p):])
// 		fmt.Println(sig)
// 		if _, ok := visited[sig]; ok {
// 			if len(p) == 0 {
// 				continue
// 			}
// 			room_idx := visited[sig]
// 			bef_room := getBeforeRoom(p)
// 			door := int(p[len(p)-1] - '0')
// 			rooms[bef_room].Connect[door].Room = room_idx

// 			continue
// 		}
// 		room_idx := len(rooms)
// 		bef_room := getBeforeRoom(p)
// 		rooms = append(rooms, domain.Room{
// 			Number: response1[len(p)],
// 			Connect: []domain.Location{
// 				{Room: -1, Door: -1},
// 				{Room: -1, Door: -1},
// 				{Room: -1, Door: -1},
// 				{Room: -1, Door: -1},
// 				{Room: -1, Door: -1},
// 				{Room: -1, Door: -1},
// 			},
// 		})
// 		door := int(p[len(p)-1] - '0')
// 		rooms[bef_room].Connect[door].Room = room_idx
// 		paths = append(paths, p)
// 		visited[sig] = room_idx
// 	}
// }

// func completeDoor() {
// 	for i, room := range rooms {
// 		for door, loc := range room.Connect {
// 			r := loc.Room
// 			for d := 0; d < 6; d++ {
// 				if rooms[r].Connect[d].Room == i && rooms[r].Connect[d].Door == -1 {
// 					rooms[r].Connect[d].Door = door
// 					rooms[i].Connect[door].Door = d
// 					break
// 				}
// 			}
// 		}
// 	}
// }

// func Solve() {
// 	c := api.NewClient()

// 	c.Select("quintus")
// 	rooms = make([]domain.Room, 0)
// 	visited = make(map[string]int)
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
// 	visited[toString(res[0], res[1])] = 0
// 	rooms = append(rooms, domain.Room{
// 		Number: -1,
// 		Connect: []domain.Location{
// 			{Room: -1, Door: -1},
// 			{Room: -1, Door: -1},
// 			{Room: -1, Door: -1},
// 			{Room: -1, Door: -1},
// 			{Room: -1, Door: -1},
// 			{Room: -1, Door: -1},
// 		},
// 	})
// 	rooms[0].Number = res[0][0]
// 	paths = []string{""}
// 	fmt.Println(paths)
// 	completeRoom(plans, res)
// 	fmt.Println(rooms)

// 	for {
// 		if len(rooms) > N {
// 			fmt.Errorf("too many rooms: %d", len(rooms))
// 			return
// 		}
// 		plans = make([]string, 0)
// 		for i, room := range rooms {
// 			for door, loc := range room.Connect {
// 				if loc.Room == -1 {
// 					plans = append(plans, paths[i]+strconv.Itoa(door))
// 				}
// 			}
// 		}
// 		if len(plans) == 0 {
// 			break
// 		}
// 		res = explore(plans)
// 		completeRoom(plans, res)
// 	}

// 	completeDoor()

// 	c.Guess(rooms, 0)
// }
