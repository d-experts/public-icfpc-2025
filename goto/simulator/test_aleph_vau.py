#!/usr/bin/env python3
"""
Test cases for graph generation.
"""

import unittest
from graph import Graph


class TestGraphGeneration(unittest.TestCase):
    """Test cases graph generation."""

    def test_graph_creation(self):
        graph = Graph("testmoririn")

        print("\n=== ALEPH GRAPH ===")
        graph.print_graph()

        # Check basic properties
        self.assertEqual(graph.repeat, 3)
        self.assertEqual(graph.num_rooms, 9)
        self.assertIsNotNone(graph.start_room)
        self.assertGreaterEqual(graph.start_room, 0)
        self.assertLess(graph.start_room, 9)

        # Check that all rooms exist
        self.assertEqual(len(graph.rooms), 9)
        for i in range(9):
            room = graph.rooms[i]
            self.assertGreaterEqual(room.label, 0)
            self.assertLess(room.label, 4)
            # Each room should have exactly 6 door connections
            self.assertEqual(len(room.connections), 6)
            for door in range(6):
                connected_room, connected_door = room.connections[door]
                self.assertGreaterEqual(connected_room, 0)
                self.assertLess(connected_room, len(graph.rooms))
                self.assertIn(connected_door, range(6))


if __name__ == "__main__":
    unittest.main()
