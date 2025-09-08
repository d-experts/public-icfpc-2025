from typing import Any, Dict, List, Optional

import requests


class ICFPCClient:
    """ICFPC 2025 コンテスト用のAPIクライアント"""

    def __init__(self, team_id: str):
        self.base_url = "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com"
        self.team_id = team_id

    def register(self, name: str, pl: str, email: str) -> str:
        """
        新しいチームを登録する

        Args:
            name: チーム名
            pl: プログラミング言語
            email: メールアドレス

        Returns:
            チームID（秘密にすること）
        """
        url = f"{self.base_url}/register"
        data = {"name": name, "pl": pl, "email": email}

        response = requests.post(url, json=data)
        response.raise_for_status()

        result = response.json()
        self.team_id = result["id"]
        return self.team_id

    def select(self, problem_name: str, team_id: Optional[str] = None) -> str:
        """
        解くべき問題を選択する

        Args:
            problem_name: 問題名（例: "probatio" for test）
            team_id: チームID（指定しない場合は登録時のIDを使用）

        Returns:
            選択された問題名
        """
        if team_id is None:
            if self.team_id is None:
                raise ValueError(
                    "team_id が設定されていません。register() を先に実行するか、team_id を指定してください。"
                )
            team_id = self.team_id

        url = f"{self.base_url}/select"
        data = {"id": team_id, "problemName": problem_name}

        response = requests.post(url, json=data)
        response.raise_for_status()

        result = response.json()
        return result["problemName"]

    def explore(
        self, plans: List[str], team_id: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        Ædificiumを探索する

        Args:
            plans: ルートプランのリスト（例: ["0325", "112"]）
                   各文字列は0-5の数字で構成され、通過するドアの順序を示す
            team_id: チームID（指定しない場合は登録時のIDを使用）

        Returns:
            結果の辞書:
                - results: 各ルートプランに対する観察結果のリスト
                - queryCount: これまでの総探索回数
        """
        if team_id is None:
            if self.team_id is None:
                raise ValueError(
                    "team_id が設定されていません。register() を先に実行するか、team_id を指定してください。"
                )
            team_id = self.team_id

        url = f"{self.base_url}/explore"
        data = {"id": team_id, "plans": plans}

        response = requests.post(url, json=data)
        response.raise_for_status()

        return response.json()

    def guess(
        self,
        rooms: List[int],
        starting_room: int,
        connections: List[Dict[str, Dict[str, int]]],
        team_id: Optional[str] = None,
    ) -> bool:
        """
        候補の地図を提出する

        Args:
            rooms: 各部屋の2ビット整数ラベルのリスト
            starting_room: 開始部屋のインデックス
            connections: 接続のリスト、各接続は以下の形式:
                {
                    "from": {"room": int, "door": int},
                    "to": {"room": int, "door": int}
                }
            team_id: チームID（指定しない場合は登録時のIDを使用）

        Returns:
            地図が正しいかどうか
        """
        if team_id is None:
            if self.team_id is None:
                raise ValueError(
                    "team_id が設定されていません。register() を先に実行するか、team_id を指定してください。"
                )
            team_id = self.team_id

        url = f"{self.base_url}/guess"
        data = {
            "id": team_id,
            "map": {
                "rooms": rooms,
                "startingRoom": starting_room,
                "connections": connections,
            },
        }

        response = requests.post(url, json=data)
        response.raise_for_status()

        result = response.json()
        return result["correct"]


def create_connection(
    from_room: int, from_door: int, to_room: int, to_door: int
) -> Dict[str, Dict[str, int]]:
    """
    接続オブジェクトを作成するヘルパー関数

    Args:
        from_room: 接続元の部屋インデックス
        from_door: 接続元のドア番号 (0-5)
        to_room: 接続先の部屋インデックス
        to_door: 接続先のドア番号 (0-5)

    Returns:
        接続オブジェクト
    """
    return {
        "from": {"room": from_room, "door": from_door},
        "to": {"room": to_room, "door": to_door},
    }


def main():
    """使用例"""
    # クライアント初期化
    client = ICFPCClient()

    # チーム登録
    # team_id = client.register(
    #     name="チーム名",
    #     pl="Python",
    #     email="your-email@example.com"
    # )
    # print(f"Team ID: {team_id}")

    # 既存のチームIDを使用する場合
    # client.team_id = "your-existing-team-id"

    # テスト問題を選択
    # problem = client.select("probatio")
    # print(f"Selected problem: {problem}")

    # 探索実行
    # result = client.explore(["0", "1", "2"])
    # print(f"Results: {result['results']}")
    # print(f"Query count: {result['queryCount']}")

    # 地図を推測して提出
    # connections = [
    #     create_connection(0, 0, 1, 3),
    #     create_connection(0, 1, 2, 4),
    #     # ... 他の接続
    # ]
    # is_correct = client.guess(
    #     rooms=[0, 1, 2],  # 各部屋のラベル
    #     starting_room=0,   # 開始部屋
    #     connections=connections
    # )
    # print(f"Guess is {'correct' if is_correct else 'incorrect'}")


if __name__ == "__main__":
    main()
