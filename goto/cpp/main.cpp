#include <bits/stdc++.h>
#include "json.hpp"

#include <iostream>
#include <string>
#include <vector>
#include <curl/curl.h>

std::random_device rd;
const auto seed = rd();
std::mt19937 gen(seed);
using json = nlohmann::json;

using namespace std;

using json = nlohmann::json;


// ドアの位置を表す構造体
struct DoorPosition
{
    int room;
    int door;
};

// 部屋間の接続を表す構造体
struct Connection
{
    DoorPosition from;
    DoorPosition to;
};

// マップデータを表す構造体
struct MapData
{
    vector<int> rooms;
    int startingRoom;
    vector<Connection> connections;
};

// メインのゲームデータ構造体
struct Input
{
    string plan;
    vector<int> results;
    MapData mapData;
};

// JSON <-> 構造体の自動変換を設定
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(DoorPosition, room, door)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Connection, from, to)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(MapData, rooms, startingRoom, connections)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Input, plan, results, mapData)

struct Destination
{
    int room;
    int door;
};

struct GachiDestination
{
    int layer;
    int room;
    int door;
};

class Solver
{
public:
    int n, num_layers;
    vector<vector<int>> chalked;
    vector<int> plan, change_to, results;
    Input input;
    vector<vector<Destination>> edges;
    vector<vector<vector<GachiDestination>>> gachi_edges;
    bool ok = false;
    int max_depth = 0;
    chrono::steady_clock::time_point start_time = chrono::steady_clock::now();
    vector<bool> visited;

    Solver(Input input, int num_layers) : input(input), num_layers(num_layers)
    {
        // k 層の部屋の割当を解く
        n = input.mapData.rooms.size();
        chalked.assign(num_layers, vector<int>(n)); // 今のチョークの状態
        for (int i = 0; i < num_layers; ++i)
        {
            for (int j = 0; j < n; ++j)
            {
                chalked[i][j] = j % 4;
            }
        }

        for (int i = 0; i + 3 < input.plan.length(); i += 4)
        {
            plan.push_back(input.plan[i + 3] - '0');
            change_to.push_back(input.plan[i + 1] - '0');
        }
        change_to.push_back(input.plan[input.plan.length() - 2] - '0');
        assert(plan.size() == n * num_layers * 6);
        assert(change_to.size() == n * num_layers * 6 + 1);

        // グラフを構築
        edges.assign(n, vector<Destination>(6, {-1, -1}));
        for (const auto &conn : input.mapData.connections)
        {
            edges[conn.from.room][conn.from.door] = {conn.to.room, conn.to.door};
            edges[conn.to.room][conn.to.door] = {conn.from.room, conn.from.door};
        }

        // // edge を出力
        // for (int from = 0; from < edges.size(); from++)
        // {
        //     for (int door = 0; door < edges[from].size(); door++)
        //     {
        //         const auto &to = edges[from][door];
        //         cerr << "room: " << from << ", door: " << door << " -> ";
        //         cerr << "(room: " << to.room << ", door: " << to.door << ")" << endl;
        //     }
        // }

        gachi_edges = vector<vector<vector<GachiDestination>>>(num_layers, vector<vector<GachiDestination>>(n, vector<GachiDestination>(6, {-1, -1, -1})));

        // resultsを構築
        assert(input.results.size() % 2 == 0);
        for (int i = 0; i < input.results.size(); i += 2)
        {
            results.push_back(input.results[i]);
        }

        visited.assign(n, false);
    }

    vector<vector<vector<GachiDestination>>> solve()
    {
        const auto start_time = chrono::steady_clock::now();
        ok = dfs(0, 0, 0);
        cerr << "dfs result: " << ok << endl;
        cerr << "plan size: " << plan.size() << ", max depth: " << max_depth << endl;
        // 埋まってない gachi_edge を埋める
        for (int l = 0; l < num_layers; l++)
        {
            for (int r = 0; r < n; r++)
            {
                for (int d = 0; d < 6; d++)
                {
                    if (gachi_edges[l][r][d].layer == -1)
                    {
                        auto candidates = get_candidates(l, r, d);
                        assert(candidates.size() > 0);
                        // if (candidates.size() > 1)
                        // {
                        //     cerr << "Warning: multiple candidates for (" << l << ", " << r << ", " << d << ")" << endl;
                        // }
                        gachi_edges[l][r][d] = candidates[0];
                        gachi_edges[candidates[0].layer][candidates[0].room][candidates[0].door] = {l, r, d};
                    }
                }
            }
        }
        return gachi_edges;
    }

    bool dfs(int depth, int current_layer, int current_room)
    {
        // cerr << "@: " << depth << " " << current_layer << " " << current_room << endl;
        max_depth = max(max_depth, depth);
        if (depth == int(plan.size()))
        {
            // ベースケース: プランの最後に到達
            return true;
        }

        if (chalked[current_layer][current_room] != results[depth])
        {
            // cerr << "Chalk color mismatch at depth " << depth << ": expected " << results[depth] << ", got " << chalked[current_layer][current_room] << endl;
            return false;
        }

        const auto elapsed = chrono::steady_clock::now() - start_time;
        if (elapsed > chrono::seconds(1))
        {
            // cerr << "Time limit exceeded at depth " << depth << endl;
            return false;
        }

        bool before_visited = visited[current_room];
        visited[current_room] = true;
        const int door = plan[depth];

        auto candidates = get_candidates(current_layer, current_room, door);
        if (candidates.empty())
        {
            // cerr << "No candidates found at depth " << depth << endl;
            visited[current_room] = before_visited;
            return false;
        }

        // 新しいチョークの色に変える
        chalked[current_layer][current_room] = change_to[depth];

        // すでに辺が決まってたらそれを使う
        if (gachi_edges[current_layer][current_room][door].layer != -1)
        {
            const auto dest = gachi_edges[current_layer][current_room][door];
            bool ok = dfs(depth + 1, dest.layer, dest.room);
            if (ok)
                return true;
            chalked[current_layer][current_room] = results[depth]; // 戻す
            visited[current_room] = before_visited;
            return false;
        }

        for (const auto &dest : candidates)
        {
            gachi_edges[current_layer][current_room][door] = dest;
            gachi_edges[dest.layer][dest.room][dest.door] = {current_layer, current_room, door};
            bool ok = dfs(depth + 1, dest.layer, dest.room);
            if (ok)
                return true;
            gachi_edges[current_layer][current_room][door] = {-1, -1, -1};
            gachi_edges[dest.layer][dest.room][dest.door] = {-1, -1, -1};
        }

        chalked[current_layer][current_room] = results[depth]; // 戻す
        visited[current_room] = before_visited;
        return false;
    }

    vector<GachiDestination> get_candidates(int layer, int room, int door)
    {
        vector<GachiDestination> candidates;
        // edgeの行き先が決まってるなら返す
        if (gachi_edges[layer][room][door].layer != -1)
        {
            candidates.push_back(gachi_edges[layer][room][door]);
            return candidates;
        }

        // edgeの行き先が決まってないなら、layer を全探索
        const int to_room = edges[room][door].room;
        const int to_door = edges[room][door].door;
        for (int l = 0; l < num_layers; l++)
        {
            // 相手のエッジが決まってるならスキップ
            if (gachi_edges[l][to_room][to_door].layer != -1)
                continue;
            candidates.push_back({l, to_room, to_door});
        }
        if (!visited[to_room])
        {
            return {candidates[0]};
        }

        // candidates をシャッフル
        std::shuffle(candidates.begin(), candidates.end(), gen);
        return candidates;
    }
};

struct Connection2
{
    int from_room;
    int from_door;
    int to_room;
    int to_door;

    json to_json() const
    {
        return {
            {"from", {{"room", from_room}, {"door", from_door}}},
            {"to", {{"room", to_room}, {"door", to_door}}}};
    }
};

class ICFPCClient
{
private:
    std::string base_url = "https://31pwr5t6ij.execute-api.eu-west-2.amazonaws.com";
    std::string team_id;

    static size_t WriteCallback(void *contents, size_t size, size_t nmemb, std::string *userp)
    {
        userp->append((char *)contents, size * nmemb);
        return size * nmemb;
    }

public:
    ICFPCClient(const std::string &team_id = "") : team_id(team_id) {}

    void set_team_id(const std::string &id)
    {
        team_id = id;
    }

    bool guess(const std::vector<int> &rooms,
               int starting_room,
               const std::vector<Connection2> &connections)
    {

        if (team_id.empty())
        {
            throw std::runtime_error("team_id is not set");
        }

        // Prepare JSON payload
        json data;
        data["id"] = team_id;

        json map_data;
        map_data["rooms"] = rooms;
        map_data["startingRoom"] = starting_room;

        json connections_json = json::array();
        for (const auto &conn : connections)
        {
            connections_json.push_back(conn.to_json());
        }
        map_data["connections"] = connections_json;

        data["map"] = map_data;

        std::string json_str = data.dump();

        // Setup CURL
        CURL *curl = curl_easy_init();
        if (!curl)
        {
            throw std::runtime_error("Failed to initialize CURL");
        }

        std::string response_string;
        std::string url = base_url + "/guess";

        struct curl_slist *headers = nullptr;
        headers = curl_slist_append(headers, "Content-Type: application/json");

        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        curl_easy_setopt(curl, CURLOPT_POST, 1L);
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, json_str.c_str());
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response_string);

        CURLcode res = curl_easy_perform(curl);

        curl_slist_free_all(headers);

        if (res != CURLE_OK)
        {
            curl_easy_cleanup(curl);
            throw std::runtime_error("CURL request failed: " + std::string(curl_easy_strerror(res)));
        }

        long http_code = 0;
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &http_code);
        curl_easy_cleanup(curl);

        if (http_code != 200)
        {
            throw std::runtime_error("HTTP request failed with code: " + std::to_string(http_code));
        }

        // Parse response
        json response = json::parse(response_string);
        return response["correct"];
    }
};

int main()
{
    cerr << "seed: " << seed << endl;

    // JSONファイルを読み込み
    ifstream file("input.json");
    json j = json::parse(file);

    // JSONから構造体に自動変換
    Input gameData = j.get<Input>();

    // データを使用
    cerr << "Plan length: " << gameData.plan.length() << endl;
    cerr << "Results count: " << gameData.results.size() << endl;
    cerr << "Starting room: " << gameData.mapData.startingRoom << endl;
    cerr << "Total rooms: " << gameData.mapData.rooms.size() << endl;
    cerr << "Total connections: " << gameData.mapData.connections.size() << endl;

    // // 最初の接続を表示
    // if (!gameData.mapData.connections.empty())
    // {
    //     const auto &firstConnection = gameData.mapData.connections[0];
    //     cerr << "First connection: Room " << firstConnection.from.room
    //          << " Door " << firstConnection.from.door
    //          << " -> Room " << firstConnection.to.room
    //          << " Door " << firstConnection.to.door << endl;
    // }

    // // 最初の10個の結果を表示
    // cerr << "First 10 results: ";
    // for (size_t i = 0; i < min(10ul, gameData.results.size()); ++i)
    // {
    //     cerr << gameData.results[i] << " ";
    // }
    // cerr << endl;

    const int num_layers = 3;
    Solver solver(gameData, num_layers);
    auto edge = solver.solve();

    // edge を出力
    // for (int l = 0; l < edge.size(); l++)
    // {
    //     for (int r = 0; r < edge[l].size(); r++)
    //     {
    //         for (int d = 0; d < edge[l][r].size(); d++)
    //         {
    //             cerr << "layer: " << l << ", room: " << r << ", door: " << d << " -> ";
    //             cerr << "(layer: " << edge[l][r][d].layer << ", room: " << edge[l][r][d].room << ", door: " << edge[l][r][d].door << ")" << endl;
    //             cerr << "input edge: (room: " << solver.edges[r][d].room << ", door: " << solver.edges[r][d].door << ")" << endl;
    //         }
    //     }
    // }

    const auto team_id = "";
    ICFPCClient client(team_id);
    vector<int> rooms;
    for (int l = 0; l < num_layers; l++)
    {
        for (int r = 0; r < solver.n; r++)
        {
            rooms.push_back(r % 4);
        }
    }
    vector<Connection2> connections;
    for (int l = 0; l < num_layers; l++)
    {
        for (int r = 0; r < solver.n; r++)
        {
            for (int d = 0; d < 6; d++)
            {
                const auto to = edge[l][r][d];
                if (to.layer < l || (to.layer == l && to.room < r)) // 二重に送らないようにする
                    continue;
                connections.push_back({l * solver.n + r, d, to.layer * solver.n + to.room, to.door});
            }
        }
    }

    if (solver.ok)
    {
        bool result = client.guess(rooms, gameData.mapData.startingRoom, connections);
        cerr << "Guess result: " << (result ? "correct" : "incorrect");
        cout << result << endl;
    }
    else
    {
        cout << false << endl;
    }

    return 0;
}
