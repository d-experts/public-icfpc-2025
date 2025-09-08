#!/usr/bin/env python3
from graph import Graph
import random
from itertools import product
import networkx as nx


def create_plan(length: int) -> list[int]:
    # 以下例は、の組み合わせを想定.下の図に対応.

    # 1.  まず3文字の組み合わせを列挙
    #       000, 001, 002, 003, 004, 005, 010, 011, 012, ..., 555
    # 2. これらをノードにしたDe bruijnグラフを作成.
    # 3. このグラフを元に点を一度しか通らないハミルトニアン路を見つけ出す.
    # 4. 見つけたハミルトニアン路に沿って文字列を集めると,
    # 5. 上記4.の数列の先頭の数字を集めていくとDe Bruijn列となる.

    # パラメータ設定
    k = 3  # k-mer のサイズ
    alphabet = list(range(6))  # 0から5までの数字

    # 1. k文字の組み合わせを列挙
    k_mers = ["".join(map(str, p)) for p in product(alphabet, repeat=k)]

    # 2. De Bruijn グラフを作成
    # (k-1)-mer をノードとし、k-mer をエッジとして扱う
    G = nx.DiGraph()

    for kmer in k_mers:
        prefix = kmer[:-1]  # 最初のk-1文字
        suffix = kmer[1:]  # 最後のk-1文字
        G.add_edge(prefix, suffix, label=kmer)

    # 3. オイラー路を見つける（De Bruijn グラフではハミルトニアン路ではなくオイラー路を使う）
    # すべてのエッジを通る経路を探す
    if nx.is_eulerian(G):
        # オイラー閉路が存在
        path = list(nx.eulerian_circuit(G))
    elif nx.has_eulerian_path(G):
        # オイラー路が存在
        path = list(nx.eulerian_path(G))
    else:
        # オイラー路が存在しない場合、ランダムな経路を生成
        # 実装を簡略化するため、単純にランダムな順列を返す
        result = []
        for _ in range(length):
            result.append(random.choice(alphabet))
        return result

    # 4. パスに沿って De Bruijn 列を構築
    de_bruijn = []

    # 最初のノードの文字を追加
    if path:
        first_node = path[0][0]
        de_bruijn.extend([int(c) for c in first_node])

        # 各エッジの最後の文字を追加
        for edge in path:
            suffix = edge[1]
            de_bruijn.append(int(suffix[-1]))

    assert length <= len(de_bruijn)

    return de_bruijn[:length]


def check():
    graph = Graph("secundus")
    N = graph.num_rooms
    plan = [random.randint(0, 5) for _ in range(18 * N)]

    room = graph.start_room
    visited: set[tuple[int, int]] = set()
    for i in range(len(plan)):
        door = plan[i]
        visited.add((room, door))
        room = graph.rooms[room].connections[door][0]

    return len(visited)


if __name__ == "__main__":
    cnt = 0
    s = 0
    for i in range(100):
        res = check()
        s += res
        if res == 6 * 12:
            cnt += 1
    print("success:", cnt, "/100")
    print("avg:", s / 100)
