#include <iostream>
#include <string>
#include <vector>
#include <curl/curl.h>
#include "json.hpp"

using json = nlohmann::json;

struct Connection
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
               const std::vector<Connection> &connections)
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

// Helper function to create connections
Connection create_connection(int from_room, int from_door, int to_room, int to_door)
{
    return {from_room, from_door, to_room, to_door};
}

// Example usage
int main()
{
    try
    {
        ICFPCClient client("");

        // Example map submission
        std::vector<int> rooms = {0, 1, 2};
        int starting_room = 0;
        std::vector<Connection> connections = {
            create_connection(0, 0, 1, 3),
            create_connection(0, 1, 2, 4)};

        bool is_correct = client.guess(rooms, starting_room, connections);
        std::cout << "Guess is " << (is_correct ? "correct" : "incorrect") << std::endl;
    }
    catch (const std::exception &e)
    {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }

    return 0;
}