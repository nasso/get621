#ifdef _WIN32
    #include <direct.h>
	#include <windows.h>
    #define getcwd _getcwd
	#define do_sleep(x) Sleep(x)
#else
    #include <unistd.h>
	#define do_sleep(x) usleep(x * 1000)
#endif

#include <iostream>
#include <curl/curl.h>

#include "json.hpp"
#include "url_encode.hpp"

const char kPathSeparator =
#ifdef _WIN32
                            '\\';
#else
                            '/';
#endif

static std::string cwd;

static size_t strAppendCallback(void *contents, size_t size, size_t nmemb, void *userp) {
	((std::string*)userp)->append((char*) contents, size * nmemb);
	return size * nmemb;
}

static size_t fileWriteCallback(void *contents, size_t size, size_t nmemb, void *userp) {
	size_t written = fwrite(contents, size, nmemb, (FILE*) userp);
	return written;
}

static CURL *curl;
static struct curl_slist *hlist = NULL;

static int setup_curl() {
	curl = curl_easy_init();
	if(!curl) {
		std::cerr << "Couldn't initialize cURL." << std::endl;
		return 1;
	}
	
	hlist = curl_slist_append(hlist, "User-Agent: get621/0.1 (by yann-the-leopard on e621)");
	
	return 0;
}

static void cleanup_curl() {
	curl_easy_cleanup(curl);
	curl_slist_free_all(hlist);
}

static nlohmann::basic_json<> getjson_curl(std::string url) {
	CURLcode res;
	std::string readBuffer;
	
	curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
	curl_easy_setopt(curl, CURLOPT_HTTPHEADER, hlist);
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, strAppendCallback);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, &readBuffer);
	
	res = curl_easy_perform(curl);
	
	if(res != CURLE_OK) {
		std::cerr << "Couldn't perform request: " << curl_easy_strerror(res) << std::endl;
		return NULL;
	}
	
	return nlohmann::json::parse(readBuffer);
}

static int download_curl(std::string url, std::string dest, bool printProgress) {
	CURLcode res;
	
	FILE* destFile = fopen(dest.c_str(), "wb");
	if(!destFile) {
		std::cerr << "Couldn't open the file." << std::endl;
		return 1;
	}
	
	curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
	// curl_easy_setopt(curl, CURLOPT_HTTPHEADER, hlist);
	curl_easy_setopt(curl, CURLOPT_NOPROGRESS, printProgress ? 0L : 1L);
	curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, fileWriteCallback);
	curl_easy_setopt(curl, CURLOPT_WRITEDATA, destFile);
	
	res = curl_easy_perform(curl);
	fclose(destFile);
	
	if(res != CURLE_OK) {
		std::cerr << "Couldn't perform request: " << curl_easy_strerror(res) << std::endl;
		return 1;
	}
	
	return 0;
}

static void printUsage() {
	std::cout
		<< "get621 - 0.1 (by nasso <https://github.com/nasso>)" << std::endl << std::endl
		<< "Usage: get621 TAGS..." << std::endl
		<< "   or: get621 -pool [POOL_ID]" << std::endl
#ifdef NSFW
		<< "Download files from <https://e621.net/>." << std::endl;
#else
		<< "Download files from <https://e926.net/>." << std::endl;
#endif
}

static int searchAndSave(int argc, char** argv) {
	std::stringstream searchQueryBuilder;
	
	bool ordered = false;
	for(int i = 1; i < argc; i++) {
		searchQueryBuilder << argv[i] << " ";
		
		ordered |= strstr(argv[i], "order:") != NULL;
	}
	
	if(!ordered && argc < 5) {
		searchQueryBuilder << "order:random";
	}
	
	std::string searchQuery = searchQueryBuilder.str();
	
	std::stringstream urlBuilder;
	
#ifdef NSFW
	std::cout << "E621: " << searchQuery << std::endl;
	urlBuilder << "https://e621.net/post/index.json?limit=1&tags=" << url_encode(searchQuery);
#else
	std::cout << "E926: " << searchQuery << std::endl;
	urlBuilder << "https://e926.net/post/index.json?limit=1&tags=" << url_encode(searchQuery);
#endif
	
	if(setup_curl() != 0) return 1;
	auto data = getjson_curl(urlBuilder.str());
	
	if(data == NULL) return 1;
	
	/*
	for(size_t i = 0; i < data.size(); i++) {
		auto post = data[i];
		std::cout << (i + 1) << ". #" << post["id"] <<
			"\t(r:" << post["rating"].get<std::string>() << ",\ts:" << post["score"] << ",\tf:" << post["fav_count"] << ")\tby ";
		
		auto artists = post["artist"];
		for(size_t j = 0; j < artists.size(); j++) {
			if(j != 0) std::cout << (j == artists.size() - 1 ? " and " : ", ");
			std::cout << artists[j].get<std::string>();
		}
		
		std::cout << std::endl;
	}
	*/
	
	if(data.size() == 0) std::cout << "No post found." << std::endl;
	else {
		auto post = data[0];
		
		std::cout << "#" << post["id"] << "\tby ";

		auto artists = post["artist"];
		for(size_t j = 0; j < artists.size(); j++) {
			if(j != 0) std::cout << (j == artists.size() - 1 ? " and " : ", ");
			std::cout << artists[j].get<std::string>();
		}
		
		std::string rating = post["rating"].get<std::string>();
		
		std::cout << std::endl << "Rating: ";
		if(rating[0] == 's') std::cout << "Safe" << std::endl;
		else if(rating[0] == 'q') std::cout << "Questionable" << std::endl;
		else if(rating[0] == 'e') std::cout << "Explicit" << std::endl;
		else std::cout << rating << std::endl;

		auto type = post["file_ext"].get<std::string>();
		std::cout
			<< "Score: " << post["score"] << std::endl
			<< "Favs:" << post["fav_count"] << std::endl
			<< "Type: " << type << std::endl
			<< "Tags: " << post["tags"].get<std::string>() << std::endl;
		
		std::string desc = post["description"].get<std::string>();
		if(!desc.empty()) std::cout << "Description:" << desc << std::endl;

		std::cout << std::endl << "Downloading to " << post["id"] << "." << type << "..." << std::endl;
		
		// Now save to <id>.<type>
		std::stringstream destPathBuilder;
		destPathBuilder << cwd << kPathSeparator << post["id"] << "." << type;
		
		download_curl(post["file_url"].get<std::string>(), destPathBuilder.str(), true);
	}
	
	cleanup_curl();
	
	return 0;
}

static bool isValidID(char* str) {
	for(int i = 0, l = strlen(str); i < l; i++) {
		if(str[i] < '0' || str[i] > '9') return false;
	}
	
	return true;
}

static int savePoolPage(char* poolID, int page, int* counter, int* postCount) {
	std::stringstream urlBuilder;
	
#ifdef NSFW
	urlBuilder << "https://e621.net/pool/show.json?id=" << poolID << "&page=" << page;
#else
	urlBuilder << "https://e926.net/pool/show.json?id=" << poolID << "&page=" << page;
#endif
	
	auto data = getjson_curl(urlBuilder.str());
	
	if(data == NULL) return -1;
	
	auto success = data["success"]; 
	if(success.is_boolean() && !data["success"]) {
		std::cout << "Can't find pool " << poolID << ": " << data["reason"] << std::endl;
		return -1;
	}
	
	if(postCount != NULL) (*postCount) = data["post_count"];
	
	auto poolName = data["name"].get<std::string>();
	std::cout << "Downloading page " << page << " of " << poolName << "..." << std::endl;
	
	auto posts = data["posts"];
	
	for(size_t i = 0; i < posts.size(); i++) {
		auto post = posts[i];
		
		std::cout << "Post " << i << " of this page..." << std::endl;
		
		// Save to i.<type>
		std::stringstream destPathBuilder;
		destPathBuilder << cwd << kPathSeparator << poolID << "-" << ((*counter)++) << "_" << post["id"] << "." << post["file_ext"].get<std::string>();
		
		download_curl(post["file_url"].get<std::string>(), destPathBuilder.str(), false);
	}
	
	return posts.size();
}

static int savePool(char* poolID) {
	if(setup_curl() != 0) return 1;
	
	int postsSaved = 0;
	int postsLeft = -1;
	int page = 1;
	
	int counter = 1;
	
	do {
		std::cout << "Please wait... 3" << std::flush;
		do_sleep(1000);
		std::cout << "\rPlease wait... 2" << std::flush;
		do_sleep(1000);
		std::cout << "\rPlease wait... 1" << std::flush;
		do_sleep(1000);
		std::cout << "\rPlease wait... 0" << std::flush << "\r";
		
		postsSaved = savePoolPage(poolID, page++, &counter, postsLeft < 0 ? &postsLeft : NULL);
		if(postsSaved < 0) return 1;
		
		postsLeft -= postsSaved;
	} while(postsLeft > 0);
	
	cleanup_curl();
	
	return 0;
}

int main(int argc, char** argv) {
	auto cwdBuf = getcwd(NULL, 0);
	cwd = cwdBuf;
	free(cwdBuf);
	
	if(argc == 1) {
		printUsage();
		return 0;
	} else {
		if(strcmp(argv[1], "-pool") == 0) {
			if(argc < 3 || !isValidID(argv[2])) {
				std::cout << "Please specify a valid pool ID." << std::endl;
				return 0;
			}
			
			return savePool(argv[2]);
		} else return searchAndSave(argc, argv);
	}
    
	return 0;
}
