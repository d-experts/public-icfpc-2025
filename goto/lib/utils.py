import random


def make_random_query(length: int):
    # 長さlengthのランダムなクエリを生成
    return "".join([str(random.randrange(6)) for _ in range(length)])


def edge_to_connections(answer, V: list[int], V_to_idx: dict[int, int]):
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


# 層1つ
# probatio	3
# primus	6
# secundus	12
# tertius	18
# quartus	24
# quintus	30

# 層2つ
# aleph	12
# beth	24
# gimel	36
# daleth	48
# he	60

# 層3つ
# vau	18
# zain	36
# hhet	54
# teth	72
# iod	90

# (層の数, 部屋の数) → 部屋名 の辞書

problem_names = {
    1: {
        3: "probatio",
        6: "primus",
        12: "secundus",
        18: "tertius",
        24: "quartus",
        30: "quintus",
    },
    2: {12: "aleph", 24: "beth", 36: "gimel", 48: "daleth", 60: "he"},
    3: {18: "vau", 36: "zain", 54: "hhet", 72: "teth", 90: "iod"},
}
