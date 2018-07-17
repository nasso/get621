#ifndef GET621_H_
#define GET621_H_

#ifdef _WIN32
    #include <direct.h>
	#include <windows.h>
    #define getcwd _getcwd
	#define do_sleep(x) Sleep(x)
#else
    #include <unistd.h>
	#define do_sleep(x) usleep(x * 1000)
#endif

// We need some cooldown to make sure we ain't going too fast
#define GET621_REQ_COOLDOWN_MS 1200

#include "E6Post.h"
#include "E6Pool.h"

#include "json.hpp"

namespace get621 {
	int init();
	void cleanup();
	
	nlohmann::basic_json<> endpointGetJSON(std::string endpoint);
	
	int downloadURL(std::string url, FILE* dest, bool printProgress = false);
	bool isValidID(char* str);
	E6Post getPostByID(int32_t id);
	std::vector<E6Post> doSearch(std::string search, int32_t limit = 30);
}

#endif // GET621_H_
