#!/usr/bin/env python3
from flask import Flask, request, jsonify
from flask_cors import CORS
import os
import logging
import random
from datetime import datetime
from pathlib import Path
from graph import Graph, PROBLEM_DATA
from state import (
    selected_graph,
    exploration_data,
    total_query_count,
    guess_history,
    selected_problem,
    current_log_dir,
)
from init import load_settings, configure_debug_logging
import json

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


# Load settings on startup
settings = load_settings()

# Configure debug logging after settings are loaded
configure_debug_logging(settings)

app = Flask(__name__)
CORS(app)


@app.route("/select", methods=["POST"])
def select():
    """
    Handle select endpoint
    Expected request body: {"id": "string", "problemName": "string"}
    """
    try:
        data = request.get_json()
        logger.debug(f"API Input [/select]: {data}")
        if not data:
            return jsonify({"error": "No JSON data provided"}), 400

        # Validate required fields (id is optional)
        if "problemName" not in data:
            return jsonify({"error": "Missing required field: problemName"}), 400

        # Store the selected problem locally
        global selected_problem
        selected_problem = data["problemName"]
        global total_query_count
        total_query_count = 0

        # Validate problem name
        if selected_problem not in PROBLEM_DATA:
            return jsonify({"error": f"Unknown problem: {selected_problem}"}), 400

        # Reset random seed before graph generation for consistent results
        if "seed" in settings:
            random.seed(settings["seed"])
            logger.info(
                f"Random seed reset to: {settings['seed']} for graph generation"
            )

        # Create graph for the selected problem
        graph_folder = settings.get("graph_folder", None)
        graph = Graph(selected_problem, load_folder=graph_folder)

        # Store the graph globally for use in explore and guess
        global selected_graph
        selected_graph = graph

        # Create timestamp with milliseconds
        timestamp = datetime.now()
        timestamp_str = timestamp.strftime("%Y%m%d_%H%M%S_%f")[
            :-3
        ]  # Remove last 3 digits to get milliseconds

        # Create log directory with timestamp
        if settings.get("debug", False):
            log_dir = Path("log") / timestamp_str
            log_dir.mkdir(parents=True, exist_ok=True)

            # Store the log directory globally for use in other endpoints
            global current_log_dir
            current_log_dir = log_dir

            # Save graph.json to the timestamped folder
            graph_json = graph.to_json()
            graph_file = log_dir / "graph.json"
            with open(graph_file, "w") as f:
                json.dump(graph_json, f, indent=2)

        logger.info(
            f"Selected problem: {selected_problem}, saved graph to {graph_file}"
        )

        result = {"problemName": selected_problem}
        logger.debug(f"API Output [/select]: {result}")
        return jsonify(result)

    except ValueError as e:
        logger.warning(f"Invalid value in /select: {e}")
        return jsonify({"error": str(e)}), 400
    except AssertionError as e:
        logger.warning(f"Assertion failed in /select: {e}")
        return jsonify(
            {"error": f"Invalid data: {str(e) if str(e) else 'Assertion failed'}"}
        ), 400
    except FileNotFoundError as e:
        logger.warning(f"File not found in /select: {e}")
        return jsonify({"error": str(e)}), 404
    except Exception as e:
        logger.error(f"Unexpected error in /select: {e}", exc_info=True)
        return jsonify({"error": "Internal server error"}), 500


@app.route("/explore", methods=["POST"])
def explore():
    """
    Handle explore endpoint
    Expected request body: {"id": "string", "plans": ["string"]}
    """
    try:
        data = request.get_json()
        logger.debug(f"API Input [/explore]: {data}")
        if not data:
            return jsonify({"error": "No JSON data provided"}), 400

        # Validate required fields (id is optional)
        if "plans" not in data:
            return jsonify({"error": "Missing required field: plans"}), 400

        if not isinstance(data["plans"], list):
            return jsonify({"error": "plans must be an array"}), 400

        if selected_graph is None:
            return jsonify(
                {"error": "No graph selected. Please select a problem first."}
            ), 400

        # Process exploration locally
        # For simulation, return mock results based on the number of plans
        plans = data["plans"]
        results = []

        for i, plan in enumerate(plans):
            if not isinstance(plan, str):
                return jsonify({"error": f"Plan at index {i} must be a string"}), 400
            try:
                selected_graph.explore(plan)
                results.append(selected_graph.explore(plan))
            except ValueError as e:
                return jsonify({"error": f"Invalid plan at index {i}: {str(e)}"}), 400

        # Store exploration data
        global exploration_data
        exploration_data[data.get("id", "default")] = {
            "plans": plans,
            "results": results,
        }

        query_count = len(plans) + 1
        response_data = {"results": results, "queryCount": query_count}
        global total_query_count
        total_query_count += query_count

        logger.info(
            f"Explore response: {len(results)} results, queryCount: {query_count}"
        )
        logger.debug(f"API Output [/explore]: {response_data}")
        return jsonify(response_data)

    except ValueError as e:
        logger.warning(f"Invalid value in /explore: {e}")
        return jsonify({"error": str(e)}), 400
    except AssertionError as e:
        logger.warning(f"Assertion failed in /explore: {e}")
        return jsonify(
            {"error": f"Invalid data: {str(e) if str(e) else 'Assertion failed'}"}
        ), 400
    except AttributeError as e:
        logger.warning(f"Attribute error in /explore: {e}")
        return jsonify(
            {"error": "Invalid graph state. Please select a problem first."}
        ), 400
    except Exception as e:
        logger.error(f"Unexpected error in /explore: {e}", exc_info=True)
        return jsonify({"error": "Internal server error"}), 500


@app.route("/guess", methods=["POST"])
def guess():
    """
    Handle guess endpoint
    Expected request body: {
        "id": "string",
        "map": {
            "rooms": [int],
            "startingRoom": int,
            "connections": [{"from": {"room": int, "door": int}, "to": {"room": int, "door": int}}]
        }
    }
    """
    try:
        data = request.get_json()
        logger.debug(f"API Input [/guess]: {data}")
        if not data:
            return jsonify({"error": "No JSON data provided"}), 400

        # Validate required fields (id is optional)
        if "map" not in data:
            return jsonify({"error": "Missing required field: map"}), 400

        map_data = data["map"]
        if (
            "rooms" not in map_data
            or "startingRoom" not in map_data
            or "connections" not in map_data
        ):
            return jsonify(
                {
                    "error": "Missing required map fields: rooms, startingRoom, and connections"
                }
            ), 400

        if not isinstance(map_data["rooms"], list):
            return jsonify({"error": "rooms must be an array"}), 400

        if not isinstance(map_data["connections"], list):
            return jsonify({"error": "connections must be an array"}), 400

        # Validate connection structure
        for conn in map_data["connections"]:
            if "from" not in conn or "to" not in conn:
                return jsonify(
                    {"error": "Each connection must have 'from' and 'to' fields"}
                ), 400
            for loc in [conn["from"], conn["to"]]:
                if "room" not in loc or "door" not in loc:
                    return jsonify(
                        {"error": "Each location must have 'room' and 'door' fields"}
                    ), 400

        logger.info(
            f"Guessing with {len(map_data['rooms'])} rooms, starting at room {map_data['startingRoom']}"
        )
        logger.debug(f"Full guess request: {data}")

        # Check if a graph has been selected
        if selected_graph is None:
            return jsonify(
                {"error": "No graph selected. Please select a problem first."}
            ), 400

        # Create a Graph object from the guessed map data using Graph.from_api
        try:
            guessed_graph = Graph.from_api(data)

            # for test
            plans: list[str] = exploration_data[data.get("id", "default")]["plans"]
            results: list[str] = exploration_data[data.get("id", "default")]["results"]

            for k, (plan, result) in enumerate(zip(plans, results)):
                res1 = selected_graph.explore(plan)
                res2 = guessed_graph.explore(plan)
                assert len(res1) == len(res2)
                for i, (r1, r2, rb) in enumerate(zip(res1, res2, result)):
                    assert r1 == r2, (
                        f"explore({k}) response error at index {i}, bef: {r1}, guess: {r2}, before: {rb}, response: {res1}, {res2}"
                    )

            # Use the is_same function to check if the graphs match
            is_correct = selected_graph.is_same(guessed_graph)

        except (AssertionError, ValueError, KeyError) as e:
            logger.warning(f"Invalid graph data in /guess: {e}")
            return jsonify({"error": f"Invalid graph data: {str(e)}"}), 400

        logger.info(
            f"Selected graph: {selected_graph.num_rooms} rooms, start: {selected_graph.start_room}"
        )
        logger.info(
            f"Guessed graph: {guessed_graph.num_rooms} rooms, start: {guessed_graph.start_room}"
        )
        logger.info(f"Graph comparison result: {is_correct}")

        # Store guess history
        global guess_history
        guess_history.append(
            {"id": data.get("id"), "map": map_data, "correct": is_correct}
        )

        # Save logs to the current log directory if it exists and debug is enabled
        if current_log_dir is not None and settings.get("debug", False):
            try:
                # Save guessed graph as JSON
                guessed_graph_data = {
                    "id": data.get("id", "unknown"),
                    "map": map_data,
                    "correct": is_correct,
                    "timestamp": datetime.now().isoformat(),
                }
                guess_file = current_log_dir / "guess.json"
                with open(guess_file, "w") as f:
                    json.dump(guessed_graph_data, f, indent=2)

                # Save exploration history
                explore_history = {
                    "exploration_data": exploration_data,
                    "total_query_count": total_query_count,
                    "timestamp": datetime.now().isoformat(),
                }
                history_file = current_log_dir / "explore_history.json"
                with open(history_file, "w") as f:
                    json.dump(explore_history, f, indent=2)

                logger.info(f"Saved guess and exploration history to {current_log_dir}")

            except Exception as e:
                logger.warning(f"Failed to save logs: {e}")

        result = {"correct": is_correct}
        logger.info(f"Guess response: correct = {is_correct}")
        logger.debug(f"API Output [/guess]: {result}")
        return jsonify(result)

    except ValueError as e:
        logger.warning(f"Invalid value in /guess: {e}")
        return jsonify({"error": str(e)}), 400
    except AssertionError as e:
        logger.warning(f"Assertion failed in /guess: {e}")
        return jsonify(
            {"error": f"Invalid data: {str(e) if str(e) else 'Assertion failed'}"}
        ), 400
    except AttributeError as e:
        logger.warning(f"Attribute error in /guess: {e}")
        return jsonify(
            {"error": "Invalid graph state. Please select a problem first."}
        ), 400
    except Exception as e:
        logger.error(f"Unexpected error in /guess: {e}", exc_info=True)
        return jsonify({"error": "Internal server error"}), 500


if __name__ == "__main__":
    port = int(os.getenv("PORT", 5000))
    debug = os.getenv("DEBUG", "False").lower() == "true"

    logger.info(f"Starting API server on port {port}")
    logger.info(f"Debug mode: {debug}")

    app.run(host="0.0.0.0", port=port, debug=debug)
