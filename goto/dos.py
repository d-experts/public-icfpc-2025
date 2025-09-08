import json
from random import randrange

import lib.icfpc_client as icfpc_client

team_id = ""
client = icfpc_client.ICFPCClient(team_id=team_id)

connections = json.load(open("connections.json"))
sasaoka_connection = json.load(open("sasaoka_connection.json"))
print("size of connections:", len(connections), flush=True)
rooms = [0, 1, 2]
loop = 0
with open("count.txt") as f:
    loop = int(f.readline())
while True:
    try:
        print(client.select("probatio"))
        is_corrected = client.guess(rooms, 0, connections[randrange(len(connections))])
        # is_corrected = client.guess(rooms, 0, sasaoka_connection)
        print(is_corrected)
    except Exception as e:
        print(e, flush=True)
        continue
    if is_corrected:
        break
    loop += 1
    if loop % 10 == 0:
        print(f"loop: {loop}", flush=True)
        with open("count.txt", "w") as f:
            print(loop, file=f)
