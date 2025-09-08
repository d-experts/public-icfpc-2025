#line 2 "src/template/template.hpp"
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
using P = pair<long long, long long>;
#define rep(i, a, b) for(long long i = (a); i < (b); ++i)
#define rrep(i, a, b) for(long long i = (a); i >= (b); --i)
constexpr long long inf = 4e18;
struct SetupIO {
    SetupIO() {
        ios::sync_with_stdio(0);
        cin.tie(0);
        cout << fixed << setprecision(30);
    }
} setup_io;
#line 3 "src/graph/graph_template.hpp"
template <typename T>
struct Edge {
    int from, to;
    T cost;
    int idx;
    Edge()
        : from(-1), to(-1), cost(-1), idx(-1) {}
    Edge(const int from, const int to, const T& cost = 1, const int idx = -1)
        : from(from), to(to), cost(cost), idx(idx) {}
    operator int() const {
        return to;
    }
};
template <typename T>
struct Graph {
    Graph(const int N)
        : n(N), es(0), g(N) {}
    int size() const {
        return n;
    }
    int edge_size() const {
        return es;
    }
    void add_edge(const int from, const int to, const T& cost = 1) {
        assert(0 <= from and from < n);
        assert(0 <= to and to < n);
        g[from].emplace_back(from, to, cost, es);
        g[to].emplace_back(to, from, cost, es++);
    }
    void add_directed_edge(const int from, const int to, const T& cost = 1) {
        assert(0 <= from and from < n);
        assert(0 <= to and to < n);
        g[from].emplace_back(from, to, cost, es++);
    }
    inline vector<Edge<T>>& operator[](const int& k) {
        assert(0 <= k and k < n);
        return g[k];
    }
    inline const vector<Edge<T>>& operator[](const int& k) const {
        assert(0 <= k and k < n);
        return g[k];
    }

   private:
    int n, es;
    vector<vector<Edge<T>>> g;
};
template <typename T>
using Edges = vector<Edge<T>>;



class TimeKeeper {
    public:
    // コンストラクタ：limitMillis に制限時間（ミリ秒）を指定
    TimeKeeper(long long limitMillis)
    : limitTime(limitMillis), startTime(std::chrono::steady_clock::now())
    {
    }
    
    // インスタンス生成直後は経過時間は0ミリ秒とみなす
    
    // 現在の経過時間（ミリ秒）を返す
    long long getNowTime() const {
        auto now = std::chrono::steady_clock::now();
        auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - startTime);
        return elapsed.count();
    }
    
    // 制限時間を超えているかを返す
    bool isTimeOver() const {
        return getNowTime() >= limitTime.count();
    }
    
    private:
    std::chrono::steady_clock::time_point startTime;     // 開始時間
    std::chrono::milliseconds limitTime;                 // 制限時間（ミリ秒）
};


int time_threshold = 20000;
int n = 6; // 部屋数
int m = 12; // クエリ長 = nm

Graph<int> generate() {
    Graph<int> g(6*n);
    vector<int> yet;
    rep(i, 0, 6*n) yet.push_back(i);

    while(yet.size() > 0) {
        int r1 = rand() % yet.size();
        int n1 = yet[r1];
        yet.erase(yet.begin()+r1);
        int r2 = rand() % yet.size();
        int n2 = yet[r2];
        yet.erase(yet.begin()+r2);

        g.add_edge(n1, n2);
    }

    return g;
}


int calc_score(Graph<int> g, vector<int> query) {
    map<string, int> signatures;

    const int DUP_LEN = 2;
    int now = 0;
    vector<bool> visited(6*n);
    rep(i, 0, query.size()) {
        int next = g[now*6+query[i]][0].to / 6;
        visited[now*6+query[i]] = true;
        now = next;

        if (i + DUP_LEN < query.size()) {
            string sig = "";
            for (int j = 0; j < DUP_LEN; j++) {
                sig += to_string(query[i+j]);
            }
            signatures[sig]++;
        }
    }

    int score = 0;
    rep(i, 0, visited.size()) {
        if(!visited[i]) score++;
    }

    score *= 1000;

    for (auto [sig, cnt] : signatures) {
        score -= 10 * cnt * (cnt - 1);
    }

    return score;
}


int main() {
    std::mt19937_64 mt(0);
    // 焼きなまし法のパラメータ設定
    double T = 10.0;       // 初期温度
    double cooling_rate = 0.99999999; // 冷却率

    TimeKeeper tk(time_threshold);
    vector<int> query;
    rep(i, 0, m*n) query.push_back(rand()%6);
    
    int current_score = 0;
    rep(i, 0, 500) {
        Graph<int> g = generate();
        current_score += calc_score(g, query);
    }

    while(!tk.isTimeOver()) {
        int pos = rand() % (m*n);
        int pre_door = query[pos];
        int door_cand = (pre_door + rand() % 5) % 6;

        query[pos] = door_cand;

        int score = 0;
        rep(i, 0, 500) {
            Graph<int> g = generate();
            score += calc_score(g, query);
        }

        double diff = current_score - score; // スコア最小化なので diff > 0 が改善
        if (diff > 0 || exp(diff / T) > (double)mt() / mt.max()) {
        // if(current_score > score) {
            current_score = score;
            cerr << current_score << endl;
        } else {
            query[pos] = pre_door;
        }
    }

    rep(i, 0, query.size()) {
        cout << query[i];
    }

    map<string ,int> signatures;
    const int DUP_LEN = 2;
    for (int i = 0; i < query.size(); i++) {
        if (i + DUP_LEN < query.size()) {
            string sig = "";
            for (int j = 0; j < DUP_LEN; j++) {
                sig += to_string(query[i+j]);
            }
            signatures[sig]++;
        }
    }

    for (auto [sig, cnt] : signatures) {
        cout << sig << " " << cnt << endl;
    }
}
