#!/usr/bin/env python3
import json
import random
from pathlib import Path
from copy import deepcopy
from logging import getLogger
from dataclasses import dataclass


PROBLEM_DATA = {
    "probatio": (3, 1),
    "primus": (6, 1),
    "secundus": (12, 1),
    "tertius": (18, 1),
    "quartus": (24, 1),
    "quintus": (30, 1),
    "aleph": (6, 2),
    "beth": (12, 2),
    "gimel": (18, 2),
    "daleth": (24, 2),
    "he": (30, 2),
    "vau": (6, 3),
    "zain": (12, 3),
    "hhet": (18, 3),
    "teth": (24, 3),
    "iod": (30, 3),
    "testmoririn": (3, 3),
}


@dataclass
class Room:
    """Represents a room with 6 door connections."""

    label: int
    connections: list[tuple[int, int]]


class Graph:
    """Graph class that generates a room graph based on problem name."""

    def __init__(self, problem_name: str, load_folder: str | None = None):
        """
        Initialize a graph for the given problem.

        Args:
            problem_name: Name of the problem from PROBLEM_DATA
            load_folder: Optional folder to load the graph JSON from.
        """
        if problem_name not in PROBLEM_DATA:
            raise ValueError(f"Unknown problem: {problem_name}")

        if load_folder is not None:
            self._load_graph(load_folder)
            self.num_rooms = len(self.rooms)
            return

        # Get problem dimensions for generation
        a, b = PROBLEM_DATA[problem_name]
        self.num_rooms = a * b  # Total number of rooms

        # Generate the graph
        self._generate_graph(a, b)
        self.start_room = 0

    def _is_connected(self) -> bool:
        """Check if the graph is fully connected using DFS."""
        visited = [False] * len(self.rooms)

        def dfs(room_id: int):
            visited[room_id] = True
            for connection in self.rooms[room_id].connections:
                if connection is not None:
                    target_room, _ = connection
                    if not visited[target_room]:
                        dfs(target_room)

        # Start DFS from the first room
        dfs(0)

        return all(visited)

    def _generate_graph(self, n: int, repeat: int) -> list[Room]:
        """Generate a connected graph where each room has 6 doors."""
        logger = getLogger(__name__)

        while True:
            logger.debug("Generating graph...")
            labels = [i % 4 for i in range(n)]
            connectings = [[(-1, -1)] * 6 for _ in range(n)]

            unassigned_doors = [(i, j) for i in range(n) for j in range(6)]
            random.shuffle(unassigned_doors)

            # Process each unassigned door from the front
            while unassigned_doors:
                room1_id, door1_id = unassigned_doors[0]
                loc2_idx = random.randint(0, len(unassigned_doors) - 1)
                room2_id, door2_id = unassigned_doors[loc2_idx]
                connectings[room1_id][door1_id] = (room2_id, door2_id)
                connectings[room2_id][door2_id] = (room1_id, door1_id)

                # Remove the assigned doors
                if loc2_idx > 0:
                    unassigned_doors.pop(loc2_idx)
                unassigned_doors.pop(0)

            self.rooms = [
                Room(label=labels[i], connections=list(connectings[i]))
                for i in range(n)
            ]

            if not self._is_connected():
                continue

            if repeat == 1:
                return

            # 複製して、辺をswapする
            for i in range(n, repeat * n):
                room = deepcopy(self.rooms[i - n])
                for j in range(6):
                    if room.connections[j] is not None:
                        room.connections[j] = (
                            room.connections[j][0] + n,
                            room.connections[j][1],
                        )
                self.rooms.append(room)

            for i1, from_room1 in enumerate(self.rooms):
                for from_door in range(6):
                    to_room1, to_door = from_room1.connections[from_door]
                    i2 = (i1 + n) % (self.num_rooms)
                    from_room2 = self.rooms[i2]
                    to_room2, to_door2 = from_room2.connections[from_door]
                    assert to_door == to_door2
                    if random.random() < 0.8:
                        continue

                    self.rooms[i1].connections[from_door] = (to_room2, to_door)
                    self.rooms[to_room2].connections[to_door] = (i1, from_door)
                    self.rooms[i2].connections[from_door] = (to_room1, to_door)
                    self.rooms[to_room1].connections[to_door] = (i2, from_door)

            if self._is_connected():
                return

    def _load_graph(self, folder_name: str):
        """Load graph from a JSON file in the graph_data folder.

        Args:
            file_name: Name of the problem (used as filename without .json)

        Returns:
            Tuple of (list of Room objects, starting room index)
        """
        # Construct the file path

        folder = Path(__file__).resolve().parent.joinpath(f"graph_data/{folder_name}")

        # フォルダ内にあるjsonファイルのうち、randomに1つ選ぶ
        json_files = list(folder.glob("*.json"))
        if not json_files:
            raise FileNotFoundError(f"No JSON files found in folder: {folder}")
        file_path = random.choice(json_files)
        with open(file_path, "r") as f:
            data = json.load(f)

        self.rooms, self.start_room = Graph.from_json(data)

    @staticmethod
    def from_json(data: dict) -> tuple[list[Room], int]:
        """Create a Graph object from JSON data."""
        map_data = data["map"]
        room_labels = map_data["rooms"]
        starting_room = map_data["startingRoom"]
        connections = map_data["connections"]

        # Create Room objects with labels
        num_rooms = len(room_labels)
        door_connections: list[list[tuple[int, int]]] = [
            [(-1, -1)] * 6 for _ in range(num_rooms)
        ]

        # Process connections
        for conn in connections:
            from_room = int(conn["from"]["room"])
            from_door = int(conn["from"]["door"])
            to_room = int(conn["to"]["room"])
            to_door = int(conn["to"]["door"])

            # Connect the doors
            assert 0 <= from_room < num_rooms
            assert 0 <= to_room < num_rooms
            assert 0 <= from_door < 6
            assert 0 <= to_door < 6
            door_connections[from_room][from_door] = (to_room, to_door)
            door_connections[to_room][to_door] = (from_room, from_door)

        assert all(conn.count((-1, -1)) == 0 for conn in door_connections), (
            "Unconnected doors found"
        )
        rooms = [
            Room(label=room_labels[i], connections=door_connections[i])
            for i in range(num_rooms)
        ]

        return rooms, starting_room

    @staticmethod
    def from_api(data: dict) -> "Graph":
        """Create a Graph object from API request data.

        Args:
            data: API request data containing map information

        Returns:
            Graph instance ready for comparison
        """
        # Extract rooms and starting room using existing from_json method
        rooms, start_room = Graph.from_json(data)

        # Create new Graph instance
        graph = Graph.__new__(Graph)
        graph.rooms = rooms
        graph.start_room = start_room
        graph.num_rooms = len(rooms)

        return graph

    def to_json(self) -> dict:
        """Convert the graph to JSON format.

        Returns:
            Dictionary in the specified JSON format
        """
        # Extract room labels
        room_labels = [room.label for room in self.rooms]

        # Build connections list
        connections = []
        processed_pairs = set()

        for room_id, room in enumerate(self.rooms):
            for door_id, connection in enumerate(room.connections):
                if connection is not None:
                    target_room, target_door = connection

                    # Create a unique identifier for this connection pair
                    pair_id = tuple(
                        sorted([(room_id, door_id), (target_room, target_door)])
                    )

                    # Only add if we haven't processed this pair yet
                    if pair_id not in processed_pairs:
                        connections.append(
                            {
                                "from": {"room": room_id, "door": door_id},
                                "to": {"room": target_room, "door": target_door},
                            }
                        )
                        processed_pairs.add(pair_id)

        # Build the final JSON structure
        return {
            "id": "id",
            "map": {
                "rooms": room_labels,
                "startingRoom": self.start_room,
                "connections": connections,
            },
        }

    def print_graph(self):
        """Pretty print the graph structure."""
        print(f"Graph for problem '{self.problem_name}' ({self.num_rooms} rooms):")
        for room_id, room in enumerate(self.rooms):
            print(f"Room {room_id}:")
            for door_id, connection in enumerate(room.connections):
                if connection is not None:
                    target_room, target_door = connection
                    print(f"  Door {door_id} -> Room {target_room}, Door {target_door}")
                else:
                    print(f"  Door {door_id} -> (unconnected)")

    def explore(self, plan: str) -> list[int]:
        labels = [room.label for room in self.rooms]
        room = self.start_room
        results = [labels[room]]
        i = 0
        while i < len(plan):
            if plan[i] == "[" and i + 2 < len(plan) and plan[i + 2] == "]":
                labels[room] = int(plan[i + 1])
                assert 0 <= labels[room] <= 3
                results.append(labels[room])
                i += 3
                continue

            if plan[i] < "0" or plan[i] > "5":
                raise ValueError(f"Invalid door character in plan: {plan[i]}")
            door = int(plan[i])
            room = self.rooms[room].connections[door][0]

            results.append(labels[room])
            i += 1
        return results

    def is_same(self, graph: "Graph") -> bool:
        """Check if this graph is the same as another graph."""
        logger = getLogger(__name__)
        if self.num_rooms != graph.num_rooms:
            return False
        for k in range(2):
            room1 = self.start_room
            room2 = graph.start_room
            labels1 = [room.label for room in self.rooms]
            labels2 = [room.label for room in graph.rooms]
            route = ""
            room1s = [room1]
            room2s = [room2]
            for i in range(100000):
                if labels1[room1] != labels2[room2]:
                    logger.debug(
                        f"Labels differ at iteration {i}: {labels1[room1]} != {labels2[room2]}, route: {route}, room1s: {room1s}, room2s: {room2s}"
                    )
                    return False
                if k > 0 and i % 10 == 0:
                    label = random.randint(0, 3)
                    route += f"[{label}]"
                    labels1[room1] = label
                    labels2[room2] = label

                door = random.randint(0, 5)
                route += str(door)
                room1 = self.rooms[room1].connections[door][0]
                room2 = graph.rooms[room2].connections[door][0]
                room1s.append(room1)
                room2s.append(room2)

        return True
