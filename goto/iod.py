import random
from copy import deepcopy

import icfpc_client

team_id = ""
client = icfpc_client.ICFPCClient(team_id=team_id)


def edge_to_connections(answer, n, V: list[int], V_to_idx: dict[int, int]):
    for room in V:
        for door in range(6):
            assert answer[room][door] != -1, f"未回答: {room} {door}"
    connections = []
    answered = {v: [False] * 6 for v in V}
    for room in V:
        for door in range(6):
            if answered[room][door]:
                continue
            to = answer[room][door]
            door2 = -1
            for d2 in range(6):
                if answered[to][d2]:
                    continue
                if answer[to][d2] == room:
                    door2 = d2
                    break
            assert door2 != -1, f"逆方向が見つからない: {room} {door} -> {to}"
            connection = {
                "from": {"room": V_to_idx[room], "door": door},
                "to": {"room": V_to_idx[answer[room][door]], "door": door2},
            }
            connections.append(connection)
            answered[room][door] = True
            answered[to][door2] = True
    for room in V:
        for door in range(6):
            assert answered[room][door], f"未回答: {room} {door}"

    return connections


def explore(query: str, n: int):
    C = [0] * 4
    response = client.explore([query])["results"][0]
    original_response = deepcopy(response)
    true_response = deepcopy(response)
    wagatta = [False] * len(response)

    while True:
        # 分かってない最小のidを求める
        i = 0
        while i < len(wagatta):
            if not wagatta[i]:
                break
            i += 1
        if i == len(wagatta):
            break
        # i にチョークを塗る
        new_query = query
        choke = "[1]" if original_response[i] == 0 else "[0]"
        new_query = new_query[:i] + choke + new_query[i:]
        res = client.explore([new_query])
        print(res["queryCount"])
        new_response = res["results"][0]
        new_response.pop(i)
        assert len(original_response) == len(new_response)
        idx = -1
        for i in range(len(original_response)):
            if original_response[i] != new_response[i]:
                wagatta[i] = True
                idx = int(original_response[i])
                true_response[i] = C[idx] * 4 + idx
        C[idx] += 1
    V = list(set(true_response))
    edges = {v: [-1] * 6 for v in V}
    for i in range(len(query)):
        edges[true_response[i]][int(query[i])] = true_response[i + 1]

    return edges, true_response


def explore_batch(query: str, V: list[int], insert_pos: dict[int, int]):
    response = client.explore([query])["results"][0]
    original_response = deepcopy(response)
    true_response = deepcopy(response)

    new_queries = []
    insert_pos_list = []
    for i in insert_pos.values():
        new_query = query
        choke = "[1]" if original_response[i] == 0 else "[0]"
        new_query = new_query[:i] + choke + new_query[i:]
        new_queries.append(new_query)
        insert_pos_list.append(i)

    res = client.explore(new_queries)
    print(res["queryCount"])
    new_responses = res["results"]
    C = [0] * 4
    for j, new_response in enumerate(new_responses):
        i = insert_pos_list[j]
        new_response.pop(i)
        new_responses[j] = new_response
        assert len(original_response) == len(new_response)
        idx = -1
        for i in range(len(original_response)):
            if original_response[i] != new_response[i]:
                idx = int(original_response[i])
                true_response[i] = C[idx] * 4 + idx
        C[idx] += 1

    edges = {v: [-1] * 6 for v in V}
    for j in range(len(query)):
        edges[true_response[j]][int(query[j])] = true_response[j + 1]

    return edges, true_response


def solve():
    n = 90
    client.select("iod")

    query = "".join([str(random.randint(0, 5)) for _ in range(6 * n)])
    edges, res = explore(query, n)

    # 全部の頂点が出るところでクエリを切る
    V = list(set(res))
    s = set()
    insert_pos = {v: -1 for v in V}  # 頂点vが最初に出てくる位置
    idx = -1
    for i, r in enumerate(res):
        if insert_pos[r] == -1:
            insert_pos[r] = i
        s.add(r)
        if len(s) == n:
            idx = i
            break
    print("base query length:", idx)
    if idx == -1:
        raise Exception("頂点情報揃ってない")
    if idx >= 400:
        raise Exception("カス seed")
    base_query = query[: idx + 1]
    while True:
        new_query = base_query + "".join(
            [str(random.randint(0, 5)) for _ in range(6 * n - len(base_query))]
        )
        edge, res = explore_batch(new_query, V, insert_pos)
        for v in V:
            for door in range(6):
                if edge[v][door] == -1:
                    continue
                if edges[v][door] == -1:
                    edges[v][door] = edge[v][door]
                else:
                    assert edges[v][door] == edge[v][door], (
                        f"{edges[v][door], edge[v][door]}"
                    )

        all_ok = True
        for v in V:
            for d in range(6):
                all_ok &= edges[v][d] != -1
        if all_ok:
            break
        print("不十分")
        print(edges)

    print("十分")
    print(edges)
    V_to_idx = {v: i for i, v in enumerate(V)}
    connections = edge_to_connections(edges, len(V), V, V_to_idx)
    rooms = [v % 4 for v in V]
    is_correct = client.guess(rooms, 0, connections)
    print("result:", is_correct)
    return is_correct


while True:
    try:
        res = solve()
        if res:
            break
    except Exception as e:
        print("Error:", e)
        continue
