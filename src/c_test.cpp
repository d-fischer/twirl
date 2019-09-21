#include <iostream>
#include <cstdlib>

extern "C" {
    struct TwitchClient {};
    struct User {
        char *id;
        char *login;
    };

    TwitchClient *createTwitchClientWithCredentials(char *clientId, char *accessToken);
    User *getMe(TwitchClient *twitchClient);
}

int main() {
    TwitchClient *client = createTwitchClientWithCredentials(getenv("TWITCH_CLIENT_ID"), getenv("TWITCH_ACCESS_TOKEN"));
    User *user = getMe(client);
    std::cout << user << std::endl;
    std::cout << user->login << " = " << user->id << std::endl;
}