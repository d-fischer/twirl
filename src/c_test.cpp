#include <iostream>
#include <cstdlib>

extern "C" {
    struct ApiClient {};
    struct CAuthProvider {};
    struct User {
        char *id;
        char *login;
    };

    CAuthProvider *createStaticAuthProvider(char *clientId, char *accessToken);
    ApiClient *createApiClient(CAuthProvider *authProvider);
    User *getMe(ApiClient *twitchClient);
}

int main() {
    CAuthProvider *auth = createStaticAuthProvider(getenv("TWITCH_CLIENT_ID"), getenv("TWITCH_ACCESS_TOKEN"));
    ApiClient *client = createApiClient(auth);
    User *user = getMe(client);
    std::cout << user << std::endl;
    std::cout << user->login << " = " << user->id << std::endl;
}