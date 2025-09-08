# ICFPC 2025 API Server Simulator

## 使い方

```
uv run main.py
```

で、`localhost:5000` に立ち上がります。

## 設定

```
{
  "seed": 42,
  "debug": true,
  "graph_folder": "template"
}
```

- seed (option[int]): 存在すると、乱数の seed となる
- debug (option[bool]): 存在すると、デバッグモードとなり、デバッグ出力される
- graph_folder (option[string]): 指定すると、graph_folder 内の指定したフォルダを参照して、その中にある json ファイルからグラフを生成する。json ファイルは guess とかで投げる形。指定しなければ自動生成。
