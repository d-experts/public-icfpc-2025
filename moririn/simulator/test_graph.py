#!/usr/bin/env python3
import json
from graph import Graph

# Load the saved graph
with open("log/20250906_234912_106/graph.json", "r") as f:
    data = json.load(f)

# Create two identical graphs from the same data
rooms1, start1 = Graph.from_json(data)
graph1 = Graph.__new__(Graph)
graph1.rooms = rooms1
graph1.start_room = start1
graph1.num_rooms = len(rooms1)
graph1.problem_name = "probatio"
graph1.repeat = 1

rooms2, start2 = Graph.from_json(data)
graph2 = Graph.__new__(Graph)
graph2.rooms = rooms2
graph2.start_room = start2
graph2.num_rooms = len(rooms2)
graph2.problem_name = "probatio"
graph2.repeat = 1

# Test if they are considered the same
result = graph1.is_same(graph2)
print(f"Are the identical graphs considered the same? {result}")

# Debug info
print(f"Graph1: {len(graph1.rooms)} rooms, starting at {graph1.start_room}")
print(f"Graph2: {len(graph2.rooms)} rooms, starting at {graph2.start_room}")